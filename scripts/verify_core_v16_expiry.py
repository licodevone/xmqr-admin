#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""A21: private native v16 backup/restore and real v15 migration, never operator state."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import socket
import stat
import struct
import subprocess
import tempfile
import time
import tomllib
import unittest
import zlib


STATE_FILES = ("state.snapshot", "state.wal")
RETAINED = b"a21/retained"
OFFLINE = b"a21/offline"
WILL = b"a21/will"
MAX_SNAPSHOT = 2 * 1024 * 1024  # Fixture bound, not a changed broker quota.
MAX_PACKET = 65_536


def require(condition, code):
    if not condition:
        raise AssertionError(code)


def digest(path):
    hasher = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(65_536):
            hasher.update(chunk)
    return hasher.hexdigest()


def state_hashes(directory):
    return {name: digest(directory / name) for name in STATE_FILES}


def snapshot_document(data):
    require(26 <= len(data) <= MAX_SNAPSHOT, "snapshot_size")
    magic, version, sequence, entries = struct.unpack_from("<8sHQI", data)
    require(magic == b"MQSNAP01" and version == 1 and entries <= 128, "snapshot_header")
    end = len(data) - 4
    require(zlib.crc32(data[:end]) == int.from_bytes(data[end:], "little"), "snapshot_crc")
    cursor, values = 22, {}
    for _ in range(entries):
        require(cursor + 6 <= end, "snapshot_entry_header")
        key_size, value_size = struct.unpack_from("<HI", data, cursor)
        cursor += 6
        require(1 <= key_size <= 1024 and value_size <= 1_048_576
                and cursor + key_size + value_size <= end, "snapshot_entry_size")
        key = data[cursor:cursor + key_size]
        cursor += key_size
        require(key not in values, "snapshot_duplicate_key")
        values[key] = data[cursor:cursor + value_size]
        cursor += value_size
    require(cursor == end and b"mqtt311.state.v1" in values, "snapshot_trailing_or_mqtt_absent")
    document = tomllib.loads(values[b"mqtt311.state.v1"].decode("utf-8"))
    require(isinstance(document.get("state"), dict), "mqtt_state_absent")
    return document, sequence


def document(directory):
    path = directory / "state.snapshot"
    require(path.stat().st_size <= MAX_SNAPSHOT, "snapshot_fixture_bound")
    return snapshot_document(path.read_bytes())[0]


def check_payload(message, expected):
    value = message.get("payload")
    require(isinstance(value, str) and len(value) <= 8192 and len(value) % 2 == 0
            and all(character in "0123456789abcdef" for character in value),
            "fixture_payload_canonical_hex")
    require(bytes.fromhex(value) == expected, "fixture_payload_bytes")


def check_document(envelope, version, original):
    require(envelope.get("version") == version, "mqtt_document_version")
    state = envelope["state"]
    retained = state["retained"][RETAINED.decode()]
    offline = state["sessions"]["a21-offline"]["offline"]
    require(len(offline) == 1 and offline[0]["topic"] == OFFLINE.decode(), "offline_message")
    check_payload(retained, b"owned-retained")
    check_payload(offline[0], b"owned-offline")
    require(retained["retain"] and offline[0]["qos"] == 1, "fixture_message_qos_and_retain")
    for message in (retained, offline[0]):
        require(message.get("expiry_at_unix_ms", 0) > 0, "message_deadline")
        require(message.get("expiry_interval_seconds") == original, "original_interval")
    will = state["pending_wills"]["a21-pending"]
    require(will["delay_seconds"] == 120
            and will["message_expiry_interval_seconds"] == 37, "pending_will_intervals")
    check_payload(will["message"], b"owned-will")
    require(will["message"]["topic"] == WILL.decode()
            and "expiry_at_unix_ms" not in will["message"]
            and "expiry_interval_seconds" not in will["message"], "will_expiry_starts_on_publication")
    if version == 16:
        require(state.get("expiry_clock_unix_ms", 0) > 0, "v16_watermark")
    else:
        require("expiry_clock_unix_ms" not in state, "legacy_watermark_fabricated")


def check_manifest(directory):
    manifest = tomllib.loads((directory / "manifest.toml").read_text(encoding="utf-8"))
    require(manifest.get("format_version") == 1
            and manifest.get("state_format_version") == 1, "backup_manifest_version")
    entries = manifest.get("files", [])
    require([entry.get("name") for entry in entries] == list(STATE_FILES), "backup_manifest_paths")
    require(sorted(path.name for path in directory.iterdir())
            == sorted((*STATE_FILES, "manifest.toml")), "backup_extra_files")
    for entry in entries:
        path = directory / entry["name"]
        require(not path.is_symlink() and path.is_file(), "backup_nonregular_entry")
        require(path.stat().st_size == entry.get("length")
                and digest(path) == entry.get("sha256"), "backup_manifest_hash")
    require(stat.S_IMODE(directory.stat().st_mode) == 0o700, "backup_directory_mode")
    for name in (*STATE_FILES, "manifest.toml"):
        require(stat.S_IMODE((directory / name).stat().st_mode) == 0o600, "backup_file_mode")
    return state_hashes(directory)


def varint(value):
    require(0 <= value <= 268_435_455, "variable_integer_bound")
    encoded = bytearray()
    while True:
        digit = value % 128
        value //= 128
        encoded.append(digit | (0x80 if value else 0))
        if not value:
            return bytes(encoded)


def read_varint(data, cursor):
    value = 0
    for index in range(4):
        require(cursor < len(data), "truncated_variable_integer")
        digit = data[cursor]
        cursor += 1
        value += (digit & 127) << (index * 7)
        if not digit & 128:
            return value, cursor
    raise AssertionError("overlong_variable_integer")


def field(value):
    require(len(value) <= 65_535, "field_size")
    return len(value).to_bytes(2, "big") + value


def send(peer, header, body):
    require(len(body) <= MAX_PACKET, "fixture_packet_size")
    peer.sendall(bytes([header]) + varint(len(body)) + body)


def exact(peer, count):
    value = bytearray()
    while len(value) < count:
        chunk = peer.recv(count - len(value))
        require(bool(chunk), "unexpected_mqtt_eof")
        value.extend(chunk)
    return bytes(value)


def packet(peer):
    header = exact(peer, 1)[0]
    length, factor = 0, 1
    for _ in range(4):
        digit = exact(peer, 1)[0]
        length += (digit & 127) * factor
        require(length <= MAX_PACKET, "received_packet_size")
        if not digit & 128:
            return header, exact(peer, length)
        factor *= 128
    raise AssertionError("received_remaining_length")


def connect(port, identity, *, persistent=False, clean=True, will=False):
    peer = socket.create_connection(("127.0.0.1", port), timeout=4)
    peer.settimeout(4)
    properties = b"\x11\xff\xff\xff\xff" if persistent else b""
    flags = (2 if clean else 0) | (0x2c if will else 0)
    body = (b"\x00\x04MQTT\x05" + bytes([flags]) + b"\x00\x1e"
            + varint(len(properties)) + properties + field(identity))
    if will:
        will_properties = b"\x18\x00\x00\x00\x78\x02\x00\x00\x00\x25"
        body += varint(len(will_properties)) + will_properties + field(WILL) + field(b"owned-will")
    try:
        send(peer, 0x10, body)
        header, acknowledgement = packet(peer)
        require(header == 0x20 and len(acknowledgement) >= 3
                and acknowledgement[1] == 0 and acknowledgement[0] in (0, 1), "connack")
        return peer, bool(acknowledgement[0])
    except BaseException:
        peer.close()
        raise


def subscribe(peer, topic, qos):
    send(peer, 0x82, b"\x00\x07\x00" + field(topic) + bytes([qos]))
    require(packet(peer) == (0x90, b"\x00\x07\x00" + bytes([qos])), "suback")


def disconnect(peer, graceful=True):
    try:
        if graceful:
            send(peer, 0xe0, b"")
        else:
            peer.shutdown(socket.SHUT_WR)
        require(peer.recv(1) == b"", "disconnect_cleanup_barrier")
    finally:
        peer.close()


def publish(peer, topic, payload, identifier, retain=False):
    properties = b"\x02\x00\x00\x00\x1f"
    body = field(topic) + identifier.to_bytes(2, "big") + varint(len(properties)) + properties + payload
    send(peer, 0x32 | int(retain), body)
    header, acknowledgement = packet(peer)
    require(header == 0x40 and acknowledgement[:2] == identifier.to_bytes(2, "big")
            and (len(acknowledgement) == 2 or acknowledgement[2] < 0x80), "puback")


def publication_fields(header, body):
    require(header >> 4 == 3 and len(body) >= 2, "publish_expected")
    topic_size = int.from_bytes(body[:2], "big")
    cursor = 2 + topic_size
    require(cursor <= len(body), "publish_topic_size")
    topic = body[2:cursor]
    qos = (header >> 1) & 3
    require(qos in (0, 1), "fixture_delivery_qos")
    identifier = None
    if qos:
        require(cursor + 2 <= len(body), "publish_identifier")
        identifier = body[cursor:cursor + 2]
        cursor += 2
    size, cursor = read_varint(body, cursor)
    end = cursor + size
    require(end <= len(body), "publish_properties_size")
    expiry = None
    while cursor < end:
        property_id = body[cursor]
        cursor += 1
        require(property_id == 0x02 and expiry is None and cursor + 4 <= end,
                "unexpected_publish_property")
        expiry = int.from_bytes(body[cursor:cursor + 4], "big")
        cursor += 4
    return topic, body[end:], qos, identifier, expiry


def receive(peer, topic, payload, qos):
    header, body = packet(peer)
    actual_topic, actual_payload, actual_qos, identifier, expiry = publication_fields(header, body)
    require((actual_topic, actual_payload, actual_qos) == (topic, payload, qos), "restored_publication")
    require(expiry is not None and 0 < expiry <= 31, "forwarded_expiry_range")
    if qos:
        send(peer, 0x40, identifier)
    return expiry


def free_port():
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        return probe.getsockname()[1]


def environment(state, port):
    # Inherited operator MQTT paths/listeners/certificates cannot enter a test.
    result = {key: value for key, value in os.environ.items() if not key.startswith("MQTT_")}
    result.update(MQTT_MODE="open-lab", MQTT_STATE_DIR=str(state),
                  MQTT_BIND=f"127.0.0.1:{port}", TMPDIR="/var/tmp", RUST_LOG="error")
    return result


class OwnedBroker:
    def __init__(self, binary, state):
        self.port = free_port()
        self.process = subprocess.Popen([str(binary)], env=environment(state, self.port),
                                        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        deadline = time.monotonic() + 8
        try:
            while time.monotonic() < deadline:
                require(self.process.poll() is None, "broker_stopped_during_startup")
                try:
                    with socket.create_connection(("127.0.0.1", self.port), timeout=0.1):
                        return
                except OSError:
                    time.sleep(0.02)
            raise AssertionError("broker_startup_deadline")
        except BaseException:
            self.cleanup()
            raise

    def stop(self):
        require(self.process.poll() is None, "broker_stopped_before_shutdown")
        self.process.terminate()
        try:
            code = self.process.wait(timeout=8)
        except subprocess.TimeoutExpired:
            self.cleanup()
            raise AssertionError("broker_shutdown_deadline") from None
        require(code == 0, "broker_shutdown_exit")

    def cleanup(self):
        if self.process.poll() is None:
            self.process.kill()
            self.process.wait(timeout=3)


def admin(binary, *arguments, success=True):
    result = subprocess.run([str(binary), *map(str, arguments)], capture_output=True,
                            env={**os.environ, "TMPDIR": "/var/tmp"}, timeout=12)
    require((result.returncode == 0) == success, "admin_exit_contract")


def seed(binary, admin_binary, root, label):
    state = root / label
    state.mkdir(mode=0o700)
    broker = OwnedBroker(binary, state)
    peers = []
    try:
        subscriber, present = connect(broker.port, b"a21-offline", persistent=True)
        peers.append(subscriber)
        require(not present, "fresh_session_present")
        subscribe(subscriber, OFFLINE, 1)
        disconnect(subscriber)
        peers.remove(subscriber)
        publisher, _ = connect(broker.port, b"a21-publisher")
        peers.append(publisher)
        publish(publisher, RETAINED, b"owned-retained", 1, True)
        publish(publisher, OFFLINE, b"owned-offline", 2)
        pending, _ = connect(broker.port, b"a21-pending", persistent=True, will=True)
        peers.append(pending)
        disconnect(pending, graceful=False)
        peers.remove(pending)
        denied = root / (label + "-active-denied")
        admin(admin_binary, "state", "backup", "--state-dir", state,
              "--destination", denied, success=False)
        require(not denied.exists(), "active_backup_partial_destination")
        disconnect(publisher)
        peers.remove(publisher)
        broker.stop()
        return state
    finally:
        for peer in peers:
            peer.close()
        broker.cleanup()


def backup_restore(admin_binary, state, root, label):
    backup, restored = root / (label + "-backup"), root / (label + "-restored")
    admin(admin_binary, "state", "backup", "--state-dir", state, "--destination", backup)
    admin(admin_binary, "state", "verify", "--backup", backup)
    hashes = check_manifest(backup)
    require(hashes == state_hashes(state), "backup_source_bytes")
    admin(admin_binary, "state", "restore", "--backup", backup, "--state-dir", restored)
    require(state_hashes(restored) == hashes, "restore_bytes")
    require(not (restored / ".mqtt-broker-profile").exists(), "backup_included_profile")
    marker = state / ".mqtt-broker-profile"
    require(marker.read_bytes().strip() == b"open-lab", "fixture_profile")
    shutil.copy2(marker, restored / marker.name)
    return backup, restored, hashes


def mqtt_after_restore(binary, restored):
    broker = OwnedBroker(binary, restored)
    peers = []
    try:
        subscriber, present = connect(broker.port, b"a21-offline", persistent=True, clean=False)
        peers.append(subscriber)
        require(present, "restored_session_absent")
        offline_expiry = receive(subscriber, OFFLINE, b"owned-offline", 1)
        disconnect(subscriber)
        peers.remove(subscriber)
        observer, _ = connect(broker.port, b"a21-observer")
        peers.append(observer)
        subscribe(observer, RETAINED, 0)
        retained_expiry = receive(observer, RETAINED, b"owned-retained", 0)
        disconnect(observer)
        peers.remove(observer)
        broker.stop()
        return {"offline_remaining_seconds": offline_expiry,
                "retained_remaining_seconds": retained_expiry}
    finally:
        for peer in peers:
            peer.close()
        broker.cleanup()


def negatives(admin_binary, state, backup, root):
    expected = state_hashes(state)
    destination = root / "exists"
    destination.mkdir(mode=0o700)
    sentinel = destination / "owned-sentinel"
    sentinel.write_bytes(b"preserve")
    admin(admin_binary, "state", "restore", "--backup", backup,
          "--state-dir", destination, success=False)
    admin(admin_binary, "state", "backup", "--state-dir", state,
          "--destination", destination, success=False)
    require(sentinel.read_bytes() == b"preserve", "existing_destination_changed")
    with (state / "state.lock").open("rb") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        locked = root / "lock-denied"
        admin(admin_binary, "state", "backup", "--state-dir", state,
              "--destination", locked, success=False)
        require(not locked.exists(), "lock_partial_destination")
    require(state_hashes(state) == expected, "negative_changed_source_state")
    corrupt = root / "corrupt"
    shutil.copytree(backup, corrupt)
    damaged = bytearray((corrupt / "state.snapshot").read_bytes())
    damaged[-1] ^= 1
    (corrupt / "state.snapshot").write_bytes(damaged)
    corrupt_hashes = state_hashes(corrupt)
    admin(admin_binary, "state", "verify", "--backup", corrupt, success=False)
    denied = root / "corrupt-restore-denied"
    admin(admin_binary, "state", "restore", "--backup", corrupt,
          "--state-dir", denied, success=False)
    require(not denied.exists() and state_hashes(corrupt) == corrupt_hashes,
            "corrupt_backup_repaired_or_restored")
    link = root / "backup-link"
    link.symlink_to(backup, target_is_directory=True)
    admin(admin_binary, "state", "verify", "--backup", link, success=False)
    entry = root / "entry-link"
    shutil.copytree(backup, entry)
    (entry / "state.snapshot").unlink()
    (entry / "state.snapshot").symlink_to(backup / "state.snapshot")
    admin(admin_binary, "state", "verify", "--backup", entry, success=False)
    destination_link = root / "destination-link"
    destination_link.symlink_to(destination, target_is_directory=True)
    admin(admin_binary, "state", "restore", "--backup", backup,
          "--state-dir", destination_link, success=False)
    require(sentinel.read_bytes() == b"preserve" and check_manifest(backup) == expected,
            "symlink_negative_changed_original")


def legacy_rejects(legacy_binary, restored, root):
    candidate = root / "legacy-rejects-v16"
    shutil.copytree(restored, candidate)
    before = state_hashes(candidate)
    result = subprocess.run([str(legacy_binary)], env=environment(candidate, free_port()),
                            capture_output=True, timeout=5)
    require(result.returncode != 0 and
            (b"Malformed" in result.stderr or b"unsupported MQTT durable document version" in result.stderr),
            "legacy_did_not_reject_document16")
    require(state_hashes(candidate) == before, "legacy_rejection_changed_state")
    return before


def binary_metadata(path, expected_product):
    result = subprocess.run([str(path), "--version"], capture_output=True, timeout=5)
    text = result.stdout.decode("utf-8").strip()
    require(result.returncode == 0 and text.startswith(expected_product + " ")
            and len(text) <= 100 and "\n" not in text, "binary_version")
    return {"version": text, "sha256": digest(path)}


def parser():
    result = argparse.ArgumentParser(description=__doc__)
    for name in ("broker", "admin", "legacy-broker", "report"):
        result.add_argument("--" + name, type=Path, required=True)
    return result


def main():
    arguments = parser().parse_args()
    if arguments.report.exists():
        raise SystemExit("new report path required; previous evidence is preserved")
    require(os.name == "posix", "linux_wsl_required")
    binaries = [arguments.broker.resolve(strict=True), arguments.admin.resolve(strict=True),
                arguments.legacy_broker.resolve(strict=True)]
    require(all(path.is_file() and os.access(path, os.X_OK) for path in binaries), "executable_inputs")
    report = {"scope": "owned Linux loopback resources; no power-loss claim or operator data",
              "harness_sha256": digest(Path(__file__)), "cases": []}
    began, stage = time.monotonic(), "binary_identity"
    try:
        report["binaries"] = dict(zip(("broker", "admin", "legacy_broker"),
            [binary_metadata(binary, "xmqr-admin" if index == 1 else "mqtt-broker")
             for index, binary in enumerate(binaries)]))
        with tempfile.TemporaryDirectory(prefix="xmqr-a21-v16-", dir="/var/tmp") as temporary:
            root = Path(temporary)
            stage = "native_v16_seed_and_active_lock"
            native = seed(binaries[0], binaries[1], root, "native")
            check_document(document(native), 16, 31)
            report["cases"].append({"case": stage, "status": "PASS"})
            stage = "native_backup_verify_restore_bytes"
            backup, restored, hashes = backup_restore(binaries[1], native, root, "native")
            check_document(document(restored), 16, 31)
            report["cases"].append({"case": stage, "status": "PASS", "state_sha256": hashes})
            stage = "legacy_refuses_v16_without_state_change"
            legacy_rejects(binaries[2], restored, root)
            report["cases"].append({"case": stage, "status": "PASS"})
            stage = "native_mqtt_after_restore"
            values = mqtt_after_restore(binaries[0], restored)
            report["cases"].append({"case": stage, "status": "PASS", **values})
            stage = "negative_lock_destination_corruption_symlink"
            negatives(binaries[1], native, backup, root)
            report["cases"].append({"case": stage, "status": "PASS"})
            stage = "real_legacy_v15_seed"
            legacy = seed(binaries[2], binaries[1], root, "legacy")
            check_document(document(legacy), 15, None)
            report["cases"].append({"case": stage, "status": "PASS"})
            stage = "legacy_backup_verify_restore_bytes"
            _, migrated, legacy_hashes = backup_restore(binaries[1], legacy, root, "legacy")
            report["cases"].append({"case": stage, "status": "PASS", "state_sha256": legacy_hashes})
            stage = "legacy_to_v16_without_invented_original"
            migration = OwnedBroker(binaries[0], migrated)
            try:
                migration.stop()
            finally:
                migration.cleanup()
            check_document(document(migrated), 16, None)
            require(state_hashes(legacy) == legacy_hashes, "legacy_fixture_changed")
            report["cases"].append({"case": stage, "status": "PASS"})
            stage = "migrated_mqtt_after_restore"
            values = mqtt_after_restore(binaries[0], migrated)
            require(state_hashes(legacy) == legacy_hashes, "legacy_fixture_changed")
            report["cases"].append({"case": stage, "status": "PASS", **values})
        report["status"] = "PASS"
    except Exception as failure:
        report["status"] = "FAIL"
        report["cases"].append({"case": stage, "status": "FAIL", "error_type": type(failure).__name__,
                                "reason": str(failure) if isinstance(failure, AssertionError)
                                else "bounded_private_fixture_error"})
    report["elapsed_seconds"] = round(time.monotonic() - began, 4)
    arguments.report.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(arguments.report, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "w", encoding="utf-8") as output:
        json.dump(report, output, indent=2)
        output.write("\n")
    print(json.dumps(report), flush=True)
    if report["status"] != "PASS":
        raise SystemExit(1)
    print("A21_CORE_V16_EXPIRY_PASS", flush=True)


class ContractTests(unittest.TestCase):
    @staticmethod
    def snapshot(value=b"version = 16\n[state]\n", *, duplicate=False, trailing=False):
        key = b"mqtt311.state.v1"
        entry = struct.pack("<HI", len(key), len(value)) + key + value
        data = struct.pack("<8sHQI", b"MQSNAP01", 1, 1, 2 if duplicate else 1) + entry
        if duplicate:
            data += entry
        if trailing:
            data += b"x"
        return data + struct.pack("<I", zlib.crc32(data))

    def test_snapshot_is_structured_and_crc_checked(self):
        parsed, sequence = snapshot_document(self.snapshot())
        self.assertEqual((parsed["version"], sequence), (16, 1))
        for damaged in (self.snapshot()[:-1], b"x" * 26, self.snapshot(duplicate=True),
                        self.snapshot(trailing=True), b"x" * (MAX_SNAPSHOT + 1)):
            with self.assertRaises(AssertionError):
                snapshot_document(damaged)

    def test_snapshot_version_string_in_payload_is_not_an_envelope(self):
        with self.assertRaises((AssertionError, tomllib.TOMLDecodeError)):
            snapshot_document(self.snapshot(b'payload = "version = 16"\n'))

    def test_snapshot_payload_hex_matches_real_persisted_representation(self):
        # Literal expected TOML, not output generated by this fixture's oracle.
        # The real frozen v15 writer was separately observed to serialize str.
        data = b'''version = 15
[state.retained."a21/retained"]
topic = "a21/retained"
payload = "6f776e65642d72657461696e6564"
retain = true
expiry_at_unix_ms = 20000
[state.sessions."a21-offline"]
offline = [{ topic = "a21/offline", payload = "6f776e65642d6f66666c696e65", qos = 1, expiry_at_unix_ms = 20000 }]
[state.pending_wills."a21-pending"]
delay_seconds = 120
message_expiry_interval_seconds = 37
[state.pending_wills."a21-pending".message]
topic = "a21/will"
payload = "6f776e65642d77696c6c"
'''
        envelope, _ = snapshot_document(self.snapshot(data))
        check_document(envelope, 15, None)
        retained = envelope["state"]["retained"][RETAINED.decode()]
        for invalid in ([111, 119], "0", "zz", " 6f", "+6f", "6F", "00",
                        "00" * 4097, None):
            retained["payload"] = invalid
            with self.assertRaises(AssertionError):
                check_document(envelope, 15, None)

    def test_mqtt_expiry_parser_checks_lengths_and_duplicate_properties(self):
        body = field(OFFLINE) + b"\x00\x07\x05\x02\x00\x00\x00\x1fowned"
        self.assertEqual(publication_fields(0x32, body), (OFFLINE, b"owned", 1, b"\x00\x07", 31))
        for invalid in (body[:2], body[:3], field(OFFLINE) + b"\x00\x07\x05\x02\x00"):
            with self.assertRaises(AssertionError):
                publication_fields(0x32, invalid)
        properties = b"\x02\x00\x00\x00\x1f" * 2
        with self.assertRaises(AssertionError):
            publication_fields(0x32, field(OFFLINE) + b"\x00\x07" + bytes([len(properties)]) + properties)

    def test_variable_integer_has_finite_size(self):
        for value in (0, 127, 128, 16_384, 268_435_455):
            encoded = varint(value)
            self.assertEqual(read_varint(encoded, 0), (value, len(encoded)))
        for invalid in (b"", b"\x80", b"\xff\xff\xff\xff\x01"):
            with self.assertRaises(AssertionError):
                read_varint(invalid, 0)

    def test_cli_requires_all_explicit_inputs(self):
        with self.assertRaises(SystemExit):
            parser().parse_args([])
        parsed = parser().parse_args(["--broker", "one", "--admin", "two",
                                      "--legacy-broker", "three", "--report", "new.json"])
        self.assertEqual(parsed.legacy_broker, Path("three"))

    def test_manifest_hash_paths_and_extra_files_are_not_trusted(self):
        with tempfile.TemporaryDirectory(prefix="xmqr-a21-contract-", dir="/var/tmp") as temporary:
            root = Path(temporary)
            root.chmod(0o700)
            for name in STATE_FILES:
                (root / name).write_bytes(b"owned-fixture")
                (root / name).chmod(0o600)
            header = "format_version = 1\nstate_format_version = 1\nstore_sequence = 1\n"
            entries = "".join(f'[[files]]\nname = "{name}"\nlength = 13\n'
                              f'sha256 = "{digest(root / name)}"\n' for name in STATE_FILES)
            manifest = root / "manifest.toml"
            manifest.write_text(header + entries, encoding="utf-8")
            manifest.chmod(0o600)
            self.assertEqual(check_manifest(root), state_hashes(root))
            for changed in (entries.replace("length = 13", "length = 14", 1),
                            entries.replace('name = "state.snapshot"', 'name = "../state.snapshot"', 1),
                            entries.replace(digest(root / "state.wal"), "0" * 64, 1)):
                manifest.write_text(header + changed, encoding="utf-8")
                with self.assertRaises(AssertionError):
                    check_manifest(root)
            manifest.write_text(header + entries, encoding="utf-8")
            (root / "unexpected").write_bytes(b"owned")
            with self.assertRaises(AssertionError):
                check_manifest(root)

    def test_native_and_legacy_interval_claims_are_distinct(self):
        def envelope(version, original):
            retained = dict(topic=RETAINED.decode(), payload="6f776e65642d72657461696e6564",
                            retain=True, expiry_at_unix_ms=20_000)
            offline = dict(topic=OFFLINE.decode(), payload="6f776e65642d6f66666c696e65",
                           qos=1, expiry_at_unix_ms=20_000)
            if original is not None:
                for message in (retained, offline):
                    message["expiry_interval_seconds"] = original
            state = dict(retained={RETAINED.decode(): retained},
                         sessions={"a21-offline": dict(offline=[offline])},
                         pending_wills={"a21-pending": dict(delay_seconds=120,
                            message_expiry_interval_seconds=37,
                            message=dict(topic=WILL.decode(), payload="6f776e65642d77696c6c"))})
            if version == 16:
                state["expiry_clock_unix_ms"] = 10_000
            return dict(version=version, state=state)
        check_document(envelope(16, 31), 16, 31)
        check_document(envelope(15, None), 15, None)
        check_document(envelope(16, None), 16, None)
        for invalid in (envelope(16, 31), envelope(16, None)):
            with self.assertRaises(AssertionError):
                check_document(invalid, 15, None)
        invalid = envelope(16, 31)
        invalid["state"].pop("expiry_clock_unix_ms")
        with self.assertRaises(AssertionError):
            check_document(invalid, 16, 31)

    def test_broker_environment_does_not_inherit_operator_mqtt_paths(self):
        from unittest.mock import patch
        with patch.dict(os.environ, {"MQTT_CONFIG": "operator-config",
                                     "MQTT_SERVER_KEY": "operator-key",
                                     "MQTT_ADMIN_SOCKET": "operator-socket"}):
            isolated = environment(Path("owned-state"), 12345)
        self.assertNotIn("MQTT_CONFIG", isolated)
        self.assertNotIn("MQTT_SERVER_KEY", isolated)
        self.assertNotIn("MQTT_ADMIN_SOCKET", isolated)
        self.assertEqual(isolated["MQTT_STATE_DIR"], "owned-state")
        self.assertEqual(isolated["MQTT_BIND"], "127.0.0.1:12345")


if __name__ == "__main__":
    main()

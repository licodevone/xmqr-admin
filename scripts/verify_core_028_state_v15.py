#!/usr/bin/env python3
"""A19: offline admin backup/restore of a pending MQTT 5 Will in state v15."""
import os
import shutil
import socket
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

from verify_core_023_state_v10 import BROKER, ADMIN, admin, free_port, start, stop

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "xmqr/scripts"))
from verify_mqtt5_sessions import packet  # noqa: E402


def pending_will(port):
    stream = socket.create_connection(("127.0.0.1", port), timeout=4)
    stream.settimeout(4)
    identity = b"a19-pending"
    topic = b"a19/will"
    properties = (b"\x18\x00\x00\x00\x0a\x02\x00\x00\x00\x02"
                  b"\x26\x00\x01a\x00\x011")
    body = (b"\x00\x04MQTT\x05\x2e\x00\x1e"
            b"\x05\x11\xff\xff\xff\xff"
            + len(identity).to_bytes(2, "big") + identity
            + bytes([len(properties)]) + properties
            + len(topic).to_bytes(2, "big") + topic
            + b"\x00\x03bye")
    stream.sendall(bytes([0x10, len(body)]) + body)
    header, ack = packet(stream)
    assert header == 0x20 and ack[1] == 0, (header, ack)
    return stream


def main():
    assert BROKER.is_file() and ADMIN.is_file(), "build Linux broker/admin first"
    project = Path(__file__).resolve().parents[1]
    version = tomllib.loads((project / "Cargo.toml").read_text())["package"]["version"]
    assert subprocess.check_output([str(ADMIN), "--version"]).strip() == f"xmqr-admin {version}".encode()
    with tempfile.TemporaryDirectory(prefix="xmqr-a19-core-") as directory:
        root = Path(directory)
        state, backup, restored, corrupt = (root / name for name in
                                            ("state", "backup", "restored", "corrupt"))
        state.mkdir(mode=0o700)
        port = free_port()
        broker = start(port, str(state))
        source = None
        try:
            source = pending_will(port)
            admin("state", "backup", "--state-dir", state,
                  "--destination", backup, success=False)
            assert not backup.exists()
        finally:
            stop(broker)
            if source is not None:
                source.close()
        snapshot = (state / "state.snapshot").read_bytes()
        assert b"version = 15" in snapshot, "v15 document missing"
        assert b"message_expiry_interval_seconds = 2" in snapshot, "pending Will expiry missing"
        assert b"a19-pending" in snapshot and b"a19/will" in snapshot
        old_admin = os.environ.get("XMQR_ADMIN_OLD")
        if old_admin:
            copied = subprocess.run([
                old_admin, "state", "backup", "--state-dir", str(state),
                "--destination", str(root / "old-backup"),
            ], capture_output=True, timeout=12)
            assert copied.returncode == 0, copied.stderr
            for command in [
                ["state", "verify", "--backup", str(root / "old-backup")],
                ["state", "restore", "--backup", str(root / "old-backup"),
                 "--state-dir", str(root / "old-restored")],
            ]:
                copied = subprocess.run([old_admin, *command], capture_output=True, timeout=12)
                assert copied.returncode == 0, copied.stderr
            assert b"message_expiry_interval_seconds = 2" in (
                root / "old-restored" / "state.snapshot").read_bytes()
        admin("state", "backup", "--state-dir", state, "--destination", backup)
        admin("state", "verify", "--backup", backup)
        admin("state", "restore", "--backup", backup, "--state-dir", restored)
        assert b"message_expiry_interval_seconds = 2" in (restored / "state.snapshot").read_bytes()
        assert not (restored / ".mqtt-broker-profile").exists()
        shutil.copy2(state / ".mqtt-broker-profile", restored / ".mqtt-broker-profile")
        old_broker = os.environ.get("XMQR_BROKER_OLD")
        if old_broker:
            env = dict(os.environ, MQTT_MODE="open-lab", MQTT_STATE_DIR=str(restored),
                       MQTT_BIND=f"127.0.0.1:{port}")
            refused = subprocess.run([old_broker], env=env, capture_output=True, timeout=4)
            assert refused.returncode != 0 and b"Malformed" in refused.stderr, refused.stderr
        broker = start(port, str(restored))
        stop(broker)
        shutil.copytree(backup, corrupt)
        damaged = bytearray((corrupt / "state.snapshot").read_bytes())
        damaged[-1] ^= 1
        (corrupt / "state.snapshot").write_bytes(damaged)
        admin("state", "verify", "--backup", corrupt, success=False)
    print("A19_V15_PASS pending Will expiry, backup/verify/restore, lock/tamper/profile"
          + (", old admin raw backup" if old_admin else ", old admin not supplied")
          + (", old broker rejected" if old_broker else ", old broker not supplied"))


if __name__ == "__main__":
    main()

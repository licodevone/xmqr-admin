#!/usr/bin/env python3
"""A12: offline admin backup/restore of disposable broker 0.23 v10 state."""
import os
import shutil
import socket
import subprocess
import sys
import tempfile
from pathlib import Path

BROKER_ROOT = Path(__file__).resolve().parents[2] / "xmqr"
sys.path.insert(0, str(BROKER_ROOT / "scripts"))
from verify_mqtt5_sessions import BROKER, packet, start, stop  # noqa: E402

ADMIN = Path(__file__).resolve().parents[1] / "target/debug/xmqr-admin"
TOPIC = b"a12/retained"


def free_port():
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        return probe.getsockname()[1]


def admin(*args, success=True):
    result = subprocess.run([str(ADMIN), *map(str, args)], capture_output=True, timeout=12)
    assert (result.returncode == 0) == success, result.stderr
    return result


def retained(port):
    with socket.create_connection(("127.0.0.1", port), timeout=4) as stream:
        stream.settimeout(4)
        identity = b"a12-pub"
        connect = (b"\x00\x04MQTT\x05\x02\x00\x1e\x00"
                   + len(identity).to_bytes(2, "big") + identity)
        stream.sendall(bytes([0x10, len(connect)]) + connect)
        assert packet(stream)[0] == 0x20
        properties = b"\x01\x01\x03\x00\x0atext/plain"
        body = (len(TOPIC).to_bytes(2, "big") + TOPIC
                + bytes([len(properties)]) + properties + b"v10")
        stream.sendall(bytes([0x31, len(body)]) + body)
        stream.sendall(b"\xe0\x00")


def read_restored(port):
    with socket.create_connection(("127.0.0.1", port), timeout=4) as stream:
        stream.settimeout(4)
        identity = b"a12-sub"
        connect = (b"\x00\x04MQTT\x05\x02\x00\x1e\x00"
                   + len(identity).to_bytes(2, "big") + identity)
        stream.sendall(bytes([0x10, len(connect)]) + connect)
        assert packet(stream)[0] == 0x20
        subscribe = b"\x00\x01\x00" + len(TOPIC).to_bytes(2, "big") + TOPIC + b"\x00"
        stream.sendall(bytes([0x82, len(subscribe)]) + subscribe)
        assert packet(stream)[0] == 0x90
        header, body = packet(stream)
        assert header == 0x31 and body.endswith(b"v10"), (header, body)
        assert b"text/plain" in body, body


def main():
    assert BROKER.is_file() and ADMIN.is_file(), "build Linux broker/admin first"
    with tempfile.TemporaryDirectory(prefix="xmqr-a12-core-") as directory:
        root = Path(directory)
        state, backup, restored, corrupt = (root / name for name in
                                             ("state", "backup", "restored", "corrupt"))
        state.mkdir(mode=0o700)
        port = free_port()
        broker = start(port, str(state))
        try:
            retained(port)
            admin("state", "backup", "--state-dir", state,
                  "--destination", backup, success=False)
            assert not backup.exists()
        finally:
            stop(broker)
        admin("state", "backup", "--state-dir", state, "--destination", backup)
        admin("state", "verify", "--backup", backup)
        admin("state", "restore", "--backup", backup, "--state-dir", restored)
        try:
            broker = start(port, str(restored))
        except AssertionError:
            env = dict(os.environ, MQTT_MODE="open-lab", MQTT_STATE_DIR=str(restored),
                       MQTT_BIND=f"127.0.0.1:{port}")
            result = subprocess.run([str(BROKER)], env=env, capture_output=True, timeout=4)
            assert result.returncode != 0 and b"unmarked MQTT_STATE_DIR" in result.stderr, result.stderr
            marker = state / ".mqtt-broker-profile"
            assert marker.read_text().strip() == "open-lab"
            shutil.copy2(marker, restored / marker.name)
            broker = start(port, str(restored))
        try:
            read_restored(port)
        finally:
            stop(broker)
        shutil.copytree(backup, corrupt)
        snapshot = corrupt / "state.snapshot"
        damaged = bytearray(snapshot.read_bytes())
        damaged[-1] ^= 1
        snapshot.write_bytes(damaged)
        admin("state", "verify", "--backup", corrupt, success=False)
    print("PASS: A12 broker 0.23 v10 offline backup/verify/restore and tamper/active-lock negatives")


if __name__ == "__main__":
    main()

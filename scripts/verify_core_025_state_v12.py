#!/usr/bin/env python3
"""A15: offline admin backup/restore of disposable broker 0.25 v12 state."""
import os
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

from verify_core_023_state_v10 import BROKER, ADMIN, admin, free_port, start, stop

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "xmqr/scripts"))
from verify_mqtt5_subscription_identifier import (  # noqa: E402
    barrier, connect, publish, receive, subscribe,
)


def main():
    assert BROKER.is_file() and ADMIN.is_file(), "build Linux broker/admin first"
    project = Path(__file__).resolve().parents[1]
    version = tomllib.loads((project / "Cargo.toml").read_text())["package"]["version"]
    assert subprocess.check_output([str(ADMIN), "--version"]).strip() == f"xmqr-admin {version}".encode()
    with tempfile.TemporaryDirectory(prefix="xmqr-a15-core-") as directory:
        root = Path(directory)
        state, backup, restored, corrupt = (root / name for name in
                                             ("state", "backup", "restored", "corrupt"))
        state.mkdir(mode=0o700)
        port = free_port()
        broker = start(port, str(state))
        try:
            subscriber, present = connect(port, b"a15-durable", persistent=True)
            assert not present
            subscribe(subscriber, 17)
            with connect(port, b"a15-publisher")[0] as publisher:
                publish(publisher, b"before-backup", 1, 41)
            receive(subscriber, b"before-backup", 1, [17, 17])
            barrier(subscriber)
            subscriber.sendall(b"\xe0\x00")
            subscriber.close()
            admin("state", "backup", "--state-dir", state,
                  "--destination", backup, success=False)
            assert not backup.exists()
        finally:
            stop(broker)
        snapshot = state / "state.snapshot"
        assert b"subscription_identifiers" in snapshot.read_bytes(), "v12 identifiers missing"
        admin("state", "backup", "--state-dir", state, "--destination", backup)
        admin("state", "verify", "--backup", backup)
        admin("state", "restore", "--backup", backup, "--state-dir", restored)
        assert not (restored / ".mqtt-broker-profile").exists()
        env = dict(os.environ, MQTT_MODE="open-lab", MQTT_STATE_DIR=str(restored),
                   MQTT_BIND=f"127.0.0.1:{port}")
        refused = subprocess.run([str(BROKER)], env=env, capture_output=True, timeout=4)
        assert refused.returncode != 0 and b"unmarked MQTT_STATE_DIR" in refused.stderr
        marker = state / ".mqtt-broker-profile"
        assert marker.read_text().strip() == "open-lab"
        shutil.copy2(marker, restored / marker.name)  # explicit disposable lab selection
        broker = start(port, str(restored))
        try:
            subscriber, present = connect(port, b"a15-durable", clean=False, persistent=True)
            assert present
            with connect(port, b"a15-after-restore")[0] as publisher:
                publish(publisher, b"after-restore", 1, 42)
            receive(subscriber, b"after-restore", 1, [17, 17])
            barrier(subscriber)
            subscriber.sendall(b"\xe0\x00")
            subscriber.close()
        finally:
            stop(broker)
        shutil.copytree(backup, corrupt)
        damaged = bytearray((corrupt / "state.snapshot").read_bytes())
        damaged[-1] ^= 1
        (corrupt / "state.snapshot").write_bytes(damaged)
        admin("state", "verify", "--backup", corrupt, success=False)
    print("A15_V12_PASS offline subscription IDs backup/restore, lock/tamper/profile negatives")


if __name__ == "__main__":
    main()

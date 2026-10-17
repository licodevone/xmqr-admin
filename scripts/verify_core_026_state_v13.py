#!/usr/bin/env python3
"""A16: offline admin backup/restore of durable MQTT 5 shared state v13."""
import os
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

from verify_core_023_state_v10 import BROKER, ADMIN, admin, free_port, start, stop

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "xmqr/scripts"))
from verify_mqtt5_shared_durable import TOPIC, SHARED, publish, receive, subscribe  # noqa: E402
from verify_mqtt5_subscribe_options import connect  # noqa: E402
from verify_mqtt5_subscription_identifier import barrier  # noqa: E402


def main():
    assert BROKER.is_file() and ADMIN.is_file(), "build Linux broker/admin first"
    project = Path(__file__).resolve().parents[1]
    version = tomllib.loads((project / "Cargo.toml").read_text())["package"]["version"]
    assert subprocess.check_output([str(ADMIN), "--version"]).strip() == f"xmqr-admin {version}".encode()
    with tempfile.TemporaryDirectory(prefix="xmqr-a16-core-") as directory:
        root = Path(directory)
        state, backup, restored, corrupt = (root / name for name in
                                            ("state", "backup", "restored", "corrupt"))
        state.mkdir(mode=0o700)
        port = free_port()
        broker = start(port, str(state))
        try:
            subscriber, present = connect(port, b"a16-durable", persistent=True)
            assert not present
            subscribe(subscriber, SHARED, qos=2, identifier=17)
            subscriber.sendall(b"\xe0\x00")
            subscriber.close()
            with connect(port, b"a16-publisher")[0] as publisher:
                publish(publisher, b"durable-v13", qos=2, packet_id=31)
            admin("state", "backup", "--state-dir", state,
                  "--destination", backup, success=False)
            assert not backup.exists()
        finally:
            stop(broker)
        snapshot = (state / "state.snapshot").read_bytes()
        assert b"shared_filters" in snapshot and SHARED in snapshot, "v13 shared membership missing"
        assert TOPIC in snapshot and b"subscription_identifiers" in snapshot
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
        shutil.copy2(marker, restored / marker.name)
        broker = start(port, str(restored))
        try:
            subscriber, present = connect(port, b"a16-durable", clean=False, persistent=True)
            assert present
            receive(subscriber, b"durable-v13", 2, 17)
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
    print("A16_V13_PASS shared membership, QoS2 offline, backup/restore, lock/tamper/profile")


if __name__ == "__main__":
    main()

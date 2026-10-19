#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Owned A25 browser fixture; token and controls stay in a private WSL directory."""
import argparse
import json
from pathlib import Path
import tempfile
import time
from verify_system_status import MutableMonitor, digest, owned_server

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--admin', type=Path, required=True)
parser.add_argument('--handoff', type=Path, required=True)
parser.add_argument('--stop', type=Path, required=True)
args = parser.parse_args()
if args.handoff.exists() or args.stop.exists():
    parser.error('use new handoff and stop paths')
admin = args.admin.resolve(strict=True)
with tempfile.TemporaryDirectory(prefix='xmqr-admin-a25-browser-', dir='/var/tmp') as directory:
    root = Path(directory)
    root.chmod(0o700)
    with MutableMonitor(root / 'monitor.control') as monitor:
        with owned_server(admin, root, monitor) as (board, child):
            with args.handoff.open('x') as handoff:
                json.dump(dict(url=f'http://127.0.0.1:{board.number}',
                               token_file=str(root / 'access.token'),
                               monitor_control=str(root / 'monitor.control'),
                               client_control=str(root / 'client.control'),
                               hashes=dict(admin_elf_sha256=digest(admin),
                                           aggregate_fixture_sha256=digest(Path(__file__).with_name('verify_system_status.py')))), handoff)
            deadline = time.monotonic() + 300
            while time.monotonic() < deadline and not args.stop.exists():
                assert child.poll() is None, 'owned browser server exited'
                time.sleep(.2)

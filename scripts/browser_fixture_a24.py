#!/usr/bin/env python3
"""Servidor próprio para QA visual de contas web, sem dados do operador."""
import argparse
import json
from pathlib import Path
import tempfile
import time
from verify_web_auth import server

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--admin', type=Path, required=True)
parser.add_argument('--handoff', type=Path, required=True)
parser.add_argument('--stop', type=Path, required=True)
args = parser.parse_args()
if args.handoff.exists() or args.stop.exists():
    parser.error('use arquivos novos')
with tempfile.TemporaryDirectory(prefix='xmqr-admin-a24-browser-', dir='/var/tmp') as directory:
    root = Path(directory)
    root.chmod(0o700)
    with server(args.admin, root, root / 'users.json') as (board, process):
        with args.handoff.open('x') as stream:
            json.dump(dict(url=f'http://127.0.0.1:{board.number}', token_file=str(root / 'access.token')), stream)
        deadline = time.monotonic() + 240
        while time.monotonic() < deadline and not args.stop.exists():
            assert process.poll() is None
            time.sleep(.2)

#!/usr/bin/env python3
"""Servidor próprio da verificação visual A23; nenhum estado do operador."""
import argparse
import json
from pathlib import Path
import tempfile
import time
from verify_mqtt_board import FAKE, private, server

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--admin', type=Path, required=True)
parser.add_argument('--handoff', type=Path, required=True)
parser.add_argument('--stop', type=Path, required=True)
args = parser.parse_args()
if args.handoff.exists() or args.stop.exists():
    parser.error('use arquivos novos para preservar as evidencias')
with tempfile.TemporaryDirectory(prefix='xmqr-admin-a23-browser-', dir='/var/tmp') as directory:
    root = Path(directory)
    root.chmod(0o700)
    private(root / 'fake-client', FAKE)
    (root / 'fake-client').chmod(0o700)
    for name in ['sub.toml', 'pub.toml']:
        private(root / name, 'version=1\n[connection]\nhost="127.0.0.1"\n')
    with server(args.admin, root) as (board, process):
        with args.handoff.open('x') as stream:
            json.dump(dict(url=board.origin, token_file=str(root / 'access.token')), stream)
        deadline = time.monotonic() + 180
        while time.monotonic() < deadline and not args.stop.exists():
            assert process.poll() is None
            time.sleep(.2)

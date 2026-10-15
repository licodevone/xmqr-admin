# SPDX-License-Identifier: MIT
"""Cross-process broker/admin check using private temporary resources."""
import argparse
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time


def private(path, text):
    path.write_text(text, encoding='utf-8')
    path.chmod(0o600)


def run(admin, *args, success=True):
    result = subprocess.run([str(admin), 'channel', *map(str, args)],
                            capture_output=True, text=True, timeout=8)
    if success != (result.returncode == 0):
        raise AssertionError(f'admin exit {result.returncode}: {result.stderr}')
    if not result.stdout.strip():
        raise AssertionError(f'admin returned no JSON: {result.stderr}')
    return [json.loads(line) for line in result.stdout.splitlines()]


def start(broker, env, socket_path):
    process = subprocess.Popen([str(broker)], env=env,
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    deadline = time.monotonic() + 8
    while not socket_path.exists():
        if process.poll() is not None or time.monotonic() > deadline:
            raise AssertionError('broker socket did not start')
        time.sleep(.02)
    return process


def stop(process, socket_path):
    process.terminate()
    output, _ = process.communicate(timeout=8)
    assert process.returncode == 0 and not socket_path.exists()
    assert b'a' * 64 not in output and b'b' * 64 not in output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--broker', type=Path, required=True)
    parser.add_argument('--admin', type=Path, required=True)
    args = parser.parse_args()
    broker = args.broker.resolve()
    admin = args.admin.resolve()
    with tempfile.TemporaryDirectory(prefix='xmqr-a08-') as temporary:
        root = Path(temporary)
        root.chmod(0o700)
        state = root / 'state'
        state.mkdir(mode=0o700)
        bundle = root / 'bundle.toml'
        fixture = ("version=1\n[[users]]\nusername='fixture'\n"
                   "password_hash='$argon2id$v=19$m=19456,t=2,p=1$"
                   "cHVibGljLXRlc3Qtc2FsdA$"
                   "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA'\n"
                   "cert_sha256='sha256:" + '00' * 32 + "'\n")
        private(bundle, fixture)
        view = root / 'view.token'
        operator = root / 'operator.token'
        private(view, 'a' * 64 + '\n')
        private(operator, 'b' * 64 + '\n')
        endpoint = root / 'admin.sock'
        with socket.socket() as reserve:
            reserve.bind(('127.0.0.1', 0))
            port = reserve.getsockname()[1]
        env = {key: value for key, value in os.environ.items() if not key.startswith('MQTT_')}
        env.update(MQTT_MODE='acl-lab', MQTT_BIND=f'127.0.0.1:{port}',
                   MQTT_STATE_DIR=str(state), MQTT_SECURITY_BUNDLE=str(bundle),
                   MQTT_ADMIN_SOCKET=str(endpoint),
                   MQTT_ADMIN_VIEW_TOKEN_FILE=str(view),
                   MQTT_ADMIN_OPERATOR_TOKEN_FILE=str(operator))
        flags = ('--socket', endpoint, '--token-file')
        process = start(broker, env, endpoint)
        try:
            assert run(admin, 'status', *flags, view)[0]['security_generation'] == 1
            bad_id = subprocess.run([str(admin), 'channel', 'receipt', *map(str, flags),
                                     str(view), '--id', 'invalid'], capture_output=True, timeout=8)
            assert bad_id.returncode != 0
            public = root / 'public.token'
            private(public, 'b' * 64 + '\n')
            public.chmod(0o644)
            refused = subprocess.run([str(admin), 'channel', 'status', '--socket', str(endpoint),
                                      '--token-file', str(public)], capture_output=True, timeout=8)
            assert refused.returncode != 0
            link = root / 'link.token'
            link.symlink_to(operator)
            refused = subprocess.run([str(admin), 'channel', 'status', '--socket', str(endpoint),
                                      '--token-file', str(link)], capture_output=True, timeout=8)
            assert refused.returncode != 0
            request_id = 'abcdef0123456789abcdef0123456789'
            reload_flags = ('--file', bundle, '--expected-generation', '1',
                            '--request-id', request_id)
            denied = run(admin, 'reload', *flags, view, *reload_flags, success=False)
            assert denied[-1]['error'] == 'forbidden'
            accepted = run(admin, 'reload', *flags, operator, *reload_flags)
            assert accepted[-1]['applied_generation'] == 2
            assert accepted[-1]['replayed'] is False
            assert accepted[0]['request_id'] == request_id
            assert run(admin, 'status', *flags, view)[0]['security_generation'] == 2
            receipt = run(admin, 'receipt', *flags, view, '--id', request_id)[0]
            assert receipt['outcome'] == 'committed'
            assert receipt['timestamp_unix_ms'] > 0
            assert run(admin, 'reload', *flags, operator, *reload_flags)[-1]['replayed'] is True
            private(bundle, fixture + '# changed\n')
            conflict = run(admin, 'reload', *flags, operator, *reload_flags, success=False)
            assert conflict[-1]['error'] == 'request_conflict'
            private(bundle, fixture)
            stop(process, endpoint)
            process = start(broker, env, endpoint)
            assert run(admin, 'reload', *flags, operator, *reload_flags)[-1]['replayed'] is True
            assert run(admin, 'receipt', *flags, view, '--id', request_id)[0]['outcome'] == 'committed'
            print('CHANNEL_CLI_PASS')
        finally:
            if process.poll() is None:
                stop(process, endpoint)


if __name__ == '__main__':
    main()

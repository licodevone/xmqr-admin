# SPDX-License-Identifier: MIT
"""Cross-process broker/admin check using private temporary resources."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import threading
import time
from urllib.request import urlopen


def private(path, text):
    path.write_text(text, encoding='utf-8')
    path.chmod(0o600)


def mqtt_retained(port, topic='lab/retained', client_id='admin-fixture'):
    def field(value):
        value = value.encode()
        return len(value).to_bytes(2, 'big') + value

    with socket.create_connection(('127.0.0.1', port), timeout=3) as connection:
        body = field('MQTT') + bytes([4, 0xc2, 0, 30]) + field(client_id)
        body += field('fixture') + field('fixture')
        connection.sendall(bytes([0x10, len(body)]) + body)
        assert connection.recv(4) == b'\x20\x02\x00\x00'
        body = field(topic) + b'payload-fixture'
        connection.sendall(bytes([0x31, len(body)]) + body)
        connection.sendall(b'\xe0\x00')


def mqtt_pending_qos2_with_will(port, client_id='active-device'):
    def field(value):
        value = value.encode()
        return len(value).to_bytes(2, 'big') + value

    connection = socket.create_connection(('127.0.0.1', port), timeout=3)
    body = field('MQTT') + bytes([4, 0xec, 0, 30]) + field(client_id)
    body += field('lab/will') + field('offline') + field('fixture') + field('fixture')
    connection.sendall(bytes([0x10, len(body)]) + body)
    assert connection.recv(4) == b'\x20\x02\x00\x00'
    body = field('lab/retained') + b'\x00\x07' + b'accepted'
    connection.sendall(bytes([0x34, len(body)]) + body)
    assert connection.recv(4) == b'\x50\x02\x00\x07'
    return connection


def mqtt_connect_result(port, client_id):
    def field(value):
        value = value.encode()
        return len(value).to_bytes(2, 'big') + value

    with socket.create_connection(('127.0.0.1', port), timeout=3) as connection:
        body = field('MQTT') + bytes([4, 0xc2, 0, 30]) + field(client_id)
        body += field('fixture') + field('fixture')
        connection.sendall(bytes([0x10, len(body)]) + body)
        return connection.recv(4)


def run(admin, *args, success=True):
    result = subprocess.run([str(admin), 'channel', *map(str, args)],
                            capture_output=True, text=True, timeout=8)
    if success != (result.returncode == 0):
        raise AssertionError(f'admin exit {result.returncode}: {result.stderr}')
    if not result.stdout.strip():
        raise AssertionError(f'admin returned no JSON: {result.stderr}')
    return [json.loads(line) for line in result.stdout.splitlines()]


def raw_admin(endpoint, token, operation, request_id, **fields):
    with socket.socket(socket.AF_UNIX) as connection:
        connection.settimeout(3)
        connection.connect(str(endpoint))
        packet = dict(version=1, operation=operation, request_id=request_id, token=token)
        packet.update(fields)
        connection.sendall(json.dumps(packet).encode() + b'\n')
        answer = bytearray()
        while True:
            chunk = connection.recv(1024)
            if not chunk:
                break
            answer.extend(chunk)
        return json.loads(answer)


def edit_bundle(admin, *args):
    result = subprocess.run([str(admin), 'bundle', *map(str, args)],
                            capture_output=True, text=True, timeout=8)
    if result.returncode != 0:
        raise AssertionError(f'bundle edit refused: {result.stderr}')


class OutputGuard:
    """Drain without retaining raw logs; scan synthetic tokens across chunks."""
    def __init__(self, stream):
        self.stream = stream
        self.leaked = False
        self.error = None
        self.thread = threading.Thread(target=self.consume, daemon=True)
        self.thread.start()

    def consume(self):
        carry = b''
        try:
            while chunk := self.stream.read(4096):
                combined = carry + chunk
                self.leaked |= b'a' * 64 in combined or b'b' * 64 in combined
                carry = combined[-63:]
        except Exception as error:
            self.error = type(error).__name__
        finally:
            try:
                self.stream.close()
            except Exception as error:
                self.error = type(error).__name__

    def finish(self, deadline):
        self.thread.join(max(0, deadline - time.monotonic()))
        assert not self.thread.is_alive(), 'owned log drain deadline exceeded'
        assert self.error is None, 'owned log drain read failure'
        assert not self.leaked, 'synthetic token appeared in broker logs'


def force_cleanup(process):
    """Only a failed fixture's already-owned child, never acceptance evidence."""
    if process.poll() is None:
        process.kill()
    process.wait(timeout=5)
    if process.channel_output is None:
        process.stdout.close()
    else:
        process.channel_output.thread.join(timeout=5)
        assert not process.channel_output.thread.is_alive(), 'owned failed-fixture drain still alive'


def anonymous_refusal(address, deadline):
    """A complete MQTT response proves the serving future has been polled."""
    with socket.create_connection(address, timeout=min(.2, max(.001, deadline - time.monotonic()))) as peer:
        peer.settimeout(min(.2, max(.001, deadline - time.monotonic())))
        # No username/password: acl-lab must refuse, without creating a session.
        peer.sendall(b'\x10\x1b\x00\x04MQTT\x04\x02\x00\x1e\x00\x0fadmin-readiness')
        response = b''
        while len(response) < 4:
            peer.settimeout(min(.2, max(.001, deadline - time.monotonic())))
            part = peer.recv(4 - len(response))
            assert part, 'owned startup MQTT response truncated'
            response += part
        assert response == b'\x20\x02\x00\x05', 'owned startup anonymous MQTT refusal invalid'
        assert time.monotonic() < deadline, 'owned startup MQTT response past deadline'


def start(broker, env, socket_path):
    deadline = time.monotonic() + 8
    process = subprocess.Popen([str(broker)], env=env,
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    process.channel_output = None
    try:
        process.channel_output = OutputGuard(process.stdout)
        host, port = env['MQTT_BIND'].rsplit(':', 1)
        assert host == '127.0.0.1' and 0 < int(port) < 65536
        while time.monotonic() < deadline:
            assert process.poll() is None, 'owned broker exited before readiness'
            if socket_path.exists():
                try:
                    anonymous_refusal((host, int(port)), deadline)
                except (ConnectionRefusedError, TimeoutError):
                    pass
                else:
                    assert process.poll() is None and time.monotonic() < deadline
                    return process
            time.sleep(min(.02, max(0, deadline - time.monotonic())))
        raise AssertionError('owned broker startup deadline exceeded')
    except BaseException as primary:
        try:
            force_cleanup(process)
        except Exception as cleanup:
            primary.add_note('owned startup cleanup failed: ' + type(cleanup).__name__)
        raise


def stop(process, socket_path):
    deadline = time.monotonic() + 8
    try:
        process.terminate()
        process.wait(timeout=max(.001, deadline - time.monotonic()))
        process.channel_output.finish(deadline)
        assert time.monotonic() < deadline, 'owned broker stop past deadline'
        assert process.returncode == 0 and not socket_path.exists(), \
            f'owned broker stop failed: exit={process.returncode}, socket_exists={socket_path.exists()}'
    except BaseException as primary:
        try:
            force_cleanup(process)
        except Exception as cleanup:
            primary.add_note('owned stop cleanup failed: ' + type(cleanup).__name__)
        raise


def finish_fixture(process, endpoint):
    primary = sys.exception()
    if process.poll() is None:
        try:
            stop(process, endpoint)
        except Exception as cleanup:
            if primary is None:
                raise
            primary.add_note('owned fixture cleanup failed: ' + type(cleanup).__name__)
    elif primary is None:
        assert process.returncode == 0 and not endpoint.exists(), 'owned fixture exited unexpectedly'
        process.channel_output.finish(time.monotonic() + 8)


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
                   "PJ/v6bCRvdNVr6dTfyfB5w$/P91B0CWkDauFZI2fj17hVdaAilPxom+AAkF39Nc9Xs'\n"
                   "cert_sha256='sha256:" + '00' * 32 + "'\n"
                   "publish=['lab/retained','lab/will']\n")
        private(bundle, fixture)
        view = root / 'view.token'
        operator = root / 'operator.token'
        private(view, 'a' * 64 + '\n')
        private(operator, 'b' * 64 + '\n')
        endpoint = root / 'admin.sock'
        with socket.socket() as reserve:
            reserve.bind(('127.0.0.1', 0))
            port = reserve.getsockname()[1]
        with socket.socket() as reserve:
            reserve.bind(('127.0.0.1', 0))
            monitor_port = reserve.getsockname()[1]
        env = {key: value for key, value in os.environ.items() if not key.startswith('MQTT_')}
        env.update(MQTT_MODE='acl-lab', MQTT_BIND=f'127.0.0.1:{port}',
                   MQTT_STATE_DIR=str(state), MQTT_SECURITY_BUNDLE=str(bundle),
                   MQTT_ADMIN_SOCKET=str(endpoint),
                   MQTT_ADMIN_VIEW_TOKEN_FILE=str(view),
                   MQTT_ADMIN_OPERATOR_TOKEN_FILE=str(operator),
                   MQTT_MONITOR_ENABLED='true',
                   MQTT_MONITOR_BIND=f'127.0.0.1:{monitor_port}')
        flags = ('--socket', endpoint, '--token-file')
        process = start(broker, env, endpoint)
        try:
            assert run(admin, 'status', *flags, view)[0]['security_generation'] == 1
            assert run(admin, 'sessions', *flags, operator)[0]['items'] == []
            assert run(admin, 'retained', *flags, operator, '--limit', '8')[0]['items'] == []
            assert run(admin, 'sessions', *flags, view, success=False)[0]['error'] == 'forbidden'
            mqtt_retained(port)
            deadline = time.monotonic() + 3
            while not run(admin, 'retained', *flags, operator)[0]['items']:
                if time.monotonic() > deadline:
                    raise AssertionError('retained PUBLISH not committed')
                time.sleep(.02)
            delete_id = '13579bdf02468ace13579bdf02468ace'
            delete_args = ('--topic', 'lab/retained', '--request-id', delete_id)
            assert run(admin, 'retained-delete', *flags, view, *delete_args, success=False)[-1]['error'] == 'forbidden'
            deleted = run(admin, 'retained-delete', *flags, operator, *delete_args)
            assert deleted[-1]['deleted'] and deleted[-1]['replayed'] is False
            assert run(admin, 'retained', *flags, operator)[0]['items'] == []
            assert run(admin, 'retained-delete', *flags, operator, *delete_args)[-1]['replayed'] is True
            assert run(admin, 'receipt', *flags, view, '--id', delete_id)[0]['operation'] == 'retained_delete'
            mqtt_retained(port)
            concurrent_id = 'aaaaaaaa00000000bbbbbbbb11111111'
            with ThreadPoolExecutor(max_workers=4) as pool:
                results = list(pool.map(lambda _: run(admin, 'retained-delete', *flags, operator,
                                      '--topic', 'lab/retained', '--request-id', concurrent_id)[-1],
                                      range(4)))
            assert sum(not item['replayed'] for item in results) == 1
            assert run(admin, 'retained', *flags, operator)[0]['items'] == []
            active = mqtt_pending_qos2_with_will(port)
            try:
                sessions = run(admin, 'sessions', *flags, operator)[0]['items']
                assert any(item['client_id'] == 'active-device' and item['inbound_qos2'] == 1
                           and item['connected'] for item in sessions)
                disconnect_id = '2468ace13579bdf02468ace13579bdf0'
                disconnect_args = ('--client-id', 'active-device', '--request-id', disconnect_id)
                assert run(admin, 'session-disconnect', *flags, view, *disconnect_args,
                           success=False)[-1]['error'] == 'forbidden'
                result = run(admin, 'session-disconnect', *flags, operator, *disconnect_args)
                assert result[-1]['disconnected'] and result[-1]['replayed'] is False
                assert active.recv(1) == b''
                sessions = run(admin, 'sessions', *flags, operator)[0]['items']
                assert any(item['client_id'] == 'active-device' and item['inbound_qos2'] == 1
                           and not item['connected'] for item in sessions)
                retained = run(admin, 'retained', *flags, operator)[0]['items']
                assert any(item['topic'] == 'lab/will' for item in retained)
                assert run(admin, 'session-disconnect', *flags, operator,
                           *disconnect_args)[-1]['replayed'] is True
                assert run(admin, 'receipt', *flags, view, '--id', disconnect_id)[0]['operation'] == 'session_disconnect'
            finally:
                active.close()
            assert run(admin, 'audit', *flags, view, success=False)[0]['error'] == 'forbidden'
            first_audit = run(admin, 'audit', *flags, operator, '--limit', '1')[0]
            assert len(first_audit['items']) == 1 and first_audit['next_cursor'] == '1'
            second_audit = run(admin, 'audit', *flags, operator, '--limit', '1',
                               '--cursor', first_audit['next_cursor'])[0]
            assert len(second_audit['items']) == 1
            assert 'payload-fixture' not in json.dumps(first_audit)
            assert 'fixture' not in json.dumps(first_audit)
            with urlopen(f'http://127.0.0.1:{monitor_port}/metrics', timeout=3) as response:
                metrics = response.read().decode()
            assert 'xmqr_admin_mutations_committed_total 3' in metrics
            assert 'xmqr_admin_receipts 3' in metrics
            assert 'xmqr_admin_mutations_replayed_total ' in metrics
            assert 'payload-fixture' not in metrics
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
            assert run(admin, 'retained-delete', *flags, operator, *delete_args)[-1]['replayed'] is True
            assert run(admin, 'session-disconnect', *flags, operator,
                       *disconnect_args)[-1]['replayed'] is True
            edit_bundle(admin, 'role', 'create', '--file', bundle, '--name', 'sender',
                        '--publish', 'lab/group', '--publish', 'lab/retained',
                        '--publish', 'lab/will')
            edit_bundle(admin, 'group', 'create', '--file', bundle, '--name', 'devices',
                        '--role', 'sender')
            edit_bundle(admin, 'user', 'set-membership', '--file', bundle,
                        '--username', 'fixture', '--group', 'devices')
            edit_bundle(admin, 'user', 'update-acl', '--file', bundle, '--username', 'fixture')
            role_id = '11111111222222223333333344444444'
            applied = run(admin, 'reload', *flags, operator, '--file', bundle,
                          '--expected-generation', '1', '--request-id', role_id)
            assert applied[-1]['applied_generation'] == 2
            mqtt_retained(port, topic='lab/group', client_id='granted-device')
            deadline = time.monotonic() + 3
            while not any(item['topic'] == 'lab/group' for item in
                          run(admin, 'retained', *flags, operator)[0]['items']):
                if time.monotonic() > deadline:
                    raise AssertionError('role/group grant not active')
                time.sleep(.02)
            mqtt_retained(port, topic='lab/denied', client_id='denied-device')
            time.sleep(.1)
            assert not any(item['topic'] == 'lab/denied' for item in
                           run(admin, 'retained', *flags, operator)[0]['items'])
            revoking = mqtt_pending_qos2_with_will(port, 'revocation-device')
            try:
                edit_bundle(admin, 'user', 'disable', '--file', bundle,
                            '--username', 'fixture')
                revoke_id = '55555555666666667777777788888888'
                applied = run(admin, 'reload', *flags, operator, '--file', bundle,
                              '--expected-generation', '2', '--request-id', revoke_id)
                assert applied[-1]['applied_generation'] == 3
                assert revoking.recv(1) == b''
                assert mqtt_connect_result(port, 'new-after-revoke') != b'\x20\x02\x00\x00'
                sessions = run(admin, 'sessions', *flags, operator)[0]['items']
                assert any(item['client_id'] == 'revocation-device'
                           and item['inbound_qos2'] == 1 and not item['connected']
                           for item in sessions)
            finally:
                revoking.close()
            stop(process, endpoint)
            process = start(broker, env, endpoint)
            assert mqtt_connect_result(port, 'new-after-restart') != b'\x20\x02\x00\x00'
            assert run(admin, 'receipt', *flags, view, '--id', revoke_id)[0]['outcome'] == 'committed'
            sessions = run(admin, 'sessions', *flags, operator)[0]['items']
            assert any(item['client_id'] == 'revocation-device' and item['inbound_qos2'] == 1
                       for item in sessions)
            existing_archive = root / 'existing.json'
            private(existing_archive, 'do-not-overwrite')
            refused = subprocess.run([str(admin), 'channel', 'audit-rotate', '--socket', str(endpoint),
                                      '--token-file', str(operator), '--archive', str(existing_archive)],
                                     capture_output=True, timeout=8)
            assert refused.returncode != 0 and existing_archive.read_text() == 'do-not-overwrite'
            assert run(admin, 'status', *flags, view)[0]['admin_epoch'] == 0
            archive = root / 'audit-epoch0.json'
            rotated = run(admin, 'audit-rotate', *flags, operator, '--archive', archive)
            rotate_id = rotated[0]['request_id']
            assert rotated[-1]['admin_epoch'] == 1 and rotated[-1]['replayed'] is False
            assert archive.stat().st_mode & 0o777 == 0o600
            verified = subprocess.run([str(admin), 'audit-verify', '--file', str(archive)],
                                      capture_output=True, text=True, timeout=8)
            assert verified.returncode == 0
            assert json.loads(verified.stdout)['sha256'] == rotated[0]['archive_sha256']
            assert run(admin, 'receipt', *flags, view, '--id', rotate_id)[0]['operation'] == 'audit_rotate'
            assert run(admin, 'status', *flags, view)[0]['admin_epoch'] == 1
            assert raw_admin(endpoint, 'a' * 64, 'audit_rotate',
                             '00000001aaaabbbbccccddddeeeeffff',
                             archive_sha256=rotated[0]['archive_sha256'],
                             archive_count=rotated[0]['archive_count'])['error'] == 'forbidden'
            assert raw_admin(endpoint, 'b' * 64, 'audit_rotate',
                             '00000001aaaabbbbccccddddeeeeffff',
                             archive_sha256=rotated[0]['archive_sha256'],
                             archive_count=rotated[0]['archive_count'])['error'] == 'archive_changed'
            stale = run(admin, 'retained-delete', *flags, operator, *delete_args, success=False)
            assert stale[-1]['error'] == 'stale_epoch'
            fresh = run(admin, 'retained-delete', *flags, operator, '--topic', 'lab/group')
            assert fresh[0]['request_id'].startswith('00000001')
            assert fresh[-1]['deleted'] is True
            tampered = root / 'tampered.json'
            private(tampered, archive.read_text(encoding='utf-8').replace('committed', 'changed', 1))
            invalid = subprocess.run([str(admin), 'audit-verify', '--file', str(tampered)],
                                     capture_output=True, timeout=8)
            assert invalid.returncode != 0
            stop(process, endpoint)
            process = start(broker, env, endpoint)
            assert run(admin, 'status', *flags, view)[0]['admin_epoch'] == 1
            assert run(admin, 'receipt', *flags, view, '--id', rotate_id)[0]['operation'] == 'audit_rotate'
            replay = raw_admin(endpoint, 'b' * 64, 'audit_rotate', rotate_id,
                               archive_sha256=rotated[0]['archive_sha256'],
                               archive_count=rotated[0]['archive_count'])
            assert replay['replayed'] is True and replay['admin_epoch'] == 1
            stop(process, endpoint)
            backup = root / 'verified-backup'
            saved = subprocess.run([str(admin), 'state', 'backup', '--state-dir', str(state),
                                    '--destination', str(backup)], capture_output=True, timeout=8)
            assert saved.returncode == 0
            process = start(broker, env, endpoint)
            diagnostic = ('diagnose', '--state-dir', str(state), '--bundle', str(bundle),
                          '--backup', str(backup), '--socket', str(endpoint),
                          '--token-file', str(view))
            checked = subprocess.run([str(admin), *diagnostic], capture_output=True,
                                     text=True, timeout=8)
            assert checked.returncode == 0, checked.stderr
            report = json.loads(checked.stdout)
            assert report['ok'] and report['broker_ready'] and report['backup_valid']
            assert report['broker_version'] and report['admin_version']
            assert 'fixture' not in checked.stdout and str(root) not in checked.stdout
            missing_backup = list(diagnostic)
            missing_backup[6] = str(root / 'missing-backup')
            checked = subprocess.run([str(admin), *missing_backup], capture_output=True,
                                     text=True, timeout=8)
            assert checked.returncode != 0 and json.loads(checked.stdout)['backup_valid'] is False
            assert str(root) not in checked.stdout
            invalid_bundle = list(diagnostic)
            invalid_bundle[4] = str(root / 'missing-bundle')
            checked = subprocess.run([str(admin), *invalid_bundle], capture_output=True,
                                     text=True, timeout=8)
            assert checked.returncode != 0 and json.loads(checked.stdout)['bundle_valid'] is False
            assert str(root) not in checked.stdout
            print('CHANNEL_CLI_PASS')
        finally:
            finish_fixture(process, endpoint)


if __name__ == '__main__':
    main()

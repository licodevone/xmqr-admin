#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""A25: owned monitor + deterministic CLI, authenticated aggregate and lease.

This checks status integration, not external MQTT device/process discovery.
"""
import argparse
from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import sys
import tempfile
import threading
import time

from verify_web_auth import Browser, digest, environment, port, private, stop, wait_port


FAKE_CLIENT = r'''#!/usr/bin/python3
import json, os, pathlib, sys, time
root = pathlib.Path(os.environ['A25_FIXTURE_ROOT'])
with (root / 'children.jsonl').open('a') as log:
    log.write(json.dumps(dict(pid=os.getpid())) + '\n')
if sys.argv[1] == 'pub':
    sys.stdin.buffer.read(8193)
    qos = int(sys.argv[sys.argv.index('--qos') + 1])
    print(json.dumps(dict(event='publish_complete', qos=qos,
          confirmation='sent' if qos == 0 else 'broker_protocol_ack')), flush=True)
    sys.exit(0)
print('Conectando; ainda sem SUBACK.', file=sys.stderr, flush=True)
confirmed = False
while True:
    mode = (root / 'client.control').read_text().strip()
    if mode == 'exit':
        sys.exit(4)
    if mode == 'connected' and not confirmed:
        print('Assinatura ativa. Aguardando mensagens; Ctrl+C para sair.', file=sys.stderr, flush=True)
        print(json.dumps(dict(event='publish', topic='a25/private-topic', qos=1,
                              retain=False, payload_bytes=list(b'PRIVATE_TEST_PAYLOAD_A25'))), flush=True)
        confirmed = True
    time.sleep(.03)
'''

METRICS = ('xmqr_connections_active', 'xmqr_sessions', 'xmqr_retained_messages',
           'xmqr_offline_messages', 'xmqr_inflight_messages', 'xmqr_persistence_commits_total',
           'xmqr_persistence_errors_total', 'xmqr_ready')


class MutableMonitor:
    def __init__(self, control):
        self.control = control
        private(control, 'ready')
        owner = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass

            def do_GET(self):
                mode = owner.control.read_text().strip()
                if mode == 'drop':
                    self.connection.shutdown(socket.SHUT_RDWR)
                    self.connection.close()
                    return
                if mode == 'timeout':
                    time.sleep(4)
                ready = mode != 'not_ready'
                status = 200 if self.path != '/ready' or ready else 503
                body = b'ok\n' if self.path == '/ready' else ''.join(
                    f'{name} {int(ready) if name == "xmqr_ready" else 1}\n'
                    for name in METRICS).encode()
                if mode == 'invalid':
                    body = b'private-upstream-diagnostic-not-for-browser'
                self.send_response(status)
                self.send_header('Content-Length', str(len(body)))
                self.end_headers()
                try:
                    self.wfile.write(body)
                except (BrokenPipeError, ConnectionResetError):
                    pass

        self.http = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        self.worker = threading.Thread(target=self.http.serve_forever, daemon=True)

    @property
    def number(self):
        return self.http.server_address[1]

    def __enter__(self):
        self.worker.start()
        return self

    def __exit__(self, *_):
        self.http.shutdown()
        self.http.server_close()
        self.worker.join(timeout=3)

    def mode(self, value):
        private(self.control, value)


@contextmanager
def owned_server(admin, root, monitor, *, users=False, console=True):
    token = secrets.token_hex(32)
    private(root / 'access.token', token)
    private(root / 'client.control', 'waiting')
    number = port()
    fields = environment(root, number, root / 'users.json' if users else None)
    fields['XMQR_ADMIN_MONITOR_BIND'] = f'127.0.0.1:{monitor.number}'
    if console:
        private(root / 'fake-client', FAKE_CLIENT)
        (root / 'fake-client').chmod(0o700)
        for name in ['sub', 'pub']:
            private(root / f'{name}.toml', 'version=1\n[connection]\nhost="127.0.0.1"\n')
        fields.update(XMQR_ADMIN_MQTT_CLIENT_BIN=str(root / 'fake-client'),
                      XMQR_ADMIN_MQTT_SUB_PROFILE=str(root / 'sub.toml'),
                      XMQR_ADMIN_MQTT_PUB_PROFILE=str(root / 'pub.toml'),
                      XMQR_ADMIN_MQTT_LABEL='PRIVATE_LABEL_A25', A25_FIXTURE_ROOT=str(root))
    with (root / 'server.log').open('ab') as log:
        child = subprocess.Popen([str(admin), 'serve'], cwd=root, env=fields,
                                 stdin=subprocess.DEVNULL, stdout=log, stderr=log)
        try:
            wait_port(child, number)
            yield Browser(number, token), child
        finally:
            stop(child)


def children_gone(root):
    path = root / 'children.jsonl'
    if not path.exists():
        return False
    for line in path.read_text().splitlines():
        try:
            os.kill(json.loads(line)['pid'], 0)
        except ProcessLookupError:
            continue
        return False
    return True


def aggregate(browser, *, bearer=False):
    status, _, data = browser.json('/api/status', bearer=bearer)
    assert status == 200, 'aggregate HTTP status'
    assert set(data) == {'admin_version', 'reachable', 'ready', 'error', 'metrics', 'systems'}
    assert set(data['systems']) == {'admin', 'broker', 'client'}
    for item in data['systems'].values():
        assert set(item) == {'online', 'reason'} and isinstance(item['online'], bool)
        assert isinstance(item['reason'], str)
    serialized = json.dumps(data)
    for forbidden in ['PRIVATE_TEST_PAYLOAD_A25', 'a25/private-topic', 'PRIVATE_LABEL_A25',
                      'password_file', 'sub.toml', 'payload_bytes', 'csrf_token']:
        assert forbidden not in serialized, 'aggregate exposed console information'
    return data


def until(check, seconds=8):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        result = check()
        if result:
            return result
        time.sleep(.05)
    raise AssertionError('aggregate transition deadline')


def run(admin, root, scenario):
    with MutableMonitor(root / 'monitor.control') as monitor:
        no_console = root / 'disabled'
        no_console.mkdir(mode=0o700)
        with owned_server(admin, no_console, monitor, console=False) as (legacy, _):
            with scenario('aggregate requires authentication and preserves legacy token access'):
                assert legacy.request('/api/status')[0] == 401
                sample = aggregate(legacy, bearer=True)
                assert sample['systems']['admin'] == dict(online=True, reason='available')
                assert sample['systems']['client'] == dict(online=False, reason='disabled')
        active = root / 'active'
        active.mkdir(mode=0o700)
        with owned_server(admin, active, monitor, users=True) as (browser, _):
            password = secrets.token_urlsafe(30)
            browser.authenticated(browser.request('/api/auth/setup', 'POST',
                                                  dict(bootstrap_token=browser.token, username='a25admin',
                                                       display_name='A25 admin', password=password)))
            assert browser.request('/api/auth/users', 'POST',
                                   dict(username='a25viewer', display_name='A25 viewer',
                                        password=password, role='viewer'))[0] == 200
            viewer = Browser(browser.number, browser.token)
            viewer.authenticated(viewer.request('/api/auth/login', 'POST',
                                                dict(username='a25viewer', password=password)))
            with scenario('viewer receives only fixed aggregates and cannot read MQTT payloads'):
                sample = aggregate(viewer)
                assert sample['systems']['client'] == dict(online=False, reason='disconnected')
                assert viewer.request('/api/mqtt')[0] == 403
            with scenario('ready monitor confirms broker and admin independently'):
                assert sample['reachable'] is True and sample['ready'] is True
                assert sample['systems']['broker'] == dict(online=True, reason='ready')
                assert sample['systems']['admin'] == dict(online=True, reason='available')
            with scenario('reachable not-ready monitor does not report broker online'):
                monitor.mode('not_ready')
                sample = aggregate(viewer)
                assert sample['reachable'] is True and sample['ready'] is False
                assert sample['systems']['broker'] == dict(online=False, reason='not_ready')
                assert sample['systems']['admin']['online'] is True
            with scenario('transport failure invalid metrics and timeout mean unconfirmed monitor'):
                for mode in ['drop', 'invalid', 'timeout']:
                    monitor.mode(mode)
                    sample = aggregate(viewer)
                    assert sample['systems']['broker'] == dict(online=False, reason='monitor_unavailable')
                    assert sample['reachable'] is False and sample['metrics'] == {}
                    assert sample['systems']['admin']['online'] is True
                monitor.mode('ready')
                assert aggregate(viewer)['systems']['broker']['online'] is True
            with scenario('client stays offline while connecting and becomes online only after SUBACK'):
                assert browser.request('/api/mqtt/connect', 'POST', dict(topic='a25/#', qos=1))[0] == 200
                assert aggregate(viewer)['systems']['client'] == dict(online=False, reason='connecting')
                private(active / 'client.control', 'connected')
                until(lambda: aggregate(viewer)['systems']['client'] == dict(online=True, reason='connected'))
                assert viewer.request('/api/mqtt')[0] == 403
            with scenario('disconnect updates aggregate and reaps owned subscriber'):
                assert browser.request('/api/mqtt/disconnect', 'POST', {})[0] == 200
                assert aggregate(viewer)['systems']['client'] == dict(online=False, reason='disconnected')
                until(lambda: children_gone(active))
            with scenario('unexpected CLI exit reports client error without changing healthy admin or broker'):
                private(active / 'client.control', 'connected')
                assert browser.request('/api/mqtt/connect', 'POST', dict(topic='a25/#', qos=1))[0] == 200
                until(lambda: aggregate(viewer)['systems']['client']['online'])
                private(active / 'client.control', 'exit')
                until(lambda: aggregate(viewer)['systems']['client'] == dict(online=False, reason='error'))
                assert aggregate(viewer)['systems']['broker']['online'] is True
                until(lambda: children_gone(active))
            with scenario('viewer aggregate polling does not renew the console subscription lease'):
                private(active / 'client.control', 'connected')
                assert browser.request('/api/mqtt/connect', 'POST', dict(topic='a25/#', qos=1))[0] == 200
                until(lambda: aggregate(viewer)['systems']['client']['online'])
                deadline = time.monotonic() + 68
                observations = 0
                while time.monotonic() < deadline:
                    value = aggregate(viewer)['systems']['client']
                    observations += 1
                    if value['reason'] == 'error':
                        break
                    assert value == dict(online=True, reason='connected')
                    time.sleep(1)
                assert observations >= 20 and value == dict(online=False, reason='error')
                until(lambda: children_gone(active))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--admin', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if os.name != 'posix':
        parser.error('Linux/WSL is required')
    args.admin = args.admin.resolve(strict=True)
    with args.report.open('x', encoding='utf-8') as output:
        result = dict(schema='xmqr-admin-system-status-a25-v1', status='FAIL',
                      started_unix=int(time.time()),
                      hashes=dict(admin_elf_sha256=digest(args.admin),
                                  fixture_sha256=digest(Path(__file__).resolve()),
                                  auth_helper_sha256=digest(Path(__file__).with_name('verify_web_auth.py'))),
                      scope='real admin HTTP; owned HTTP monitor and deterministic child CLI; no real MQTT network',
                      scenarios=[])

        @contextmanager
        def scenario(name):
            item = dict(name=name, status='FAIL')
            result['scenarios'].append(item)
            try:
                yield
                item['status'] = 'PASS'
                print('PASS:', name, flush=True)
            except Exception as failure:
                item['error_type'] = type(failure).__name__
                trace = failure.__traceback__
                while trace and trace.tb_next:
                    trace = trace.tb_next
                if trace:
                    item['error_line'] = trace.tb_lineno
                if isinstance(failure, AssertionError):
                    item['reason'] = str(failure)[:160]
                raise

        try:
            with tempfile.TemporaryDirectory(prefix='xmqr-admin-a25-') as directory:
                root = Path(directory)
                root.chmod(0o700)
                run(args.admin, root, scenario)
            result['status'] = 'PASS'
        except Exception as failure:
            result['error_type'] = type(failure).__name__
            print('FAIL: A25 fixture stopped; see sanitized immutable report.', file=sys.stderr)
        finally:
            result['finished_unix'] = int(time.time())
            result['passed'] = sum(row['status'] == 'PASS' for row in result['scenarios'])
            result['total'] = len(result['scenarios'])
            json.dump(result, output, indent=2)
            output.write('\n')
        return 0 if result['status'] == 'PASS' else 1


if __name__ == '__main__':
    sys.exit(main())

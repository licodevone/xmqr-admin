#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""A01/A02: owned Linux processes, temporary state/token and independent MQTT bytes."""
import argparse
from contextlib import contextmanager
import hashlib
import http.client
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import select
import signal
import socket
import struct
import subprocess
import tempfile
import threading
import time


def port():
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        return listener.getsockname()[1]


def wait_port(number, process):
    deadline = time.monotonic() + 12
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise AssertionError('owned process stopped during startup')
        try:
            with socket.create_connection(('127.0.0.1', number), timeout=.1):
                return
        except OSError:
            time.sleep(.03)
    raise AssertionError('owned process readiness timeout')


def stop(process):
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=8)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=3)


def request(number, path='/api/status', token=None, host=None, origin=None, method='GET', body=None):
    headers = {'Host': host or f'127.0.0.1:{number}'}
    if token is not None:
        headers['Authorization'] = 'Bearer ' + token
    if origin is not None:
        headers['Origin'] = origin
    connection = http.client.HTTPConnection('127.0.0.1', number, timeout=6)
    try:
        connection.request(method, path, body=body, headers=headers)
        response = connection.getresponse()
        content = response.read(100_000)
        return response.status, dict(response.getheaders()), content
    finally:
        connection.close()


def exact(stream, count):
    data = b''
    while len(data) < count:
        chunk = stream.recv(count - len(data))
        if not chunk:
            raise AssertionError('owned MQTT peer closed unexpectedly')
        data += chunk
    return data


def password_command(binary, arguments, password):
    import pty
    import termios
    pid, descriptor = pty.fork()
    if pid == 0:
        os.execl(binary,binary,*arguments)
    data, prompts, waited = b'', 0, False
    deadline = time.monotonic()+12
    def send_password():
        # The prompt is flushed before rpassword disables terminal echo.
        # Observe the PTY state, never race input against that transition.
        while termios.tcgetattr(descriptor)[3] & termios.ECHO:
            if time.monotonic() >= deadline:
                raise AssertionError('owned password prompt did not disable echo')
            time.sleep(.01)
        os.write(descriptor,password+b'\n')
    try:
        while time.monotonic() < deadline:
            if select.select([descriptor],[],[],.1)[0]:
                try:
                    chunk = os.read(descriptor,4096)
                except OSError:
                    break
                data += chunk
                if prompts == 0 and b'Password: ' in data:
                    send_password()
                    prompts=1
                if prompts == 1 and b'Confirm password: ' in data:
                    send_password()
                    prompts=2
        _, status = os.waitpid(pid,0)
        waited=True
        assert status == 0 and prompts == 2 and password not in data, dict(exit=os.waitstatus_to_exitcode(status), prompts=prompts, echoed=password in data)
    finally:
        os.close(descriptor)
        if not waited:
            try:
                os.kill(pid,signal.SIGKILL)
            except ProcessLookupError:
                pass
            os.waitpid(pid,0)


def authenticated_connect(number, password):
    with socket.create_connection(('127.0.0.1',number),timeout=3) as peer:
        peer.settimeout(3)
        def text(value):
            return struct.pack('!H',len(value))+value
        body=b'\x00\x04MQTT\x04\xc2\x00\x1e'+text(b'admin-compat')+text(b'student')+text(password)
        peer.sendall(bytes([0x10,len(body)])+body)
        response=exact(peer,4)
        if response[-1] == 0:
            peer.sendall(b'\xe0\x00')
        return response


class Lab:
    def __init__(self, admin, broker, admin_port=None, monitor_port=None, mqtt_port=None):
        self.root = tempfile.TemporaryDirectory(prefix='xmqr-admin-lab-')
        self.path = Path(self.root.name)
        self.admin, self.broker = str(admin.resolve()), str(broker.resolve())
        self.admin_port, self.monitor_port, self.mqtt_port = admin_port or port(), monitor_port or port(), mqtt_port or port()
        self.processes = []
        self.peer = None
        try:
            self.token_file = self.path/'access.token'
            self.run('token-init', '--file', str(self.token_file))
            self.token = self.token_file.read_text().strip()
            self.state = self.path/'state'
            self.state.mkdir(mode=0o700)
            self.environment = {k:v for k,v in os.environ.items() if not k.startswith(('MQTT_', 'XMQR_ADMIN_'))}
            self.broker_process = self.spawn(self.broker, dict(MQTT_MODE='open-lab', MQTT_STATE_DIR=str(self.state),
                MQTT_BIND=f'127.0.0.1:{self.mqtt_port}', MQTT_MONITOR_ENABLED='true', MQTT_MONITOR_BIND=f'127.0.0.1:{self.monitor_port}', RUST_LOG='error'))
            wait_port(self.mqtt_port, self.broker_process)
            wait_port(self.monitor_port, self.broker_process)
            self.admin_process = self.start_admin(self.monitor_port)
        except BaseException:
            self.close()
            raise

    def run(self, *args):
        result = subprocess.run([self.admin, *args], capture_output=True, timeout=15)
        if result.returncode:
            raise AssertionError('owned admin command failed')
        return result

    def spawn(self, binary, overrides):
        process = subprocess.Popen([binary, *(['serve'] if binary == self.admin else [])],
            env={**self.environment, **overrides}, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        self.processes.append(process)
        return process

    def start_admin(self, monitor_port):
        process = self.spawn(self.admin, dict(XMQR_ADMIN_TOKEN_FILE=str(self.token_file),
            XMQR_ADMIN_BIND=f'127.0.0.1:{self.admin_port}', XMQR_ADMIN_MONITOR_BIND=f'127.0.0.1:{monitor_port}'))
        wait_port(self.admin_port, process)
        return process

    def observe(self):
        code, _, data = request(self.admin_port, token=self.token)
        assert code == 200
        return json.loads(data)

    def publish(self):
        self.peer = socket.create_connection(('127.0.0.1', self.mqtt_port), timeout=3)
        self.peer.settimeout(3)
        identifier = b'admin-observer'
        connect = b'\x00\x04MQTT\x04\x00\x00\x1e' + struct.pack('!H', len(identifier)) + identifier
        self.peer.sendall(bytes([0x10,len(connect)])+connect)
        assert exact(self.peer,4) == b'\x20\x02\x00\x00'
        topic = b'admin-test/state'
        body = struct.pack('!H',len(topic))+topic+b'\x00\x07fixture-only'
        self.peer.sendall(bytes([0x33,len(body)])+body)
        assert exact(self.peer,4) == b'\x40\x02\x00\x07'

    def close(self):
        if self.peer:
            try:
                self.peer.sendall(b'\xe0\x00')
            except OSError:
                pass
            self.peer.close()
        for process in reversed(self.processes):
            stop(process)
        self.root.cleanup()


@contextmanager
def fake_monitor(mode):
    hits = []
    metrics = '\n'.join(name+' 1' for name in ['xmqr_connections_active','xmqr_sessions','xmqr_retained_messages',
        'xmqr_offline_messages','xmqr_inflight_messages','xmqr_persistence_commits_total','xmqr_persistence_errors_total','xmqr_ready'])+'\n'
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass
        def do_GET(self):
            hits.append(self.path)
            if mode == 'timeout':
                time.sleep(4)
            status, body = 200, metrics.encode()
            if self.path == '/ready':
                body = b'ok\n'
            if mode == 'unready':
                if self.path == '/ready':
                    status = 503
                body = body.replace(b'xmqr_ready 1', b'xmqr_ready 0')
            elif mode == 'invalid':
                body = b'private-content-must-not-return'
            elif mode == 'oversized':
                body = b'x'*65537
            elif mode == 'redirect':
                status = 302
            self.send_response(status)
            if status == 302:
                self.send_header('Location','http://127.0.0.1:1/never-follow')
            self.send_header('Content-Length',str(len(body)))
            self.end_headers()
            try:
                self.wfile.write(body)
            except (BrokenPipeError, ConnectionResetError):
                pass
    server = ThreadingHTTPServer(('127.0.0.1',0),Handler)
    worker = threading.Thread(target=server.serve_forever,daemon=True)
    worker.start()
    try:
        yield server.server_address[1], hits
    finally:
        server.shutdown()
        server.server_close()
        worker.join(timeout=3)


def state_digest(lab):
    return {p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in lab.state.iterdir() if p.name in ('state.wal','state.snapshot')}


def campaign(args):
    results = []
    lab = Lab(args.admin,args.broker)
    try:
        for label, options, expected in [
            ('missing bearer',{},401), ('wrong bearer',dict(token='b'*64),401),
            ('wrong host',dict(token=lab.token,host='evil.test:9080'),403),
            ('wrong origin',dict(token=lab.token,origin='http://evil.test'),403),
            ('null origin',dict(token=lab.token,origin='null'),403),
            ('write method',dict(token=lab.token,method='POST'),405),
            ('query',dict(token=lab.token,path='/api/status?token=must-not-return'),400),
            ('GET body',dict(token=lab.token,body=b'body'),400),
        ]:
            code, headers, body = request(lab.admin_port,**options)
            assert code == expected and headers.get('cache-control') == 'no-store'
            assert 'access-control-allow-origin' not in headers
            assert lab.token.encode() not in body and b'must-not-return' not in body
            results.append(label)
        code, headers, body = request(lab.admin_port,path='/')
        assert code == 200 and b'XMQR Admin' in body and "frame-ancestors 'none'" in headers['content-security-policy']
        results.append('public shell secure headers')
        baseline = lab.observe()
        assert baseline['reachable'] and baseline['ready'] and baseline['metrics']['xmqr_retained_messages'] == 0
        lab.publish()
        observed = lab.observe()
        assert observed['metrics']['xmqr_connections_active'] == 1 and observed['metrics']['xmqr_retained_messages'] == 1
        results.append('independent MQTT client changes real aggregates')
        before = state_digest(lab)
        for _ in range(3):
            assert lab.observe()['ready']
        assert state_digest(lab) == before
        results.append('panel reads do not modify durable documents')
        stop(lab.broker_process)
        observed = lab.observe()
        assert not observed['reachable'] and not observed['ready'] and observed['metrics'] == {}
        results.append('stopped broker is unavailable, not zero')
        stop(lab.admin_process)
        users=lab.path/'users.toml'
        password_command(lab.admin,['user','create','--users-file',str(users),'--username','student'],b'first-public-fixture')
        assert lab.run('user','list','--users-file',str(users)).stdout == b'student\n'
        password_state=lab.path/'password-state'
        password_state.mkdir(mode=0o700)
        def password_broker():
            process=lab.spawn(lab.broker,dict(MQTT_MODE='password-lab',MQTT_STATE_DIR=str(password_state),MQTT_BIND=f'127.0.0.1:{lab.mqtt_port}',MQTT_USERS_FILE=str(users),RUST_LOG='error'))
            wait_port(lab.mqtt_port,process)
            return process
        process=password_broker()
        assert authenticated_connect(lab.mqtt_port,b'first-public-fixture') == b'\x20\x02\x00\x00'
        stop(process)
        password_command(lab.admin,['user','update-password','--users-file',str(users),'--username','student'],b'new-public-fixture')
        process=password_broker()
        assert authenticated_connect(lab.mqtt_port,b'first-public-fixture')[-1] == 5
        assert authenticated_connect(lab.mqtt_port,b'new-public-fixture') == b'\x20\x02\x00\x00'
        stop(process)
        lab.run('user','delete','--users-file',str(users),'--username','student')
        assert lab.run('user','list','--users-file',str(users)).stdout == b''
        results.append('new CLI users/password files accepted by independent broker, old password refused after restart')
        for mode in ['unready','invalid','oversized','redirect','timeout']:
            with fake_monitor(mode) as (number,hits):
                lab.admin_process = lab.start_admin(number)
                started = time.monotonic()
                data = lab.observe()
                assert time.monotonic()-started < 5
                if mode == 'unready':
                    assert data['reachable'] and not data['ready']
                else:
                    assert not data['reachable'] and data['metrics'] == {}
                assert 'private-content' not in json.dumps(data)
                if mode == 'redirect':
                    assert hits == ['/ready']
                stop(lab.admin_process)
                results.append('monitor '+mode)
        report = dict(status='PASS',scenarios=results,count=len(results),admin_version='0.1.0',broker_version='0.11.5',
                      limits=['test-owned loopback only','not exhaustive HTTP security or MQTT conformance','no real operator state touched'])
        args.report.parent.mkdir(parents=True,exist_ok=True)
        args.report.write_text(json.dumps(report,indent=2)+'\n')
        print('ADMIN_INTEGRATION_PASS',len(results))
    finally:
        lab.close()


def browser_fixture(args):
    lab = Lab(args.admin,args.broker,args.admin_port,args.monitor_port,args.mqtt_port)
    try:
        lab.publish()
        args.handoff.parent.mkdir(parents=True,exist_ok=True)
        args.handoff.write_text(json.dumps(dict(url=f'http://127.0.0.1:{lab.admin_port}',token_file=str(lab.token_file),mqtt_port=lab.mqtt_port,monitor_port=lab.monitor_port)))
        deadline = time.monotonic()+120
        while time.monotonic() < deadline and not args.stop_file.exists():
            time.sleep(.1)
    finally:
        lab.close()
        args.handoff.with_suffix('.closed.json').write_text(json.dumps(dict(cleaned=True)))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--admin',type=Path,required=True)
    parser.add_argument('--broker',type=Path,required=True)
    parser.add_argument('--report',type=Path,default=Path('dist/integration.json'))
    parser.add_argument('--browser-fixture',action='store_true')
    parser.add_argument('--admin-port',type=int)
    parser.add_argument('--monitor-port',type=int)
    parser.add_argument('--mqtt-port',type=int)
    parser.add_argument('--handoff',type=Path)
    parser.add_argument('--stop-file',type=Path)
    args = parser.parse_args()
    if not args.browser_fixture and args.report.exists():
        parser.error('report exists; choose a new output')
    if args.browser_fixture:
        if not all([args.handoff,args.stop_file,args.admin_port,args.monitor_port,args.mqtt_port]):
            parser.error('browser fixture requires ports/handoff/stop-file')
        browser_fixture(args)
    else:
        campaign(args)

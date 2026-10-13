#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""A03 owned disposable Docker namespace/volume; no published ports or real state."""
import argparse
import json
from pathlib import Path
import subprocess
import time
import uuid


def docker(*args, check=True, timeout=35):
    result = subprocess.run(['docker', *args], capture_output=True, timeout=timeout)
    if check and result.returncode:
        for line in result.stderr.decode('utf8', errors='replace').splitlines():
            if line.startswith('Owned Docker HTTP probe failed at '):
                raise RuntimeError(line)
        raise RuntimeError('owned docker command failed: ' + args[0])
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--admin-image', required=True)
    parser.add_argument('--broker-image', default='lepinheiro100/xmqr-broker:0.11.5')
    parser.add_argument('--client-image', default='lepinheiro100/xmqr-client:0.8.1')
    parser.add_argument('--probe-image', default='node:24-bookworm-slim')
    parser.add_argument('--revision', required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists():
        parser.error('report exists; choose a new output')
    root = Path(__file__).resolve().parents[1]
    name = 'xmqr-a03-' + uuid.uuid4().hex[:12]
    volume, broker, admin = name + '-work', name + '-broker', name + '-admin'
    owned_containers, created = [], False
    checks = []
    try:
        image = json.loads(docker('image', 'inspect', args.admin_image).stdout)[0]
        config = image['Config']
        assert image['Os'] == 'linux' and image['Architecture'] == 'amd64'
        assert config['User'] == '10001:10001' and config['StopSignal'] == 'SIGTERM'
        assert config['Entrypoint'] == ['/usr/local/bin/xmqr-admin']
        assert config['Labels']['org.opencontainers.image.version'] == '0.1.0'
        assert config['Labels']['org.opencontainers.image.revision'] == args.revision
        assert docker('run', '--rm', '--network', 'none', '--read-only', '--cap-drop=ALL', args.admin_image, '--version').stdout.strip() == b'xmqr-admin 0.1.0'
        docker('run', '--rm', '--network', 'none', '--read-only', '--cap-drop=ALL', '--entrypoint', 'sh', args.admin_image, '-c', 'test ! -e /usr/local/bin/mqtt-broker && test ! -e /usr/local/bin/mqtt-client')
        checks.append('dedicated non-root immutable-version image, no broker/client executable')
        docker('volume', 'create', volume)
        created = True
        docker('run', '--rm', '--network', 'none', '--user', '0', '--entrypoint', 'sh', '-v', volume + ':/work', args.admin_image, '-c',
               'mkdir /work/config /work/state && chown -R 10001:10001 /work && chmod 700 /work /work/config /work/state')

        def command(*options, check=True):
            return docker('run', '--rm', '--network', 'none', '--read-only', '--cap-drop=ALL', '--security-opt', 'no-new-privileges',
                          '--tmpfs', '/tmp:rw,noexec,nosuid,size=64m,mode=1777', '-v', volume + ':/work', args.admin_image, *options, check=check)

        command('token-init', '--file', '/work/config/access.token')
        assert command('token-init', '--file', '/work/config/access.token', check=False).returncode != 0
        checks.append('private random token creation, existing token never overwritten')
        docker('run', '-d', '--name', broker, '--read-only', '--cap-drop=ALL', '--security-opt', 'no-new-privileges',
               '-v', volume + ':/work', '-e', 'MQTT_MODE=open-lab', '-e', 'MQTT_BIND=127.0.0.1:1883',
               '-e', 'MQTT_MONITOR_ENABLED=true', '-e', 'MQTT_MONITOR_BIND=127.0.0.1:9090',
               '-e', 'MQTT_STATE_DIR=/work/state', '-e', 'RUST_LOG=info', args.broker_image)
        owned_containers.append(broker)
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            logs = docker('logs', broker)
            if b'listener started' in logs.stdout + logs.stderr:
                break
            time.sleep(.1)
        else:
            raise AssertionError('owned broker startup failed')
        docker('run', '--rm', '--network', 'container:' + broker, '--read-only', '--cap-drop=ALL', '--security-opt', 'no-new-privileges',
               args.client_image, 'pub', '--open-lab', '--host', '127.0.0.1', '--port', '1883', '--topic', 'admin-fixture/state',
               '--qos', '1', '--message', 'public-test-fixture', '--retain', 'true')
        docker('run', '-d', '--name', admin, '--network', 'container:' + broker, '--read-only', '--cap-drop=ALL', '--security-opt', 'no-new-privileges',
               '-v', volume + ':/work:ro', '-e', 'XMQR_ADMIN_TOKEN_FILE=/work/config/access.token',
               '-e', 'XMQR_ADMIN_BIND=127.0.0.1:9080', '-e', 'XMQR_ADMIN_MONITOR_BIND=127.0.0.1:9090', args.admin_image, 'serve')
        owned_containers.append(admin)

        def probe(mode):
            value = docker('run', '--rm', '--user', '10001:10001', '--network', 'container:' + admin,
                           '--read-only', '--cap-drop=ALL', '--security-opt', 'no-new-privileges',
                           '-v', volume + ':/work:ro', '-v', str(root / 'scripts/docker_probe.mjs') + ':/probe.mjs:ro',
                           args.probe_image, 'node', '/probe.mjs', mode)
            report = json.loads(value.stdout)
            assert report['status'] == 'PASS'
            checks.extend(report['checks'])

        probe('live')
        assert command('state', 'backup', '--state-dir', '/work/state', '--destination', '/work/forbidden', check=False).returncode != 0
        checks.append('backup refuses active broker lock')
        docker('stop', '--time', '20', broker)
        assert json.loads(docker('inspect', broker).stdout)[0]['State']['ExitCode'] == 0
        probe('stopped')
        command('state', 'backup', '--state-dir', '/work/state', '--destination', '/work/backup')
        command('state', 'verify', '--backup', '/work/backup')
        command('state', 'restore', '--backup', '/work/backup', '--state-dir', '/work/restored')
        assert command('state', 'restore', '--backup', '/work/backup', '--state-dir', '/work/restored', check=False).returncode != 0
        checks.append('offline backup/verify/restore and existing destination refusal')
        started = time.monotonic()
        docker('stop', '--time', '8', admin)
        assert time.monotonic() - started < 8
        assert json.loads(docker('inspect', admin).stdout)[0]['State']['ExitCode'] == 0
        checks.append('SIGTERM exits cleanly within bounded interval')
        args.report.parent.mkdir(parents=True, exist_ok=True)
        with args.report.open('x', encoding='utf8') as stream:
            json.dump(dict(status='PASS', count=len(checks), scenarios=checks, revision=args.revision, admin_image=args.admin_image,
                           broker_image=args.broker_image, client_image=args.client_image, probe_image=args.probe_image,
                           isolation='test-owned shared loopback namespace, no published ports, disposable private volume',
                           limits=['not remote panel certification', 'not exhaustive HTTP/MQTT conformance']), stream, indent=2)
            stream.write('\n')
        print('ADMIN_DOCKER_PASS', len(checks))
    finally:
        for container in reversed(owned_containers):
            docker('rm', '-f', container, check=False)
        if created:
            docker('volume', 'rm', volume, check=False)


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""A03 export accepted native packages and exact Docker image; never publishes."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def validate_image(image, version, revision):
    config = image['Config']
    if image['Os'] != 'linux' or image['Architecture'] != 'amd64':
        raise ValueError('only Linux/amd64 accepted')
    if config['User'] != '10001:10001' or config['StopSignal'] != 'SIGTERM':
        raise ValueError('runtime user/signal mismatch')
    if config['Entrypoint'] != ['/usr/local/bin/xmqr-admin']:
        raise ValueError('entrypoint mismatch')
    labels = config.get('Labels') or {}
    if labels.get('org.opencontainers.image.version') != version or labels.get('org.opencontainers.image.revision') != revision:
        raise ValueError('version/source revision mismatch')
    return {'id': image['Id'], 'source_revision': revision, 'version': version,
            'platform': 'linux/amd64', 'user': config['User'], 'runtime': 'Debian Bookworm'}


def export_image(reference, target):
    with tempfile.TemporaryFile() as errors:
        process = subprocess.Popen(['docker', 'image', 'save', reference], stdout=subprocess.PIPE, stderr=errors)
        try:
            with target.open('xb') as stream:
                with gzip.GzipFile(filename='', fileobj=stream, mode='wb', mtime=0) as compressed:
                    shutil.copyfileobj(process.stdout, compressed)
            if process.wait() != 0:
                raise RuntimeError('docker save failed')
        finally:
            process.stdout.close()
            if process.poll() is None:
                process.kill()
                process.wait()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--native', type=Path, required=True)
    parser.add_argument('--image', default='lepinheiro100/xmqr-admin-ui:0.1.0')
    parser.add_argument('--evidence', type=Path, action='append', default=[])
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    package = tomllib.loads((root / 'Cargo.toml').read_text(encoding='utf8'))['package']
    version = package['version']
    dirty = subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=all'], cwd=root, text=True).strip()
    if dirty:
        parser.error('accepted clean committed source is required')
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
    native_manifest = args.native / f'xmqr-admin-{version}-distribution-manifest.json'
    native = json.loads(native_manifest.read_text(encoding='utf8'))
    if native['draft'] or native['source_revision'] != revision or native['version'] != version:
        parser.error('native packages must be from this exact accepted commit')
    if args.image != 'lepinheiro100/xmqr-admin-ui:' + version:
        parser.error('only the new numbered admin UI repository is accepted')
    image = json.loads(subprocess.check_output(['docker', 'image', 'inspect', args.image]))[0]
    metadata = validate_image(image, version, revision)
    actual = subprocess.check_output(['docker', 'run', '--rm', '--network', 'none', '--read-only', '--cap-drop=ALL',
                                      '--security-opt', 'no-new-privileges', args.image, '--version'], text=True).strip()
    if actual != 'xmqr-admin ' + version:
        parser.error('runtime binary version mismatch')
    if args.output.exists():
        parser.error('output exists; use a new owned directory')
    sources = []
    for artifact in native['artifacts']:
        name = artifact['file']
        if name != Path(name).name or not name.startswith('xmqr-admin'):
            parser.error('invalid artifact filename')
        source = args.native / name
        if source.is_symlink() or digest(source) != artifact['sha256'] or source.stat().st_size != artifact['size']:
            parser.error('native artifact hash/size mismatch')
        sources.append(source)
    sources.append(native_manifest)
    for evidence in args.evidence:
        # Only explicitly selected sanitized PASS JSON reports, not logs/state.
        if evidence.is_symlink() or evidence.suffix != '.json':
            parser.error('evidence must be regular sanitized JSON')
        report = json.loads(evidence.read_text(encoding='utf8'))
        if report.get('status') != 'PASS':
            parser.error('evidence not accepted')
        sources.append(evidence)
    args.output.mkdir(parents=True)
    artifacts = []
    for source in sources:
        target = args.output / source.name
        with source.open('rb') as src, target.open('xb') as dst:
            shutil.copyfileobj(src, dst)
        artifacts.append(target)
    archive = args.output / f'xmqr-admin-ui-{version}-docker-linux-amd64.tar.gz'
    export_image(args.image, archive)
    artifacts.append(archive)
    manifest = dict(schema_version=1, package=package['name'], version=version, source_revision=revision,
                    git_tag='v' + version, native_target='Ubuntu 26.04 linux/amd64', docker_target='Debian Bookworm linux/amd64',
                    images={args.image: metadata}, experimental=True, artifacts=[])
    for artifact in artifacts:
        checksum = digest(artifact)
        with (args.output / (artifact.name + '.sha256')).open('x', encoding='utf8') as stream:
            stream.write(checksum + '  ' + artifact.name + '\n')
        manifest['artifacts'].append({'file': artifact.name, 'sha256': checksum, 'size': artifact.stat().st_size})
    target = args.output / f'xmqr-admin-{version}-release-manifest.json'
    with target.open('x', encoding='utf8') as stream:
        json.dump(manifest, stream, indent=2)
        stream.write('\n')
    with (args.output / (target.name + '.sha256')).open('x', encoding='utf8') as stream:
        stream.write(digest(target) + '  ' + target.name + '\n')
    print(target)


if __name__ == '__main__':
    main()

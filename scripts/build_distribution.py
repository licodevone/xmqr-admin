#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Luis E. S. Pinheiro
"""Build native Ubuntu26 amd64 tar/deb, original notices and SHA/provenance.

Never installs packages, creates services, reads real state or publishes.
Output must be new; local draft builds are labeled draft until exact-commit gates.
"""
import argparse
import gzip
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
import tempfile
import tomllib

from collect_notices import collect, digest


def run(*arguments, **kwargs):
    return subprocess.run(arguments, check=True, **kwargs)


def revision_number(value):
    if not re.fullmatch(r'[1-9][0-9]{0,5}', value):
        raise ValueError('Debian revision must be 1..999999 without leading zeros')
    return value


def pinned_core_revision(root):
    manifest = tomllib.loads((root / 'Cargo.toml').read_text(encoding='utf8'))
    dependency = manifest['dependencies'].get('mqtt-broker')
    if not isinstance(dependency, dict):
        raise ValueError('mqtt-broker must use an immutable Git dependency')
    url, revision = dependency.get('git'), dependency.get('rev')
    if not isinstance(url, str) or not isinstance(revision, str) or not re.fullmatch(r'[0-9a-f]{40}', revision):
        raise ValueError('mqtt-broker Git revision must be a full commit SHA')
    lock = tomllib.loads((root / 'Cargo.lock').read_text(encoding='utf8'))
    entries = [item for item in lock['package'] if item['name'] == 'mqtt-broker']
    if len(entries) != 1 or entries[0].get('source') != f'git+{url}?rev={revision}#{revision}':
        raise ValueError('mqtt-broker Cargo.lock source differs from Cargo.toml pin')
    return revision


def source_revision(root, allow_draft):
    commit = subprocess.run(['git', 'rev-parse', '--verify', 'HEAD'], cwd=root,
                            text=True, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    dirty = subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=all'],
                                    cwd=root, text=True).strip()
    if commit.returncode or dirty:
        if not allow_draft:
            raise ValueError('accepted clean committed source is required; --allow-draft is test-only')
        return 'draft-uncommitted', True
    return commit.stdout.strip(), False


def copy_file(source, target, mode=0o644):
    if source.is_symlink() or not source.is_file():
        raise ValueError('input must be a regular file')
    target.parent.mkdir(parents=True, exist_ok=True)
    with source.open('rb') as src, target.open('xb') as dst:
        shutil.copyfileobj(src, dst)
    target.chmod(mode)


def make_tar(stage, destination, epoch):
    with destination.open('xb') as stream:
        with gzip.GzipFile(filename='', fileobj=stream, mode='wb', mtime=0) as zipped:
            with tarfile.open(fileobj=zipped, mode='w') as archive:
                for path in sorted(stage.rglob('*')):
                    if path.is_symlink():
                        raise ValueError('symlink in distribution')
                    item = archive.gettarinfo(str(path), arcname=path.relative_to(stage).as_posix())
                    item.uid = item.gid = 0
                    item.uname = item.gname = 'root'
                    item.mtime = epoch
                    if path.is_file():
                        with path.open('rb') as source:
                            archive.addfile(item, source)
                    else:
                        archive.addfile(item)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--skip-build', action='store_true')
    parser.add_argument('--bin-dir', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--revision', default='1')
    parser.add_argument('--allow-draft', action='store_true')
    args = parser.parse_args()
    try:
        revision_number(args.revision)
    except ValueError as error:
        parser.error(str(error))
    release = dict(line.split('=', 1) for line in Path('/etc/os-release').read_text().splitlines() if '=' in line)
    if release.get('ID', '').strip('"') != 'ubuntu' or release.get('VERSION_ID', '').strip('"') != '26.04':
        parser.error('build requires Ubuntu 26.04; other releases are not certified')
    architecture = subprocess.check_output(['dpkg', '--print-architecture'], text=True).strip()
    if architecture != 'amd64':
        parser.error('only native Linux amd64 is certified')
    root = Path(__file__).resolve().parents[1]
    package = tomllib.loads((root / 'Cargo.toml').read_text())['package']
    if package['name'] != 'xmqr-admin':
        parser.error('not an xmqr-admin source tree')
    version = package['version']
    core_revision = pinned_core_revision(root)
    commit, draft = source_revision(root, args.allow_draft)
    epoch = 0 if draft else int(subprocess.check_output(['git', 'show', '-s', '--format=%ct', 'HEAD'], cwd=root))
    if not args.skip_build:
        run('cargo', 'build', '--locked', '--release', cwd=root)
    binary = (args.bin_dir or Path(os.environ.get('CARGO_TARGET_DIR', root / 'target')) / 'release').resolve() / 'xmqr-admin'
    with binary.open('rb') as stream:
        if stream.read(20) != b'\x7fELF\x02\x01\x01\x00' + b'\x00' * 8 + b'\x03\x00\x3e\x00':
            parser.error('binary is not the expected Linux amd64 ELF PIE')
    if subprocess.check_output([str(binary), '--version'], text=True).strip() != 'xmqr-admin ' + version:
        parser.error('binary version differs from Cargo.toml')
    out = args.output.resolve()
    if out.exists():
        parser.error('output exists; choose a fresh owned directory')
    out.mkdir(parents=True)
    notices = collect(root, out / 'third-party-notices')
    with tempfile.TemporaryDirectory(prefix='xmqr-admin-distribution-') as temporary:
        temp = Path(temporary)
        deb_stage, tar_stage = temp / 'deb', temp / 'tar'
        deb_doc = deb_stage / 'usr/share/doc/xmqr-admin'
        tar_doc = tar_stage / 'share/doc/xmqr-admin'
        copy_file(binary, deb_stage / 'usr/bin/xmqr-admin', 0o755)
        copy_file(binary, tar_stage / 'bin/xmqr-admin', 0o755)
        for doc in ('LICENSE', 'README.md', 'ORIGIN.json', 'docs/third-party-licenses.md', 'docs/distribution.md'):
            for location in (deb_doc, tar_doc):
                copy_file(root / doc, location / Path(doc).name)
        # Debian convention: retained even by the minimal Ubuntu Docker policy.
        copy_file(root / 'LICENSE', deb_doc / 'copyright')
        for location in (deb_doc, tar_doc):
            shutil.copytree(out / 'third-party-notices', location / 'third-party-notices')
            provenance = {'version': version, 'source_revision': commit, 'draft': draft,
                          'core_revision': core_revision,
                          'target': 'Ubuntu 26.04 linux/amd64', 'binary_sha256': digest(binary),
                          'cargo_lock_sha256': digest(root / 'Cargo.lock')}
            (location / 'distribution-provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')
        (temp / 'debian').mkdir()
        (temp / 'debian/control').write_text('Source: xmqr-admin\n\nPackage: xmqr-admin\nArchitecture: any\nDescription: XMQR admin\n')
        result = subprocess.check_output(['dpkg-shlibdeps', '-O', '-e' + str(deb_stage / 'usr/bin/xmqr-admin')], cwd=temp, text=True)
        dependencies = next(line.split('=', 1)[1] for line in result.splitlines() if line.startswith('shlibs:Depends='))
        control = deb_stage / 'DEBIAN'
        control.mkdir()
        (control / 'control').write_text(
            f'Package: xmqr-admin\nVersion: {version}-{args.revision}\nArchitecture: amd64\n'
            f'Maintainer: XMQR maintainers <xmqr@localhost>\nDepends: {dependencies}\n'
            'Section: net\nPriority: optional\nHomepage: https://github.com/licodevone/xmqr-admin\n'
            'Description: Experimental local XMQR administrative CLI and observer\n'
            ' Built for Ubuntu 26.04 amd64. No services are installed or enabled.\n')
        tar = out / f'xmqr-admin-{version}-ubuntu26-amd64.tar.gz'
        make_tar(tar_stage, tar, epoch)
        deb = out / f'xmqr-admin_{version}-{args.revision}_amd64.deb'
        run('dpkg-deb', '--root-owner-group', '--build', str(deb_stage), str(deb),
            env={**os.environ, 'SOURCE_DATE_EPOCH': str(epoch)})
    artifacts = []
    for artifact in (tar, deb):
        checksum = digest(artifact)
        (out / (artifact.name + '.sha256')).write_text(checksum + '  ' + artifact.name + '\n')
        artifacts.append({'file': artifact.name, 'sha256': checksum, 'size': artifact.stat().st_size})
    manifest = {'schema_version': 1, 'package': 'xmqr-admin', 'version': version,
                'debian_revision': args.revision, 'source_revision': commit, 'draft': draft,
                'experimental': True, 'native_target': 'Ubuntu 26.04 linux/amd64',
                'docker_target': 'Debian Bookworm linux/amd64 (separate build)',
                'core_revision': core_revision,
                'notice_packages': len(notices['packages']),
                'notice_files': sum(len(p['files']) for p in notices['packages']),
                'artifacts': artifacts}
    path = out / f'xmqr-admin-{version}-distribution-manifest.json'
    path.write_text(json.dumps(manifest, indent=2) + '\n')
    (out / (path.name + '.sha256')).write_text(digest(path) + '  ' + path.name + '\n')
    print(path)


if __name__ == '__main__':
    main()

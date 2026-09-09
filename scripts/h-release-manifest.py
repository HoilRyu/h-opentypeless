#!/usr/bin/env python3
"""Record checksums/provenance; validate packages before a draft can be created."""
import argparse
import hashlib
import fcntl
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]

def git(*args):
    return subprocess.check_output(['git', '-C', str(ROOT), *args], text=True).strip()

def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()

def snapshot():
    status = git('status', '--porcelain')
    h = hashlib.sha256(status.encode())
    h.update(git('diff', 'HEAD', '--binary').encode())
    for name in git('ls-files', '--others', '--exclude-standard').splitlines():
        h.update(name.encode())
        h.update(digest(ROOT / name).encode())
    return dict(source_commit=git('rev-parse', 'HEAD'), source_dirty=bool(status), source_fingerprint=h.hexdigest())

def record(path, platform, version, source=None):
    path = path.resolve()
    metadata = path.with_name(path.name + '.json')
    if metadata.exists():
        raise ValueError('Provenance already exists; do not overwrite released packages.')
    source = json.loads(source.read_text()) if source else snapshot()
    if source != snapshot():
        raise ValueError('Source changed during build; rebuild before packaging.')
    with (path.parent / '.manifest.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        if metadata.exists():
            raise ValueError('Provenance already exists; do not overwrite released packages.')
        metadata.write_text(json.dumps({
            'file': path.name, 'sha256': digest(path), 'platform': platform, 'version': version,
            **source,
        }, indent=2) + '\n')
        checksum_lines = []
        for item in sorted(path.parent.glob('*.json')):
            data = json.loads(item.read_text())
            checksum_lines.append(f"{data['sha256']}  {data['file']}\n")
        (path.parent / 'SHA256SUMS').write_text(''.join(checksum_lines))

def verify(directory, commit):
    records = sorted(directory.glob('*.json'))
    if len(records) != 2:
        raise ValueError('First release requires exactly two package provenance records.')
    seen = set()
    for item in records:
        data = json.loads(item.read_text())
        name = data['file']
        if pathlib.Path(name).name != name:
            raise ValueError('Unsafe package filename.')
        if data['source_dirty'] is not False or data['source_commit'] != commit:
            raise ValueError(f'{name}: rebuild from the clean release commit before publishing.')
        if digest(directory / name) != data['sha256']:
            raise ValueError(f'{name}: checksum mismatch.')
        seen.add(data['platform'])
    if seen != {'macos-arm64', 'android-arm64'}:
        raise ValueError('First release requires exactly macos-arm64 and android-arm64 packages.')
    expected = ''.join(f"{json.loads(p.read_text())['sha256']}  {json.loads(p.read_text())['file']}\n" for p in records)
    if (directory / 'SHA256SUMS').read_text() != expected:
        raise ValueError('SHA256SUMS does not match package provenance.')
    print('Package checksums and clean source commit verified.')

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest='command', required=True)
    p = sub.add_parser('record'); p.add_argument('path', type=pathlib.Path)
    p.add_argument('--platform', required=True); p.add_argument('--version', required=True)
    p.add_argument('--source', type=pathlib.Path)
    p = sub.add_parser('snapshot'); p.add_argument('path', type=pathlib.Path)
    p = sub.add_parser('verify'); p.add_argument('path', type=pathlib.Path); p.add_argument('--commit', required=True)
    args = parser.parse_args()
    try:
        if args.command == 'record': record(args.path, args.platform, args.version, args.source)
        elif args.command == 'snapshot': args.path.write_text(json.dumps(snapshot()))
        else: verify(args.path, args.commit)
    except (ValueError, KeyError, OSError) as error:
        parser.exit(1, str(error) + '\n')

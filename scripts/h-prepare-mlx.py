#!/usr/bin/env python3
"""Prepare a pinned, relocatable macOS arm64 runtime; no end-user pip needed."""
import hashlib
import json
import pathlib
import shutil
import subprocess
import tarfile
import urllib.request

URL = 'https://github.com/astral-sh/python-build-standalone/releases/download/20260901/cpython-3.12.14%2B20260901-aarch64-apple-darwin-install_only_stripped.tar.gz'
SHA = '81a359f1cfadd4da11766534c5913791cea55f26e1bb902cacd2a531bb1e4b2b'


def prepare(root):
    source = pathlib.Path(__file__).resolve().parent.parent
    root.mkdir(parents=True, exist_ok=True)
    lock = source / 'scripts/mlx/requirements.lock'
    fingerprint = hashlib.sha256(lock.read_bytes() + SHA.encode()).hexdigest()
    bundle = root / 'runtime'
    marker = bundle / 'runtime.json'
    if not marker.exists() or json.loads(marker.read_text()).get('fingerprint') != fingerprint:
        archive = root / 'python.tar.gz'
        if not archive.exists():
            urllib.request.urlretrieve(URL, archive)
        if hashlib.sha256(archive.read_bytes()).hexdigest() != SHA:
            raise RuntimeError('Python archive checksum mismatch')
        stage = root / 'staging'
        if stage.exists():
            shutil.rmtree(stage)
        stage.mkdir()
        with tarfile.open(archive) as tar:
            tar.extractall(stage, filter='data')
        runtime = stage / 'python'
        subprocess.run([str(runtime/'bin/python3'), '-m', 'pip', 'install', '--require-hashes',
                        '--only-binary=:all:', '--platform', 'macosx_14_0_arm64', '--upgrade',
                        '--target', str(runtime/'lib/python3.12/site-packages'), '-r', str(lock)], check=True)
        if bundle.exists():
            shutil.rmtree(bundle)
        runtime.rename(bundle)
        marker.write_text(json.dumps({'fingerprint': fingerprint, 'python': '3.12.14', 'mlx': '0.31.1', 'asr': '0.4.0', 'whisper': '0.4.3', 'minimum_macos': '14.0'}))
    for name in ('worker.py', 'whisper_session.py'):
        shutil.copy2(source/'native/mlx-stt'/name, bundle/name)
    shutil.copy2(lock, bundle/'requirements.lock')
    aux = bundle/'tokenizers'
    aux.mkdir(exist_ok=True)
    # Small, pinned auxiliary files remain separate from the existing weight catalog.
    manifest_path = source/'native/mlx-stt/tokenizers.json'
    manifest = json.loads(manifest_path.read_text())
    for item in manifest:
        dest = aux/(item['id'] + '.json')
        shutil.copy2(source/'native/mlx-stt/tokenizers'/dest.name, dest)
        if hashlib.sha256(dest.read_bytes()).hexdigest() != item['sha256']:
            raise RuntimeError('Tokenizer checksum mismatch')
    shutil.copy2(manifest_path, bundle/'tokenizers.json')
    return bundle


if __name__ == '__main__':
    print(prepare(pathlib.Path.home()/'.local/share/h-opentypeless/mlx-build'))

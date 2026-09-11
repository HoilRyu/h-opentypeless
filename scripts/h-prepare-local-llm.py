#!/usr/bin/env python3
"""Prepare pinned Ollama and the parent-lifetime supervisor outside the checkout."""
import argparse, hashlib, json, pathlib, platform, subprocess, tarfile, urllib.request
VERSION = '0.34.0'
SHA256 = 'dd12b00bcce2d6551178e67ada90d5af9f75bdb54a118b96655250fa3e8ef734'
def prepare(root, config):
    if platform.system() != 'Darwin' or platform.machine() != 'arm64':
        raise SystemExit('Built-in LLM currently supports macOS Apple Silicon only')
    root.mkdir(parents=True, exist_ok=True)
    archive = root / ('ollama-' + VERSION + '.tgz')
    if not archive.exists():
        temporary = archive.with_suffix('.part')
        urllib.request.urlretrieve('https://github.com/ollama/ollama/releases/download/v'+VERSION+'/ollama-darwin.tgz', temporary)
        temporary.rename(archive)
    with archive.open('rb') as f:
        if hashlib.file_digest(f, 'sha256').hexdigest() != SHA256:
            raise SystemExit('Ollama archive checksum mismatch')
    bundle = root / 'bundle'
    bundle.mkdir(exist_ok=True)
    with tarfile.open(archive) as tar:
        tar.extractall(bundle, filter='data')
    source = pathlib.Path(__file__).resolve().parent.parent
    subprocess.run(['clang', '-O2', '-Wall', '-Wextra', '-Werror', '-mmacosx-version-min=14.0', str(source/'native/local-llm/supervisor.c'), '-o', str(bundle/'h-llm-supervisor')], check=True)
    # Keep the exact upstream license alongside the runtime.
    license_file = bundle/'LICENSE-ollama.txt'
    urllib.request.urlretrieve('https://raw.githubusercontent.com/ollama/ollama/v'+VERSION+'/LICENSE', license_file)
    (bundle/'runtime.json').write_text(json.dumps({'version':VERSION, 'archive_sha256':SHA256}))
    data = json.loads(config.read_text()) if config.exists() else {}
    data.setdefault('bundle',{}).setdefault('resources',{})[str(bundle)+'/']='local-llm/'
    config.write_text(json.dumps(data,indent=2)+'\n')
if __name__ == '__main__':
    p=argparse.ArgumentParser(); p.add_argument('--root',type=pathlib.Path,default=pathlib.Path.home()/'.local/share/h-opentypeless/local-llm-runtime'); p.add_argument('--config',type=pathlib.Path,required=True)
    a=p.parse_args(); prepare(a.root,a.config)

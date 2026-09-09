#!/usr/bin/env python3
"""Build pinned native STT binaries outside the checkout. Python is build-only.
Windows bundles Whisper; Qwen is gated until its POSIX runtime is ported/tested.
"""
import argparse
import json
import os
import pathlib
import platform
import shutil
import subprocess

WHISPER = '2eeeba56e9edd762b4b38467bab96c2517163158' # v1.8.3
QWEN = 'b00b789b17051aea61e9717458171100662318a4'

def run(*args, cwd=None, env=None):
    subprocess.run(args, cwd=cwd, env=env, check=True)

def prepare(root, config):
    root.mkdir(parents=True, exist_ok=True)
    out = root / 'bundle'
    out.mkdir(exist_ok=True)
    whisper = root / 'whisper'
    qwen = root / 'qwen'
    for repo, path, rev in [('ggml-org/whisper.cpp', whisper, WHISPER), ('antirez/qwen-asr', qwen, QWEN)]:
        if platform.system() == 'Windows' and path == qwen:
            continue
        if not (path / '.git').exists():
            run('git', 'clone', 'https://github.com/' + repo + '.git', str(path))
        run('git', 'checkout', '--detach', rev, cwd=path)
    mac_args = ['-DCMAKE_OSX_DEPLOYMENT_TARGET=11.0'] if platform.system() == 'Darwin' else []
    run('cmake', '-S', str(whisper), '-B', str(whisper / 'build'), '-DBUILD_SHARED_LIBS=OFF', '-DGGML_METAL=OFF', '-DGGML_OPENMP=OFF', '-DGGML_NATIVE=OFF', '-DWHISPER_BUILD_TESTS=OFF', '-DWHISPER_BUILD_SERVER=OFF', '-DCMAKE_BUILD_TYPE=Release', *mac_args)
    run('cmake', '--build', str(whisper / 'build'), '--config', 'Release', '--target', 'whisper-cli', '-j', '4')
    exe = 'whisper-cli.exe' if platform.system() == 'Windows' else 'whisper-cli'
    candidates = [whisper / 'build/bin' / exe, whisper / 'build/bin/Release' / exe]
    shutil.copy2(next(p for p in candidates if p.exists()), out / exe)
    shutil.copy2(whisper / 'LICENSE', out / 'LICENSE-whisper.txt')
    if platform.system() != 'Windows':
        engine_env = os.environ.copy()
        if platform.system() == 'Darwin':
            engine_env['MACOSX_DEPLOYMENT_TARGET'] = '13.3'
        run('make', 'blas', 'CFLAGS_BASE=-Wall -Wextra -O3 -ffast-math', cwd=qwen, env=engine_env)
        shutil.copy2(qwen / 'qwen_asr', out / 'qwen_asr')
        shutil.copy2(qwen / 'LICENSE', out / 'LICENSE-qwen-asr.txt')
    notices = pathlib.Path(__file__).resolve().parent.parent / 'docs/fork/LOCAL_STT_NOTICES.md'
    shutil.copy2(notices, out / notices.name)
    # No model weights, credentials or build caches enter the application bundle.
    files = [out / exe, out / 'LICENSE-whisper.txt', out / notices.name]
    if platform.system() != 'Windows':
        files += [out / 'qwen_asr', out / 'LICENSE-qwen-asr.txt']
    config.write_text(json.dumps({'bundle': {'resources': {str(p.resolve()): 'local-stt/' + p.name for p in files}}}, indent=2) + '\n')

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=pathlib.Path, default=pathlib.Path.home() / '.local/share/h-opentypeless/local-stt-build')
    parser.add_argument('--config', type=pathlib.Path, required=True)
    args = parser.parse_args()
    prepare(args.root, args.config)

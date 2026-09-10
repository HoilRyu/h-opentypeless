#!/usr/bin/env python3
"""Explicit orphan-process check for the private worker; does not record audio."""
import os
import pathlib
import subprocess
import sys
import tempfile
import time

runtime = pathlib.Path(sys.argv[1]).resolve()
with tempfile.TemporaryDirectory(prefix='h-mlx-parent-') as directory:
    pidfile = pathlib.Path(directory)/'pid'
    parent_script = '''import pathlib, subprocess, sys, time, json, struct
p=subprocess.Popen([sys.argv[1], '-I', '-B', sys.argv[2]], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
pathlib.Path(sys.argv[3]).write_text(str(p.pid))
if len(sys.argv) > 4:
    model = pathlib.Path(sys.argv[4])
    pcm = bytes([1, 0]) * 16000 * 120
    header = json.dumps(dict(id=1, model=str(model), pcm_bytes=len(pcm), language='Korean', weights=sorted(p.name for p in model.glob('*.safetensors')))).encode()
    p.stdin.write(struct.pack('>I', len(header)) + header + pcm)
    p.stdin.flush()
    time.sleep(0.5)
else:
    time.sleep(2)
# Exit while a request may be loading/inferencing, without waiting for its result.
'''
    subprocess.run([sys.executable, '-c', parent_script, str(runtime/'bin/python3'), str(runtime/'worker.py'), str(pidfile), *sys.argv[2:]], check=True, timeout=10)
    pid = int(pidfile.read_text())
    for _ in range(50):
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            print('PASS: worker exited and was reaped after parent exit')
            break
        time.sleep(0.1)
    else:
        os.kill(pid, 9)
        raise RuntimeError('Worker survived parent exit')

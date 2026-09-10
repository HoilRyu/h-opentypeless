#!/usr/bin/env python3
import pathlib
import subprocess
import sys
root = pathlib.Path(sys.argv[1])
magic = {bytes.fromhex(s) for s in ('feedface', 'feedfacf', 'cefaedfe', 'cffaedfe', 'cafebabe', 'bebafeca')}
if root.exists():
    for path in sorted(root.rglob('*'), key=lambda p: len(p.parts), reverse=True):
        if path.is_symlink() or not path.is_file():
            continue
        with path.open('rb') as stream:
            native = stream.read(4) in magic
        if native:
            subprocess.run(['codesign', '--force', '--sign', sys.argv[2], '--timestamp=none', str(path)], check=True)

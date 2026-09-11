import os
import pathlib
import platform
import shlex
import signal
import subprocess
import sys
import tempfile
import time
import unittest

@unittest.skipUnless(platform.system() == 'Darwin', 'macOS supervisor')
class SupervisorTests(unittest.TestCase):
    def test_parent_death_and_explicit_shutdown_stop_descendants(self):
        source = pathlib.Path(__file__).resolve().parents[2] / 'native/local-llm/supervisor.c'
        with tempfile.TemporaryDirectory(prefix='h-llm-lifetime-') as directory:
            root = pathlib.Path(directory)
            binary = root/'supervisor'
            subprocess.run(['clang','-Wall','-Wextra','-Werror',str(source),'-o',str(binary)],check=True)
            for parent_death in [False, True]:
                record = root/('pids-'+str(parent_death))
                engine = root/'engine'
                engine.write_text('#!/bin/sh\nsleep 300 &\necho "$$ $!" > '+shlex.quote(str(record))+'\nwait\n')
                engine.chmod(0o755)
                if parent_death:
                    script = 'import os,subprocess,sys,time; p=subprocess.Popen([sys.argv[1],str(os.getpid()),sys.argv[2]]); print(p.pid,flush=True); time.sleep(300)'
                    owner = subprocess.Popen([sys.executable,'-c',script,str(binary),str(engine)],stdout=subprocess.PIPE,text=True)
                    supervisor_pid = int(owner.stdout.readline())
                    owner.stdout.close()
                else:
                    owner = subprocess.Popen([str(binary),str(os.getpid()),str(engine)])
                    supervisor_pid = owner.pid
                try:
                    deadline = time.monotonic()+5
                    while not record.exists() and time.monotonic()<deadline: time.sleep(.05)
                    self.assertTrue(record.exists())
                    pids = [int(p) for p in record.read_text().split()]
                    os.kill(owner.pid if parent_death else supervisor_pid, signal.SIGKILL if parent_death else signal.SIGTERM)
                    owner.wait(timeout=5)
                    deadline = time.monotonic()+5
                    while time.monotonic()<deadline:
                        alive = []
                        for pid in pids:
                            state = subprocess.run(['ps','-p',str(pid),'-o','stat='],capture_output=True,text=True).stdout.strip()
                            if state and not state.startswith('Z'): alive.append(pid)
                        if not alive: break
                        time.sleep(.1)
                    self.assertEqual(alive, [])
                finally:
                    if owner.poll() is None: owner.kill(); owner.wait()
                    for pid in [supervisor_pid]+([int(p) for p in record.read_text().split()] if record.exists() else []):
                        try: os.kill(pid,signal.SIGTERM)
                        except ProcessLookupError: pass
if __name__ == '__main__': unittest.main()

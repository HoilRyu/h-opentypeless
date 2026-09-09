#!/usr/bin/env python3
"""Create a persistent local Android release key without printing passwords."""
import os
import pathlib
import secrets
import shutil
import subprocess

root = pathlib.Path.home() / '.local/share/h-opentypeless/signing/android'
root.mkdir(parents=True, exist_ok=True, mode=0o700)
root.chmod(0o700)
props = root / 'release-signing.properties'
key = root / 'release.p12'
if props.exists():
    if not key.is_file():
        raise SystemExit('Signing configuration exists but its keystore is missing. Restore the backup.')
    print('Existing Android release key preserved.')
    raise SystemExit(0)
if key.exists():
    raise SystemExit('Keystore exists without configuration. Restore configuration; refusing to replace key.')
java_home = os.environ.get('JAVA_HOME')
keytool = str(pathlib.Path(java_home) / 'bin/keytool') if java_home else shutil.which('keytool')
if not keytool:
    raise SystemExit('Set JAVA_HOME to a JDK containing keytool.')
password = secrets.token_urlsafe(36)
env = dict(os.environ, H_ANDROID_KEY_PASSWORD=password)
os.umask(0o077)
subprocess.run([keytool, '-genkeypair', '-keystore', str(key), '-storetype', 'PKCS12',
                '-storepass:env', 'H_ANDROID_KEY_PASSWORD', '-keypass:env', 'H_ANDROID_KEY_PASSWORD',
                '-alias', 'h-opentypeless', '-keyalg', 'RSA', '-keysize', '3072', '-validity', '10000',
                '-dname', 'CN=H-OpenTypeless, OU=Mobile, O=H-OpenTypeless', '-noprompt'],
               env=env, check=True)
with props.open('x') as out:
    out.write(f'storeFile={key}\nstorePassword={password}\nkeyAlias=h-opentypeless\nkeyPassword={password}\n')
key.chmod(0o600)
props.chmod(0o600)
print('Android release key created outside the repository. Back up the signing/android directory securely.')

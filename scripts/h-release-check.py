#!/usr/bin/env python3
"""Static release invariants, independent of local signing credentials."""
import json
import pathlib
import re

root = pathlib.Path(__file__).resolve().parents[1]
load = lambda name: json.loads((root / name).read_text())
version = load('package.json')['version']
versions = [load('package-lock.json')['version'], load('package-lock.json')['packages']['']['version'], load('src-tauri/tauri.conf.json')['version']]
versions += [re.search(r'^version = "([^"]+)"', (root / 'src-tauri/Cargo.toml').read_text(), re.M)[1]]
versions += [re.search(r'name = "opentypeless"\nversion = "([^"]+)"', (root / 'src-tauri/Cargo.lock').read_text())[1]]
assert all(v == version for v in versions), 'Desktop versions differ'
config = load('src-tauri/tauri.conf.json')
assert config['identifier'] == 'dev.hoilryu.hopentypeless'
assert config['bundle']['macOS']['minimumSystemVersion'] == '14.0', 'Apple Silicon MLX release requires macOS 14 or later'
assert config['plugins']['updater']['endpoints'] == []
assert config['bundle']['createUpdaterArtifacts'] is False
for file in ('release.yml', 'release-windows-signpath.yml', 'staple-macos-release.yml', 'release-drafter.yml'):
    assert "if: github.repository == 'tover0314-w/opentypeless'" in (root / '.github/workflows' / file).read_text(), file
print(f'Release invariants passed: desktop {version}; upstream publication guarded; updater disabled.')

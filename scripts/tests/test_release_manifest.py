import contextlib
import importlib.util
import io
import json
import pathlib
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('manifest', pathlib.Path(__file__).parents[1] / 'h-release-manifest.py')
manifest = importlib.util.module_from_spec(spec)
spec.loader.exec_module(manifest)

class ReleaseManifestTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        for platform, extension in [('macos-arm64', 'zip'), ('android-arm64', 'apk')]:
            package = self.root / ('H_' + platform + '.' + extension)
            package.write_bytes(b'package fixture')
            data = dict(file=package.name, sha256=manifest.digest(package), platform=platform,
                        version='1.0.0', source_commit='abc', source_dirty=False)
            package.with_name(package.name + '.json').write_text(json.dumps(data))
        self.update_checksums()

    def update_checksums(self):
        records = [json.loads(p.read_text()) for p in sorted(self.root.glob('*.json'))]
        (self.root / 'SHA256SUMS').write_text(''.join(f"{r['sha256']}  {r['file']}\n" for r in records))

    def edit_record(self, **updates):
        path = next(self.root.glob('*.json'))
        data = json.loads(path.read_text()); data.update(updates)
        path.write_text(json.dumps(data))
        self.update_checksums()

    def test_valid_packages(self):
        with contextlib.redirect_stdout(io.StringIO()): manifest.verify(self.root, 'abc')

    def test_dirty_build_rejected(self):
        self.edit_record(source_dirty=True)
        with self.assertRaisesRegex(ValueError, 'clean release commit'): manifest.verify(self.root, 'abc')

    def test_wrong_source_commit_rejected(self):
        with self.assertRaisesRegex(ValueError, 'clean release commit'): manifest.verify(self.root, 'def')

    def test_modified_package_rejected(self):
        next(self.root.glob('*.apk')).write_bytes(b'tampered')
        with self.assertRaisesRegex(ValueError, 'checksum mismatch'): manifest.verify(self.root, 'abc')

    def test_path_traversal_rejected(self):
        self.edit_record(file='../outside.apk')
        with self.assertRaisesRegex(ValueError, 'Unsafe'): manifest.verify(self.root, 'abc')

    def test_missing_platform_rejected(self):
        next(self.root.glob('*.apk.json')).unlink(); self.update_checksums()
        with self.assertRaisesRegex(ValueError, 'requires exactly'): manifest.verify(self.root, 'abc')

    def test_modified_checksum_list_rejected(self):
        (self.root / 'SHA256SUMS').write_text('invalid')
        with self.assertRaisesRegex(ValueError, 'SHA256SUMS'): manifest.verify(self.root, 'abc')

class SourceSnapshotTests(unittest.TestCase):
    def test_source_change_during_build_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            package = root / 'app.zip'; package.write_bytes(b'app')
            source = root / 'source.json'
            source.write_text(json.dumps(dict(source_commit='before', source_dirty=False)))
            with patch.object(manifest, 'snapshot', return_value=dict(source_commit='after', source_dirty=False)):
                with self.assertRaisesRegex(ValueError, 'Source changed during build'):
                    manifest.record(package, 'macos-arm64', '1.0.0', source)
            self.assertFalse((root / 'app.zip.json').exists())

    def test_existing_provenance_is_not_overwritten(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            package = root / 'app.zip'; package.write_bytes(b'app')
            (root / 'app.zip.json').write_text('original')
            with self.assertRaisesRegex(ValueError, 'already exists'):
                manifest.record(package, 'macos-arm64', '1.0.0')
            self.assertEqual((root / 'app.zip.json').read_text(), 'original')

if __name__ == '__main__': unittest.main()

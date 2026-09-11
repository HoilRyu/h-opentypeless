import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('local_llm_packaging', Path(__file__).parents[1] / 'h-prepare-local-llm.py')
packaging = importlib.util.module_from_spec(spec)
spec.loader.exec_module(packaging)

class PackagingTests(unittest.TestCase):
    def test_remove_archive_metadata_without_removing_real_resources(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            backend = root / 'mlx_metal_v4'
            backend.mkdir()
            (backend / '._mlx.metallib').write_bytes(bytes.fromhex('00051607') + b'metadata')
            (backend / 'mlx.metallib').write_bytes(b'metal shader')
            (root / '._ordinary').write_bytes(b'ordinary resource')
            (root / '._link').symlink_to(backend / 'mlx.metallib')
            packaging.remove_appledouble(root)
            packaging.remove_appledouble(root)
            self.assertFalse((backend / '._mlx.metallib').exists())
            self.assertEqual((backend / 'mlx.metallib').read_bytes(), b'metal shader')
            self.assertEqual((root / '._ordinary').read_bytes(), b'ordinary resource')
            self.assertTrue((root / '._link').is_symlink())

if __name__ == '__main__': unittest.main()

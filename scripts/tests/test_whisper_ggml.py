"""Bounded GGML parsing and MLX tensor-layout regression tests."""
import importlib.util
from pathlib import Path
import struct
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('whisper_session', Path(__file__).resolve().parents[2] / 'native/mlx-stt/whisper_session.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def header(kind=1):
    return (struct.pack('<14i', 0x67676D6C, 51865, 1500, 384, 6, 4, 448, 384, 6, 4, 80, kind, 80, 201)
            + bytes(80 * 201 * 4) + struct.pack('<ii', 1, 1) + b'a')


def tensor(name, shape, values):
    name = name.encode()
    return (struct.pack('<3i', len(shape), len(name), 0)
            + struct.pack('<' + 'i' * len(shape), *reversed(shape)) + name
            + struct.pack('<' + 'f' * len(values), *values))


class GgmlTests(unittest.TestCase):
    def parse(self, data):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / 'model.bin'
            path.write_bytes(data)
            return list(module.read_ggml(path))

    def test_conv_layout_and_mlp_names(self):
        items = self.parse(header() + tensor('encoder.conv1.weight', (2, 2, 3), list(range(12)))
                           + tensor('encoder.conv1.bias', (2, 1), [1, 2])
                           + tensor('decoder.blocks.0.mlp.0.bias', (2,), [3, 4]))
        self.assertEqual(items[1][1].shape, (2, 3, 2))
        self.assertEqual(items[1][1][0].tolist(), [[0, 3], [1, 4], [2, 5]])
        self.assertEqual(items[2][1].shape, (2,))
        self.assertEqual(items[3][0], 'decoder.blocks.0.mlp1.bias')

    def test_quantized_truncated_duplicate_and_oversized_inputs(self):
        t = tensor('encoder.conv1.bias', (1,), [1])
        for data in (header(2), header()[:-1], header() + t[:-1], header() + t + t,
                     header() + struct.pack('<3i', 4, 10, 0)):
            with self.subTest(size=len(data)), self.assertRaises(ValueError):
                self.parse(data)


if __name__ == '__main__':
    unittest.main()

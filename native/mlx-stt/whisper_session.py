"""Offline MLX adapter for the catalog's unquantized whisper.cpp GGML weights.

Format: ggml-org/whisper.cpp models/convert-pt-to-ggml.py.
Tensor mapping: ml-explore/mlx-examples whisper/convert.py.
No converted weight cache or additional model download is needed.
"""
import importlib
import math
import mmap
import struct


def read_ggml(path):
    """Yield dimensions then bounded F16/F32 tensors, copying out of the mapping."""
    import numpy as np
    with open(path, 'rb') as source, mmap.mmap(source.fileno(), 0, access=mmap.ACCESS_READ) as data:
        cursor = 0

        def take(size):
            nonlocal cursor
            if size < 0 or cursor + size > len(data):
                raise ValueError('truncated GGML')
            start = cursor
            cursor += size
            return start

        def integers(count):
            return struct.unpack_from('<' + 'i' * count, data, take(count * 4))

        if integers(1)[0] != 0x67676D6C:
            raise ValueError('unsupported GGML magic')
        keys = ('n_vocab', 'n_audio_ctx', 'n_audio_state', 'n_audio_head', 'n_audio_layer',
                'n_text_ctx', 'n_text_state', 'n_text_head', 'n_text_layer', 'n_mels')
        values = integers(11)
        if values[-1] not in (0, 1):
            raise ValueError('quantized GGML is not supported')
        dims = dict(zip(keys, values))
        if (dims['n_vocab'] not in (51864, 51865, 51866) or dims['n_audio_ctx'] != 1500
                or dims['n_text_ctx'] != 448 or dims['n_mels'] not in (80, 128)
                or any(not 0 < dims[k] <= limit for k, limit in (
                    ('n_audio_state', 1280), ('n_text_state', 1280),
                    ('n_audio_head', 20), ('n_text_head', 20),
                    ('n_audio_layer', 32), ('n_text_layer', 32)))
                or dims['n_audio_state'] % dims['n_audio_head']
                or dims['n_text_state'] % dims['n_text_head']):
            raise ValueError('unsupported Whisper dimensions')
        mel, fft = integers(2)
        if mel != dims['n_mels'] or fft != 201:
            raise ValueError('invalid mel filters')
        take(mel * fft * 4)
        vocab = integers(1)[0]
        if not 0 < vocab <= dims['n_vocab']:
            raise ValueError('invalid vocabulary')
        for _ in range(vocab):
            size = integers(1)[0]
            if not 0 <= size <= 65536:
                raise ValueError('invalid token length')
            take(size)
        yield dims
        names = set()
        while cursor < len(data):
            rank, name_size, kind = integers(3)
            if not 1 <= rank <= 3 or not 0 < name_size <= 256 or kind not in (0, 1):
                raise ValueError('unsupported tensor')
            shape = tuple(reversed(integers(rank)))
            if any(n <= 0 for n in shape) or math.prod(shape) > 51866 * 1280:
                raise ValueError('invalid tensor shape')
            offset = take(name_size)
            name = data[offset:offset + name_size].decode('utf-8')
            if name in names or len(names) >= 2000:
                raise ValueError('duplicate or excessive tensors')
            names.add(name)
            dtype = np.dtype('<f4' if kind == 0 else '<f2')
            offset = take(math.prod(shape) * dtype.itemsize)
            value = np.frombuffer(data, dtype=dtype, count=math.prod(shape), offset=offset).reshape(shape).copy()
            if name == 'encoder.positional_embedding':
                continue  # MLX computes the same sinusoidal positions.
            name = name.replace('.mlp.0.', '.mlp1.').replace('.mlp.2.', '.mlp2.')
            if name in ('encoder.conv1.bias', 'encoder.conv2.bias'):
                value = value.reshape(-1)
            if 'conv' in name and value.ndim == 3:
                value = value.swapaxes(1, 2)
            yield name, value


class Session:
    def __init__(self, path):
        import mlx.core as mx
        from mlx.utils import tree_flatten
        from mlx_whisper.whisper import ModelDimensions, Whisper
        self.api = importlib.import_module('mlx_whisper.transcribe')
        self.key = str(path)
        tensors = read_ggml(path)
        try:
            self.model = Whisper(ModelDimensions(**next(tensors)), mx.float16)
            expected = {k: v.shape for k, v in tree_flatten(self.model.parameters())
                        if k not in ('encoder.positional_embedding', 'alignment_heads')}
            weights = []
            for name, value in tensors:
                if expected.pop(name, None) != value.shape:
                    raise ValueError('unexpected Whisper tensor or shape')
                weights.append((name, mx.array(value).astype(mx.float16)))
            if expected:
                raise ValueError('missing Whisper tensors')
            self.model.load_weights(weights, strict=False)
            mx.eval(self.model.parameters())
        finally:
            tensors.close()
        # The pinned mlx-whisper API retains exactly one model between requests.
        self.api.ModelHolder.model = self.model
        self.api.ModelHolder.model_path = self.key

    def transcribe(self, audio, language):
        return self.api.transcribe(audio, path_or_hf_repo=self.key, language=language,
                                   verbose=None, temperature=0.0, word_timestamps=False)['text']

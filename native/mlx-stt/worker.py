"""Private bounded PCM protocol. No HTTP, microphone access or remote model loading."""
import json
import os
import pathlib
import struct
import sys
import threading
import time

MAX_PCM = 120 * 16000 * 2
MAX_JSON = 65536
protocol = sys.stdout.buffer
sys.stdout = sys.stderr
parent = os.getppid()
last_activity = time.monotonic()
busy = False


def watchdog():
    import ctypes
    libc = ctypes.CDLL(None)
    while True:
        time.sleep(1)
        if os.getppid() != parent:
            os._exit(0)
        age = time.monotonic() - last_activity
        if age > (100 if busy else 120):
            os._exit(0)
        if not busy:
            level = ctypes.c_int(0)
            length = ctypes.c_size_t(ctypes.sizeof(level))
            if libc.sysctlbyname(b'kern.memorystatus_vm_pressure_level', ctypes.byref(level), ctypes.byref(length), None, 0) == 0 and level.value >= 2:
                os._exit(0)


def read_exact(n):
    data = sys.stdin.buffer.read(n)
    if len(data) != n:
        raise EOFError()
    return data


def reply(value):
    data = json.dumps(value, ensure_ascii=False).encode()
    if len(data) > MAX_JSON:
        raise ValueError('output limit')
    protocol.write(struct.pack('>I', len(data)) + data)
    protocol.flush()


def main():
    global busy, last_activity
    import mlx.core as mx
    import numpy as np
    if not mx.metal.is_available():
        raise RuntimeError('Metal unavailable')
    mx.set_default_device(mx.gpu)
    mx.eval(mx.ones((2, 2)) @ mx.ones((2, 2)))
    # Bound transient allocator caching; model tensors are retained by Session.
    mx.set_cache_limit(256 * 1024 * 1024)
    budget = int(mx.metal.device_info()['max_recommended_working_set_size'] * 0.65)
    mx.set_memory_limit(budget)
    mx.set_wired_limit(min(budget, 4 * 1024 ** 3))
    if sys.argv[1:] == ['--probe']:
        reply({'ok': True, 'backend': 'mlx'})
        return
    threading.Thread(target=watchdog, daemon=True).start()
    sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
    session = None
    model_path = None
    while True:
        length = struct.unpack('>I', read_exact(4))[0]
        if length > MAX_JSON:
            raise ValueError('header limit')
        request = json.loads(read_exact(length))
        size = request.get('pcm_bytes', 0)
        if not isinstance(size, int) or size < 0 or size > MAX_PCM or size % 2:
            raise ValueError('PCM limit')
        pcm = read_exact(size)
        busy = True
        last_activity = time.monotonic()
        started = last_activity
        try:
            path = pathlib.Path(request['model']).resolve(strict=True)
            engine = request.get('engine', 'qwen')
            if engine not in ('qwen', 'whisper') or not path.is_dir():
                raise ValueError('unsupported local model')
            if engine == 'qwen':
                if not (path / 'tokenizer_config.json').is_file():
                    raise ValueError('local model incomplete')
                if sorted(p.name for p in path.glob('*.safetensors')) != sorted(request['weights']):
                    raise ValueError('unexpected weight files')
            else:
                weights = request['weights']
                if len(weights) != 1 or pathlib.Path(weights[0]).name != weights[0] or not weights[0].endswith('.bin'):
                    raise ValueError('invalid Whisper weights')
            if model_path != (engine, str(path)):
                if session is not None:
                    raise ValueError('worker model changed')
                mx.clear_cache()
                if engine == 'qwen':
                    from mlx_qwen3_asr import Session
                    session = Session(model=str(path), dtype=mx.float16)
                else:
                    from whisper_session import Session
                    session = Session(path / weights[0])
                model_path = (engine, str(path))
            loaded = time.monotonic()
            text = ''
            if size:
                audio = np.frombuffer(pcm, dtype='<i2').astype(np.float32) / 32768.0
                if engine == 'qwen':
                    text = session.transcribe((audio, 16000), language=request.get('language')).text
                else:
                    text = session.transcribe(audio, request.get('language'))
                del audio
            reply({'ok': True, 'id': request['id'], 'text': text,
                   'load_ms': round((loaded-started)*1000),
                   'inference_ms': round((time.monotonic()-loaded)*1000),
                   'active_bytes': mx.get_active_memory()})
        except Exception as exc:
            # Do not echo model content, paths or potentially sensitive payloads.
            reply({'ok': False, 'id': request.get('id'), 'error': type(exc).__name__})
            return
        finally:
            pcm = b''
            last_activity = time.monotonic()
            busy = False


if __name__ == '__main__':
    try:
        main()
    except EOFError:
        pass
    except Exception:
        sys.exit(1)

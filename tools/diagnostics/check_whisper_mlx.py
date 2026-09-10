#!/usr/bin/env python3
"""Offline repeated Whisper inference using explicitly supplied test PCM."""
import argparse
import json
import os
from pathlib import Path
import select
import struct
import subprocess
import time


def receive(stream, size):
    data = b''
    deadline = time.monotonic() + 90
    while len(data) < size:
        if not select.select([stream], [], [], max(0, deadline - time.monotonic()))[0]:
            raise TimeoutError('MLX response')
        chunk = os.read(stream.fileno(), size - len(data))
        if not chunk:
            raise RuntimeError('MLX worker exited')
        data += chunk
    return data


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--model', type=Path, required=True)
    parser.add_argument('--pcm', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--rounds', type=int, default=10)
    args = parser.parse_args()
    pcm = args.pcm.read_bytes() + bytes(32000)
    worker = subprocess.Popen([str(args.runtime/'bin/python3'), '-I', '-B', str(args.runtime/'worker.py')],
                              stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                              env=dict(os.environ, HF_HUB_OFFLINE='1', HF_HUB_DISABLE_TELEMETRY='1'))
    results = []
    try:
        for i in range(args.rounds + 1):
            audio = b'' if i == 0 else pcm
            header = json.dumps(dict(id=i, engine='whisper', model=str(args.model),
                                     weights=[p.name for p in args.model.glob('*.bin')],
                                     pcm_bytes=len(audio), language='ko')).encode()
            start = time.monotonic()
            worker.stdin.write(struct.pack('>I', len(header)) + header + audio)
            worker.stdin.flush()
            size = struct.unpack('>I', receive(worker.stdout, 4))[0]
            assert size <= 65536
            result = json.loads(receive(worker.stdout, size))
            result.update(wall_ms=round((time.monotonic()-start)*1000), audio_ms=len(audio)//32)
            results.append(result)
            print(json.dumps(result, ensure_ascii=False), flush=True)
            assert result['ok'] and result['id'] == i, result
            if i:
                assert result['text'].strip()
    finally:
        worker.stdin.close()
        try:
            worker.wait(timeout=5)
        except subprocess.TimeoutExpired:
            worker.kill()
            worker.wait()
        args.output.write_text(json.dumps(results, ensure_ascii=False, indent=2))
        diagnostic = worker.stderr.read().decode(errors='replace')
        if diagnostic:
            print(diagnostic[-4000:])


if __name__ == '__main__':
    main()

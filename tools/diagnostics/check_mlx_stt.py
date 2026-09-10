#!/usr/bin/env python3
"""Explicit local-only synthetic inference benchmark. Never captures the microphone."""
import argparse
import json
import os
import pathlib
import select
import shutil
import statistics
import struct
import subprocess
import tempfile
import time
import wave


def receive(stream, count):
    result = b''
    deadline = time.monotonic() + 90
    while len(result) < count:
        if not select.select([stream], [], [], max(0, deadline-time.monotonic()))[0]:
            raise TimeoutError('worker reply')
        chunk = os.read(stream.fileno(), count-len(result))
        if not chunk:
            raise RuntimeError('worker exited')
        result += chunk
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--runtime', type=pathlib.Path, required=True)
    parser.add_argument('--model', type=pathlib.Path, required=True)
    parser.add_argument('--cpu', type=pathlib.Path, required=True)
    parser.add_argument('--output', type=pathlib.Path, required=True)
    parser.add_argument('--idle-check', action='store_true')
    args = parser.parse_args()
    env = dict(os.environ, HF_HUB_OFFLINE='1', VECLIB_MAXIMUM_THREADS='4', OPENBLAS_NUM_THREADS='4', OMP_NUM_THREADS='4', QWEN_BF16_CACHE_MB='256')
    report = {'model': args.model.name, 'synthetic': True, 'mlx': [], 'cpu': []}
    with tempfile.TemporaryDirectory(prefix='H MLX 한국어 ') as work:
        work = pathlib.Path(work)
        sentences = ['설정 화면에 취소 버튼을 추가해 주세요.',
                     '안드로이드 앱에서 녹음을 취소하면 서버 요청도 중단해 주세요. 연결이 끊어졌을 때 오류를 보여 주고 다시 시도할 수 있도록 수정해 주세요.',
                     '이 프로젝트는 맥과 윈도우 그리고 리눅스에서 사용할 예정입니다. 기존 설정을 보존하면서 음성 인식 엔진을 변경하고, 모델을 다운로드한 후 인터넷 없이도 실행할 수 있도록 만들어 주세요. 반복 사용 후 메모리가 계속 늘어나지 않는지도 확인해 주세요.']
        samples = []
        for i, sentence in enumerate(sentences):
            aiff, wav = work/f'{i}.aiff', work/f'{i}.wav'
            subprocess.run(['say', '-v', 'Yuna', '-r', '180', '-o', str(aiff), sentence], check=True)
            subprocess.run(['afconvert', str(aiff), str(wav), '-f', 'WAVE', '-d', 'LEI16@16000', '-c', '1'], check=True)
            with wave.open(str(wav)) as reader:
                samples.append((wav, reader.readframes(reader.getnframes())))
        shutil.copy2(args.runtime/'tokenizers'/f'{args.model.name}.json', args.model/'tokenizer_config.json')
        command = [str(args.runtime/'bin/python3'), '-I', '-B', str(args.runtime/'worker.py')]
        worker = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, env=env)
        try:
            for i in range(26):
                sample = 0 if i < 20 else (1 if i < 23 else 2)
                _, pcm = samples[sample]
                header = json.dumps(dict(id=i, model=str(args.model), pcm_bytes=len(pcm), language='Korean', weights=sorted(p.name for p in args.model.glob('*.safetensors')))).encode()
                start = time.monotonic()
                worker.stdin.write(struct.pack('>I', len(header))+header+pcm); worker.stdin.flush()
                size = struct.unpack('>I', receive(worker.stdout, 4))[0]
                assert size <= 65536
                reply = json.loads(receive(worker.stdout, size))
                assert reply['ok'] and reply['id'] == i, reply
                reply.update(sample=sample, wall_seconds=time.monotonic()-start)
                report['mlx'].append(reply)
                if i in (0,19,25): print(json.dumps(reply, ensure_ascii=False), flush=True)
            if args.idle_check:
                started = time.monotonic()
                worker.wait(timeout=130)
                report['idle_exit_seconds'] = time.monotonic()-started
                print('idle worker exited', flush=True)
        finally:
            if worker.poll() is None: worker.kill()
            worker.wait()
        for sample, (wav, _) in enumerate(samples):
            for i in range(3):
                start = time.monotonic()
                result = subprocess.run([str(args.cpu), '-d', str(args.model), '--stdin', '--silent', '-t', '4', '-S', '20', '--language', 'Korean'], input=wav.read_bytes(), capture_output=True, check=True, timeout=90, env=env)
                report['cpu'].append(dict(sample=sample, wall_seconds=time.monotonic()-start, text=result.stdout.decode().strip()))
        report['warm_short_median_seconds'] = statistics.median(r['wall_seconds'] for r in report['mlx'][1:20])
        report['warm_short_gpu_bytes'] = [r['active_bytes'] for r in report['mlx'][1:20]]
        args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2))
        print('Report:', args.output, flush=True)


if __name__ == '__main__':
    main()

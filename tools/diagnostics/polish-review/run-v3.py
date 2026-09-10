"""Frozen local comparison inputs; never changes production settings or prompts."""
import argparse
import hashlib
import json
import time
import urllib.request
from pathlib import Path

root = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--repeat', type=int, default=1)
parser.add_argument('--ids', nargs='+')
parser.add_argument('--output', default='results-v3.json')
args = parser.parse_args()
cases = json.loads((root / 'cases.json').read_text())
if args.ids:
    cases = [case for case in cases if case['id'] in args.ids]
prompt = (root / 'candidate-v3.txt').read_text()
report = {'model': 'gemma4:12b', 'temperature': 0.3, 'reasoning_effort': 'none',
          'prompt_sha256': hashlib.sha256(prompt.encode()).hexdigest(), 'results': []}
for repeat in range(args.repeat):
    for case in cases:
        body = {'model': report['model'], 'messages': [
            {'role': 'system', 'content': prompt},
            {'role': 'user', 'content': 'Edit this transcript; output only edited text:\n<transcription>' + case['input'] + '</transcription>'}],
            'temperature': 0.3, 'max_tokens': 1500, 'stream': False, 'reasoning_effort': 'none'}
        start = time.monotonic()
        request = urllib.request.Request('http://127.0.0.1:11434/v1/chat/completions', data=json.dumps(body).encode(), headers={'Content-Type': 'application/json'})
        with urllib.request.urlopen(request, timeout=90) as response:
            data = json.load(response)
        row = {'id': case['id'], 'repeat': repeat + 1, 'input': case['input'], 'seconds': round(time.monotonic() - start, 2),
               'output': data['choices'][0]['message']['content'], 'finish_reason': data['choices'][0]['finish_reason']}
        report['results'].append(row)
        (root / args.output).write_text(json.dumps(report, ensure_ascii=False, indent=2))
        print(json.dumps(row, ensure_ascii=False), flush=True)

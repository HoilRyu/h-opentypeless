"""Manual local evaluation; no app settings, history, or production prompt changes."""
import json
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent
import sys
variants = ("candidate-v2",) if "--v2" in sys.argv else ("baseline", "candidate")
output_name = "results-v2.json" if "--v2" in sys.argv else "results.json"
results = []
for case in json.loads((ROOT / 'cases.json').read_text()):
    for variant in variants:
        body = {'model': 'gemma4:12b', 'messages': [
            {'role': 'system', 'content': (ROOT / f'{variant}.txt').read_text()},
            {'role': 'user', 'content': 'Edit this transcript; output only edited text:\n<transcription>' + case['input'] + '</transcription>'},
        ], 'temperature': 0.3, 'max_tokens': 1500, 'stream': False, 'reasoning_effort': 'none'}
        start = time.monotonic()
        request = urllib.request.Request('http://127.0.0.1:11434/v1/chat/completions', data=json.dumps(body).encode(), headers={'Content-Type': 'application/json'})
        with urllib.request.urlopen(request, timeout=90) as response:
            data = json.load(response)
        row = {'id': case['id'], 'variant': variant, 'seconds': round(time.monotonic() - start, 2), 'output': data['choices'][0]['message']['content'], 'finish_reason': data['choices'][0]['finish_reason']}
        results.append(row)
        (ROOT / output_name).write_text(json.dumps(results, ensure_ascii=False, indent=2))
        print(json.dumps(row, ensure_ascii=False), flush=True)

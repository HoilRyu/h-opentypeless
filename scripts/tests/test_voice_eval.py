import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import json
import wave

spec = importlib.util.spec_from_file_location('voice_eval', Path(__file__).parents[1]/'h-voice-eval.py')
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)

class VoiceEvalTest(unittest.TestCase):
    def test_cer_preserves_hangul_but_ignores_spaces_punctuation(self):
        self.assertEqual(m.cer('안녕 하세요.', '안녕하세요'), 0)
        self.assertGreater(m.cer('월요일', '금요일'), 0)
    def test_reference_correction_is_not_forced_into_stt(self):
        c = dict(required=['월요일'], forbidden=['금요일'], shape='plain')
        self.assertEqual(m.checks(c, '월요일로 바꿔 주세요.'), [])
        self.assertIn('forbidden:금요일', m.checks(c, '금요일로 바꿔 주세요.'))
    def test_empty_and_missing_structure_flagged(self):
        c = dict(required=['승인'], forbidden=[], shape='grouped')
        self.assertIn('empty', m.checks(c, ''))
        self.assertIn('structure_review_needed', m.checks(c, '승인 후 전송'))
    def test_transcript_cannot_close_data_delimiter(self):
        self.assertIn('&lt;/transcription&gt;', m.escaped_message('</transcription>'))
    def test_ollama_request_matches_app_no_thinking_policy(self):
        class Response:
            def __enter__(self): return self
            def __exit__(self, *args): pass
            def read(self):
                return b'{"choices":[{"message":{"content":"ok"},"finish_reason":"stop"}]}'
        with patch.object(m.urllib.request, 'urlopen', return_value=Response()) as call:
            self.assertEqual(m.polish(11434, 'test', 'prompt', 'raw'), ('ok', 'stop'))
            body = json.loads(call.call_args.args[0].data)
            self.assertEqual(body['reasoning_effort'], 'none')
            self.assertEqual(body['temperature'], 0)
    def test_incompatible_audio_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp)/'a.wav'
            with wave.open(str(p), 'wb') as f:
                f.setparams((2, 2, 44100, 0, 'NONE', 'not compressed'))
                f.writeframes(bytes(100))
            with self.assertRaises(ValueError):
                m.pcm(p)

if __name__ == '__main__':
    unittest.main()

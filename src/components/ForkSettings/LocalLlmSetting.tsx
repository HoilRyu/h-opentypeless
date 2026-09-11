import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'
type Status = {
  models: { id: string; name: string; size: number; installed: boolean }[]
  selected: string | null
  available: boolean
  busy: boolean
  running: boolean
  progress: {
    model: string
    downloaded: number
    total: number
    phase: string
    error: string | null
  }
}
export function LocalLlmSetting() {
  const { i18n } = useTranslation()
  const ko = i18n.language.startsWith('ko')
  const [status, setStatus] = useState<Status | null>(null)
  const [chosen, setChosen] = useState('12b')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [latency, setLatency] = useState<number | null>(null)
  useEffect(() => {
    let live = true
    let pending = false
    const refresh = async () => {
      if (pending) return
      pending = true
      try {
        const next = await invoke<Status>('get_local_llm_status')
        if (live) setStatus(next)
      } catch (e) {
        if (live) setError(String(e))
      } finally {
        pending = false
      }
    }
    void refresh()
    const timer = setInterval(() => void refresh(), 1000)
    return () => {
      live = false
      clearInterval(timer)
    }
  }, [])
  const act = async (command: string, args?: { id: string }) => {
    setError(null)
    setBusy(true)
    setLatency(null)
    try {
      const result = await invoke<number | null>(command, args)
      if (command === 'test_local_llm') setLatency(result)
      setStatus(await invoke<Status>('get_local_llm_status'))
    } catch (e) {
      setError(String(e))
    } finally {
      setBusy(false)
    }
  }
  const model = status?.models.find((m) => m.id === chosen)
  const downloading = ['downloading', 'verifying'].includes(status?.progress.phase ?? '')
  const disabled = busy || status?.busy || !status?.available
  const button = 'rounded-lg border border-border px-3 py-2 text-[13px] disabled:opacity-40'
  return (
    <section
      className="space-y-3 rounded-xl border border-border p-4"
      aria-label={ko ? '내장 LLM 모델' : 'Built-in LLM models'}
    >
      <p className="text-sm">
        {ko
          ? 'Ollama 별도 설치 없이, 다운로드한 Gemma 4로 이 Mac에서 다듬습니다.'
          : 'Polish locally with Gemma 4. No separate Ollama installation required.'}
      </p>
      <label className="block text-sm">
        {ko ? 'LLM 모델' : 'LLM model'}
        <select
          value={chosen}
          onChange={(e) => setChosen(e.target.value)}
          className="mt-1 w-full rounded-lg border border-border bg-bg-secondary p-2"
        >
          {(
            status?.models ?? [
              { id: 'e2b', name: 'Gemma 4 E2B' },
              { id: 'e4b', name: 'Gemma 4 E4B' },
              { id: '12b', name: 'Gemma 4 12B' },
            ]
          ).map((m) => (
            <option key={m.id} value={m.id}>
              {m.name}
            </option>
          ))}
        </select>
      </label>
      {model && (
        <p className="text-xs text-text-secondary">
          {ko ? '다운로드' : 'Download'}: {(model.size / 1e9).toFixed(1)} GB ·{' '}
          {model.installed ? (ko ? '설치됨' : 'Installed') : ko ? '미설치' : 'Not installed'}
        </p>
      )}
      <p className="text-xs">
        {ko ? '사용 중인 모델' : 'Selected model'}:{' '}
        {status?.models.find((m) => m.id === status.selected)?.name ??
          (ko ? '선택되지 않음' : 'None')}
      </p>
      {status && !status.available && (
        <p role="status">
          {ko
            ? '내장 LLM 엔진이 포함된 Apple Silicon용 앱이 필요합니다.'
            : 'Requires an Apple Silicon app with the bundled LLM runtime.'}
        </p>
      )}
      <div className="flex flex-wrap gap-2">
        {!model?.installed && (
          <button
            className={button}
            disabled={disabled}
            onClick={() => void act('download_local_llm_model', { id: chosen })}
          >
            {ko ? '다운로드' : 'Download'}
          </button>
        )}
        {model?.installed && (
          <>
            <button
              className={button}
              disabled={disabled || status?.selected === chosen}
              onClick={() => void act('select_local_llm_model', { id: chosen })}
            >
              {ko ? '이 모델 사용' : 'Use this model'}
            </button>
            <button
              className={button}
              disabled={disabled}
              onClick={() => void act('delete_local_llm_model', { id: chosen })}
            >
              {ko ? '모델 삭제' : 'Delete model'}
            </button>
          </>
        )}
        <button
          className={button}
          disabled={disabled || !status?.selected}
          onClick={() => void act('test_local_llm')}
        >
          {ko ? '모델 테스트' : 'Test model'}
        </button>
        <button
          className={button}
          disabled={busy || status?.busy || !status?.running}
          onClick={() => void act('unload_local_llm')}
        >
          {ko ? '메모리 해제' : 'Unload model'}
        </button>
      </div>
      {downloading && (
        <div role="status">
          <p>
            {status?.models.find((m) => m.id === status.progress.model)?.name}:{' '}
            {status?.progress.phase === 'verifying'
              ? ko
                ? '파일 검증 중'
                : 'Verifying'
              : ko
                ? '다운로드 중'
                : 'Downloading'}
          </p>
          <progress
            className="w-full"
            max={status?.progress.total || 1}
            value={status?.progress.downloaded ?? 0}
          />
          <button
            className={button}
            onClick={() => {
              void invoke('cancel_local_llm_download').catch((e) => setError(String(e)))
            }}
          >
            {ko ? '중단 · 나중에 이어받기' : 'Pause download'}
          </button>
        </div>
      )}
      {latency !== null && (
        <p role="status">
          {ko ? '테스트 완료' : 'Test passed'} · {latency} ms
        </p>
      )}
      {(error || status?.progress.error) && (
        <p role="alert" className="text-sm text-error">
          {error || status?.progress.error}
        </p>
      )}
      <p className="text-xs text-text-secondary">
        {ko
          ? '모델 크기는 실행 메모리와 다릅니다. STT와 함께 사용할 때 메모리 사용량이 늘어납니다. 실패 시 외부 API로 자동 전환하지 않습니다.'
          : 'Download size differs from runtime memory. STT and LLM share memory. No automatic fallback to external APIs.'}
      </p>
    </section>
  )
}

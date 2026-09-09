import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'
import type { FeedbackSettings, FeedbackSnapshot } from '../VoiceFeedback'
export function VoiceFeedbackSetting() {
  const { i18n } = useTranslation()
  const ko = i18n.language.startsWith('ko')
  const [settings, setSettings] = useState<FeedbackSettings | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  useEffect(() => {
    invoke<FeedbackSnapshot>('get_voice_feedback')
      .then((s) => setSettings(s.settings))
      .catch((e) => setError(String(e)))
  }, [])
  const save = async (patch: Partial<FeedbackSettings>) => {
    if (!settings || busy) return
    setBusy(true)
    setError('')
    try {
      const result = await invoke<FeedbackSnapshot>('set_voice_feedback', {
        settings: { ...settings, ...patch },
      })
      setSettings(result.settings)
    } catch (e) {
      setError(String(e))
    } finally {
      setBusy(false)
    }
  }
  return (
    <section className="rounded-xl border border-neutral-200 dark:border-neutral-700 p-4 space-y-3">
      <h3 className="text-sm font-semibold">{ko ? '음성 입력 표시' : 'Voice input appearance'}</h3>
      <p className="text-xs text-neutral-500">
        {ko
          ? '녹음은 민트색, 처리 중에는 보라색 테두리로 표시합니다. 화면 공유에도 표시될 수 있습니다.'
          : 'Mint edges indicate recording; violet indicates processing. These effects may appear in screen sharing.'}
      </p>
      {settings && (
        <fieldset disabled={busy} className="space-y-3 text-sm">
          <label className="flex items-center justify-between gap-4">
            {ko ? '화면 그라데이션 테두리' : 'Screen gradient edges'}
            <input
              type="checkbox"
              checked={settings.enabled}
              onChange={(e) => save({ enabled: e.target.checked })}
            />
          </label>
          <label className="flex items-center justify-between gap-4">
            {ko ? '테두리 밝기' : 'Edge brightness'}
            <select
              value={settings.brightness}
              onChange={(e) => save({ brightness: Number(e.target.value) })}
              className="bg-transparent"
            >
              <option value={0.3}>{ko ? '은은하게' : 'Subtle'}</option>
              <option value={0.55}>{ko ? '보통' : 'Balanced'}</option>
              <option value={0.85}>{ko ? '선명하게' : 'Bright'}</option>
            </select>
          </label>
          <label className="flex items-center justify-between gap-4">
            {ko ? '목소리 크기에 반응' : 'React to microphone level'}
            <input
              type="checkbox"
              checked={settings.reactive}
              onChange={(e) => save({ reactive: e.target.checked })}
            />
          </label>
          <label className="flex items-center justify-between gap-4">
            {ko ? '캡슐 위치' : 'Capsule position'}
            <select
              className="bg-transparent"
              value={settings.near_caret ? 'caret' : 'bottom'}
              onChange={(e) => save({ near_caret: e.target.value === 'caret' })}
            >
              <option value="caret">{ko ? '입력창 근처' : 'Near input field'}</option>
              <option value="bottom">{ko ? '화면 하단' : 'Screen bottom'}</option>
            </select>
          </label>
          <p className="text-xs text-neutral-500">
            {ko
              ? '입력창 아래에 표시하며, 공간이 부족하면 위에 표시합니다. 위치를 읽을 수 없으면 화면 하단에 표시합니다. 녹음 중 위치는 고정됩니다.'
              : 'Appears below the input field, or above if space is limited. Falls back to the screen bottom when its position is unavailable. Position stays fixed during recording.'}
          </p>
          <button
            className="rounded-lg border px-3 py-2"
            type="button"
            disabled={!settings.enabled}
            onClick={() => invoke('preview_voice_feedback').catch((e) => setError(String(e)))}
          >
            {ko ? '테두리 3초 미리 보기' : 'Preview edges for 3 seconds'}
          </button>
        </fieldset>
      )}
      {error && (
        <p role="alert" className="text-xs text-red-500">
          {error}
        </p>
      )}
    </section>
  )
}

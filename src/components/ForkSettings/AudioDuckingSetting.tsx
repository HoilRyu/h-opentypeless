import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'
type Mode = 'off' | 'reduce' | 'mute'
type Status = { mode: Mode; volume_percent: number; active: boolean; warning: string | null }
export function AudioDuckingSetting() {
  const { i18n } = useTranslation()
  const ko = i18n.language.startsWith('ko')
  const [status, setStatus] = useState<Status | null>(null)
  const [percent, setPercent] = useState('20')
  const [error, setError] = useState(false)
  const [invalid, setInvalid] = useState(false)
  const [saving, setSaving] = useState(false)
  const editing = useRef(false)
  const version = useRef(0)
  const busy = useRef(false)
  useEffect(() => {
    let live = true
    const read = () => {
      const requestVersion = version.current
      invoke<Status>('get_audio_ducking')
        .then((value) => {
          if (live && requestVersion === version.current && !editing.current && !busy.current) {
            setStatus(value)
            setPercent(String(value.volume_percent ?? 20))
          }
        })
        .catch(() => {
          if (live) setError(true)
        })
    }
    read()
    const timer = window.setInterval(read, 2000)
    return () => {
      live = false
      window.clearInterval(timer)
    }
  }, [])
  const save = async (mode: Mode, volumePercent = status?.volume_percent ?? 20) => {
    if (busy.current) return
    version.current += 1
    busy.current = true
    setSaving(true)
    setError(false)
    try {
      const saved = await invoke<Status>('set_audio_ducking', { mode, volumePercent })
      setStatus(saved)
      setPercent(String(saved.volume_percent ?? volumePercent))
      setInvalid(false)
    } catch {
      setPercent(String(status?.volume_percent ?? 20))
      setError(true)
    } finally {
      editing.current = false
      busy.current = false
      setSaving(false)
    }
  }
  const edit = (value: string) => {
    editing.current = true
    version.current += 1
    setPercent(value)
    setInvalid(false)
  }
  const commit = () => {
    const value = Number(percent)
    if (percent.trim() === '' || !Number.isInteger(value) || value < 0 || value > 100) {
      setInvalid(true)
      return
    }
    if (value === status?.volume_percent) {
      editing.current = false
      return
    }
    void save(status?.mode ?? 'reduce', value)
  }
  return (
    <section className="space-y-2.5" aria-labelledby="h-audio-label">
      <h3
        id="h-audio-label"
        className="text-[11px] font-medium text-text-tertiary uppercase tracking-wider"
      >
        {ko ? '녹음 중 다른 소리' : 'Other audio while recording'}
      </h3>
      <select
        aria-labelledby="h-audio-label"
        value={status?.mode ?? 'off'}
        disabled={!status || saving}
        onChange={(event) => void save(event.target.value as Mode)}
        className="w-full rounded-lg border border-border bg-bg-secondary px-3 py-2 text-sm text-text-primary disabled:opacity-50"
      >
        <option value="off">{ko ? '끄기' : 'Off'}</option>
        <option value="reduce">{ko ? '소리 줄이기' : 'Reduce volume'}</option>
        <option value="mute">{ko ? '음소거' : 'Mute'}</option>
      </select>
      {status?.mode === 'reduce' && (
        <div className="rounded-lg border border-border p-3 space-y-3">
          <div className="flex items-center justify-between gap-3">
            <label htmlFor="h-audio-percent" className="text-sm text-text-secondary">
              {ko ? '기존 음량의' : 'Percentage of original volume'}
            </label>
            <div className="flex items-center gap-1.5 text-sm">
              <input
                id="h-audio-percent"
                type="number"
                min="0"
                max="100"
                step="1"
                value={percent}
                disabled={saving}
                aria-invalid={invalid}
                onChange={(event) => edit(event.target.value)}
                onBlur={commit}
                onKeyDown={(event) => {
                  if (event.key === 'Enter') event.currentTarget.blur()
                }}
                className="w-20 rounded-md border border-border bg-bg-secondary px-2 py-1 text-right text-text-primary"
              />
              <span className="text-text-tertiary">%</span>
            </div>
          </div>
          <input
            type="range"
            min="0"
            max="100"
            step="1"
            value={Number(percent) || 0}
            disabled={saving}
            aria-label={ko ? '녹음 중 음량 비율' : 'Recording volume percentage'}
            onChange={(event) => edit(event.target.value)}
            onPointerUp={commit}
            onKeyUp={commit}
            onBlur={commit}
            className="w-full accent-blue-500"
          />
          <p className="text-xs text-text-tertiary">
            {ko
              ? '0%는 무음, 100%는 현재 음량 그대로입니다.'
              : '0% is silent; 100% keeps the original volume.'}
          </p>
          {invalid && (
            <p role="alert" className="text-xs text-amber-500">
              {ko ? '0~100 사이의 정수를 입력해 주세요.' : 'Enter a whole number from 0 to 100.'}
            </p>
          )}
        </div>
      )}
      <p className="text-xs text-text-tertiary leading-relaxed">
        {ko
          ? '자동 저장 · 비율 변경은 다음 녹음부터, 끄기는 현재 녹음에도 적용됩니다. 종료 시 원래 음량으로 돌아가며 직접 바꾼 음량은 유지합니다.'
          : 'Saved automatically. Percentage changes apply next time; Off also ends current attenuation. Volume is restored when recording ends; manual adjustments are respected.'}
      </p>
      {(error || status?.warning) && (
        <p role="status" className="text-xs text-amber-500">
          {ko
            ? '음량 조절 또는 설정 저장에 실패했습니다. 오디오 장치를 확인한 뒤 앱을 다시 실행해 주세요.'
            : 'Audio control or saving settings failed. Check your audio device and restart the app.'}
        </p>
      )}
    </section>
  )
}

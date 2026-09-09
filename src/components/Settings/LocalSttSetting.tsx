import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'
import { Download, Check, HardDrive, Pause, Trash2 } from 'lucide-react'

type Model = {
  id: string
  name: string
  engine: string
  recommended_ram_gb: number
  license: string
  installed: boolean
  available: boolean
  partial_bytes: number
  files: { size: number }[]
}
type Status = {
  memory_gb?: number | null
  models: Model[]
  selected: string | null
  busy: boolean
  progress: {
    model: string
    downloaded: number
    total: number
    phase: string
    error: string | null
  }
}
const size = (bytes: number) =>
  bytes >= 1024 ** 3 ? `${(bytes / 1024 ** 3).toFixed(1)} GB` : `${Math.ceil(bytes / 1024 ** 2)} MB`
export function LocalSttSetting() {
  const { t } = useTranslation()
  const [status, setStatus] = useState<Status | null>(null)
  const [error, setError] = useState('')
  const [pending, setPending] = useState(false)
  useEffect(() => {
    let active = true
    let timer: ReturnType<typeof setTimeout>
    const poll = async () => {
      try {
        const next = await invoke<Status>('get_local_stt_status')
        if (active) setStatus(next)
      } catch (e) {
        if (active) setError(String(e))
      } finally {
        if (active) timer = setTimeout(poll, 1000)
      }
    }
    void poll()
    return () => {
      active = false
      clearTimeout(timer)
    }
  }, [])
  const run = async (command: string, id?: string) => {
    setPending(true)
    setError('')
    try {
      await invoke(command, id ? { id } : {})
      setStatus(await invoke<Status>('get_local_stt_status'))
    } catch (e) {
      setError(String(e))
    } finally {
      setPending(false)
    }
  }
  const button =
    'inline-flex items-center justify-center gap-1.5 rounded-lg border border-border px-3 py-2 text-xs transition-colors hover:bg-bg-tertiary disabled:opacity-40 disabled:cursor-not-allowed'
  return (
    <section className="space-y-3" aria-label={t('h.localStt.provider')}>
      <div className="rounded-xl border border-border bg-bg-secondary p-4 space-y-2">
        <div className="flex items-center gap-2 font-medium text-sm">
          <HardDrive size={16} />
          {t('h.localStt.title')}
        </div>
        <p className="text-xs leading-relaxed text-text-secondary">{t('h.localStt.description')}</p>
        <p className="text-xs leading-relaxed text-text-tertiary">
          {t('h.localStt.resources')}
          {status?.memory_gb ? ` ${t('h.localStt.memory', { count: status.memory_gb })}` : ''}
        </p>
      </div>
      {error && (
        <p role="alert" className="text-xs text-red-500">
          {error}
        </p>
      )}
      {!status && !error && (
        <p role="status" className="text-xs text-text-secondary">
          {t('h.localStt.loading')}
        </p>
      )}
      {status?.models.map((model) => {
        const selected = status.selected === model.id
        const progress = status.progress
        const downloading =
          progress.model === model.id && ['downloading', 'verifying'].includes(progress.phase)
        const failed = progress.model === model.id && progress.phase === 'paused'
        return (
          <article
            key={model.id}
            className={`rounded-xl border p-4 space-y-3 ${selected ? 'border-accent bg-accent/5' : 'border-border'}`}
          >
            <div className="flex items-center justify-between gap-2">
              <h3 className="text-sm font-medium">{model.name}</h3>
              {selected && (
                <span className="inline-flex items-center gap-1 text-xs text-accent">
                  <Check size={13} />
                  {t('h.localStt.selected')}
                </span>
              )}
            </div>
            <p className="text-xs text-text-secondary">
              {size(model.files.reduce((n, f) => n + f.size, 0))} ·{' '}
              {t('h.localStt.ram', { count: model.recommended_ram_gb })} · {model.license}
            </p>
            {!model.available ? (
              <p className="text-xs text-text-tertiary">{t('h.localStt.unavailable')}</p>
            ) : (
              <div className="flex flex-wrap gap-2">
                {model.installed ? (
                  <button
                    className={button}
                    disabled={pending || status.busy || selected}
                    onClick={() => void run('select_local_stt_model', model.id)}
                  >
                    <Check size={13} />
                    {t('h.localStt.use')}
                  </button>
                ) : downloading ? (
                  <button
                    className={button}
                    disabled={pending}
                    onClick={() => void run('cancel_local_stt_download')}
                  >
                    <Pause size={13} />
                    {t('h.localStt.pause')}
                  </button>
                ) : (
                  <button
                    className={button}
                    disabled={pending || status.busy}
                    onClick={() => void run('download_local_stt_model', model.id)}
                  >
                    <Download size={13} />
                    {t(model.partial_bytes || failed ? 'h.localStt.resume' : 'h.localStt.download')}
                  </button>
                )}
                {(model.installed || model.partial_bytes > 0) && (
                  <button
                    className={button}
                    disabled={pending || status.busy}
                    onClick={() => void run('delete_local_stt_model', model.id)}
                  >
                    <Trash2 size={13} />
                    {t('h.localStt.delete')}
                  </button>
                )}
              </div>
            )}
            {downloading && (
              <div className="space-y-1" role="status">
                <progress
                  className="w-full h-1.5 accent-accent"
                  aria-label={t('h.localStt.download')}
                  value={progress.downloaded}
                  max={progress.total || 1}
                />
                <p className="text-xs text-text-secondary">
                  {progress.phase === 'verifying'
                    ? t('h.localStt.verifying')
                    : `${size(progress.downloaded)} / ${size(progress.total)}`}
                </p>
              </div>
            )}
            {failed && progress.error && (
              <p role="status" className="text-xs text-text-secondary">
                {progress.error}
              </p>
            )}
          </article>
        )
      })}
      {status && !status.selected && (
        <p className="text-xs text-text-secondary">{t('h.localStt.choose')}</p>
      )}
    </section>
  )
}

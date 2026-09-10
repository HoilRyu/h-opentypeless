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
  preview_enabled?: boolean
  engine?: { preference: string; active: string; reason: string | null; resident: boolean }
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
  const [viewedModelId, setViewedModelId] = useState('')
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
  const transferActive = status && ['downloading', 'verifying'].includes(status.progress.phase)
  const displayedId = status?.models.some((model) => model.id === viewedModelId)
    ? viewedModelId
    : ((transferActive ? status.progress.model : status?.selected) ?? status?.models[0]?.id ?? '')
  const transferModel = status?.models.find((model) => model.id === status.progress.model)
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
      {status && (
        <div className="space-y-2">
          <label htmlFor="local-stt-model" className="block text-sm font-medium">
            {t('h.localStt.modelLabel')}
          </label>
          <select
            id="local-stt-model"
            className="w-full rounded-xl border border-border bg-bg-secondary px-3 py-3 text-sm"
            value={displayedId}
            disabled={pending}
            onChange={(event) => setViewedModelId(event.target.value)}
          >
            {status.models.map((model) => (
              <option key={model.id} value={model.id}>
                {model.name}
                {status.selected === model.id
                  ? ` · ${t('h.localStt.selected')}`
                  : model.installed
                    ? ` · ${t('h.localStt.downloaded')}`
                    : ''}
              </option>
            ))}
          </select>
          <p className="text-xs text-text-tertiary">{t('h.localStt.modelBrowseHint')}</p>
        </div>
      )}
      {transferActive && status && status.progress.model !== displayedId && (
        <div
          className="flex items-center justify-between gap-3 rounded-xl border border-border p-3 text-xs"
          role="status"
        >
          <span>
            {transferModel?.name} ·{' '}
            {status.progress.phase === 'verifying'
              ? t('h.localStt.verifying')
              : `${size(status.progress.downloaded)} / ${size(status.progress.total)}`}
          </span>
          <button
            type="button"
            className={button}
            onClick={() => setViewedModelId(status.progress.model)}
          >
            {t('h.localStt.showDownload')}
          </button>
        </div>
      )}
      {status?.models
        .filter((model) => model.id === displayedId)
        .map((model) => {
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
              <p className="text-xs leading-relaxed text-text-tertiary">
                {t(
                  model.engine === 'whisper'
                    ? 'h.localStt.whisperCpuHint'
                    : 'h.localStt.qwenEngineHint',
                )}
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
                      {t(
                        model.partial_bytes || failed ? 'h.localStt.resume' : 'h.localStt.download',
                      )}
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
      {status && (
        <label className="flex items-start gap-3 rounded-xl border border-border p-4 cursor-pointer">
          <input
            type="checkbox"
            className="mt-1 accent-blue-500"
            checked={status.preview_enabled ?? true}
            disabled={pending || status.busy}
            onChange={(event) =>
              void run('set_local_stt_preview', event.target.checked ? 'on' : 'off')
            }
          />
          <span className="space-y-1">
            <span className="block text-sm font-medium">{t('h.localStt.previewTitle')}</span>
            <span className="block text-xs leading-relaxed text-text-secondary">
              {t('h.localStt.previewDescription')}
            </span>
          </span>
        </label>
      )}
      {status?.engine && (
        <div className="rounded-xl border border-border p-4 space-y-3">
          <label className="flex items-center justify-between gap-3 text-sm">
            {t('h.localStt.engineTitle')}
            <select
              className="rounded-lg border border-border bg-bg-secondary px-3 py-2 text-xs"
              value={status.engine.preference}
              disabled={pending || status.busy}
              onChange={(event) => void run('set_local_stt_engine', event.target.value)}
            >
              <option value="auto">{t('h.localStt.engineAuto')}</option>
              <option value="mlx">MLX GPU</option>
              <option value="cpu">CPU</option>
            </select>
          </label>
          <p className="text-xs text-text-secondary" role="status">
            {t('h.localStt.engineActive', { engine: status.engine.active.toUpperCase() })}
            {' · '}
            {t(
              status.busy
                ? 'h.localStt.engineBusy'
                : status.engine.resident
                  ? 'h.localStt.engineResident'
                  : 'h.localStt.engineIdle',
            )}
          </p>
          {status.engine.reason && (
            <p className="text-xs text-text-tertiary">{status.engine.reason}</p>
          )}
          <p className="text-xs text-text-tertiary">{t('h.localStt.engineHelp')}</p>
          {status.engine.resident && (
            <button
              className={button}
              disabled={pending || status.busy}
              onClick={() => void run('unload_local_stt_engine')}
            >
              {t('h.localStt.engineUnload')}
            </button>
          )}
        </div>
      )}
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
      {status && !status.selected && (
        <p className="text-xs text-text-secondary">{t('h.localStt.choose')}</p>
      )}
    </section>
  )
}

import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { HistoryEntry } from '../../stores/appStore'
import { repolishHistory, type RepolishResult } from '../../lib/tauri'

export function RepolishDialog({ entry, onClose }: { entry: HistoryEntry; onClose: () => void }) {
  const { t } = useTranslation()
  const [style, setStyle] = useState('clean')
  const [result, setResult] = useState<RepolishResult | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [copied, setCopied] = useState(false)
  const panel = useRef<HTMLDivElement>(null)
  const alive = useRef(true)
  const running = useRef(false)
  useEffect(() => {
    alive.current = true
    panel.current?.querySelector<HTMLSelectElement>('select')?.focus()
    return () => {
      alive.current = false
    }
  }, [])

  const run = async () => {
    if (running.current) return
    running.current = true
    setBusy(true)
    setError('')
    setCopied(false)
    try {
      const next = await repolishHistory(entry.id, style)
      if (alive.current) setResult(next)
    } catch (e) {
      const key = String(e)
      if (alive.current)
        setError(key.startsWith('history.repolish') ? key : 'history.repolishFailed')
    } finally {
      running.current = false
      if (alive.current) setBusy(false)
    }
  }
  const copy = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text)
      if (alive.current) setCopied(true)
    } catch {
      if (alive.current) setError('history.failedToCopy')
    }
  }
  const field = (label: string, value: string) => (
    <label className="block text-[12px] text-text-secondary">
      {t(label)}
      <textarea
        readOnly
        value={value}
        rows={4}
        className="mt-1 w-full resize-y rounded-lg border border-border bg-bg-secondary p-2 text-[13px] text-text-primary"
      />
    </label>
  )
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/25 p-4">
      <div
        ref={panel}
        role="dialog"
        aria-modal="true"
        aria-label={t('history.repolish')}
        className="w-full max-w-[620px] max-h-[90vh] overflow-y-auto rounded-xl border border-border bg-bg-primary p-4 shadow-float space-y-3"
        onKeyDown={(event) => {
          if (event.key === 'Escape') {
            event.stopPropagation()
            onClose()
          }
          if (event.key !== 'Tab') return
          const nodes = panel.current?.querySelectorAll<HTMLElement>(
            'button:not(:disabled),select:not(:disabled),textarea',
          )
          if (!nodes?.length) return
          const first = nodes[0],
            last = nodes[nodes.length - 1]
          if (event.shiftKey && document.activeElement === first) {
            event.preventDefault()
            last.focus()
          } else if (!event.shiftKey && document.activeElement === last) {
            event.preventDefault()
            first.focus()
          }
        }}
      >
        <h3 className="text-[15px] font-medium">{t('history.repolish')}</h3>
        <p className="text-[12px] text-text-secondary">{t('history.repolishHint')}</p>
        {field('history.originalTranscript', entry.raw_text)}
        {field('history.previousResult', entry.polished_text)}
        <div className="flex items-end gap-2">
          <label className="flex-1 text-[12px] text-text-secondary">
            {t('history.repolishStyle')}
            <select
              value={style}
              disabled={busy}
              onChange={(event) => setStyle(event.target.value)}
              className="block mt-1 w-full rounded-lg border border-border bg-bg-secondary p-2"
            >
              {['minimal', 'clean', 'structured', 'professional'].map((value) => (
                <option key={value} value={value}>
                  {t(`history.repolishStyles.${value}`)}
                </option>
              ))}
            </select>
          </label>
          <button
            type="button"
            disabled={busy || !entry.raw_text.trim()}
            onClick={run}
            className="rounded-lg bg-accent px-3 py-2 text-[12px] text-white disabled:opacity-50"
          >
            {t(busy ? 'history.repolishing' : 'history.repolishRun')}
          </button>
        </div>
        {result && (
          <>
            {field('history.newResult', result.polished_text)}
            <p className="text-[11px] text-text-tertiary">
              {result.model} · {t(`history.repolishStyles.${result.style}`)} ·{' '}
              {(result.elapsed_ms / 1000).toFixed(1)}s
            </p>
          </>
        )}
        {error && (
          <p role="alert" className="text-[12px] text-error">
            {t(error)}
          </p>
        )}
        {copied && (
          <p role="status" className="text-[12px] text-text-secondary">
            {t('history.repolishCopied')}
          </p>
        )}
        <div className="flex flex-wrap justify-end gap-2 text-[12px]">
          <button
            type="button"
            onClick={() => copy(entry.raw_text)}
            className="rounded-lg border border-border px-3 py-2"
          >
            {t('history.copyOriginal')}
          </button>
          {result && (
            <button
              type="button"
              onClick={() => copy(result.polished_text)}
              className="rounded-lg border border-border px-3 py-2"
            >
              {t('history.copyNewResult')}
            </button>
          )}
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg border border-border px-3 py-2"
          >
            {t('history.repolishClose')}
          </button>
        </div>
      </div>
    </div>
  )
}

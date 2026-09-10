import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Loader2, Mic, Square } from 'lucide-react'
import type { ko } from './copy'

type Phase = 'idle' | 'preparing' | 'recording' | 'processing'
interface Preview {
  raw_text: string
  polished_text: string
  warning: string | null
  processing_ms: number
}
export function Practice({
  copy: c,
  onBusy,
  onSuccess,
}: {
  copy: typeof ko
  onBusy: (v: boolean) => void
  onSuccess: () => void
}) {
  const [phase, setPhase] = useState<Phase>('idle')
  const [result, setResult] = useState<Preview | null>(null)
  const [error, setError] = useState('')
  const current = useRef<string | null>(null)
  const mounted = useRef(true)
  const cancelled = useRef(false)
  const cancel = () => {
    const id = current.current
    if (!id) return
    cancelled.current = true
    void invoke('control_tutorial_recording', { id, cancel: true }).catch((e) => {
      if (mounted.current) setError(String(e))
    })
  }
  useEffect(() => {
    mounted.current = true
    const hide = () => {
      if (document.hidden) cancel()
    }
    document.addEventListener('visibilitychange', hide)
    return () => {
      mounted.current = false
      cancel()
      current.current = null
      document.removeEventListener('visibilitychange', hide)
    }
  }, [])
  const start = async () => {
    if (current.current) return
    const id = crypto.randomUUID()
    current.current = id
    cancelled.current = false
    setResult(null)
    setError('')
    setPhase('preparing')
    onBusy(true)
    let off: (() => void) | undefined
    try {
      off = await listen<{ id: string; phase: Phase }>('h-tutorial:phase', (event) => {
        if (event.payload.id !== id) return
        if (!mounted.current || cancelled.current) {
          // Retry if cancellation raced the backend reserving this session.
          void invoke('control_tutorial_recording', { id, cancel: true }).catch(() => {})
          return
        }
        if (current.current === id) setPhase(event.payload.phase)
      })
      if (!mounted.current || cancelled.current) return
      const preview = await invoke<Preview>('run_tutorial_recording', { id })
      if (!mounted.current || current.current !== id) return
      if (cancelled.current) {
        setError(c.cancelled)
        return
      }
      if (!preview.raw_text.trim()) {
        setError(c.empty)
        return
      }
      setResult(preview)
      onSuccess()
    } catch (e) {
      if (mounted.current && current.current === id)
        setError(cancelled.current ? c.cancelled : String(e))
    } finally {
      off?.()
      if (current.current === id) current.current = null
      if (mounted.current) {
        setPhase('idle')
        onBusy(false)
      }
    }
  }
  return (
    <div className="h-guide-practice">
      <div className="h-guide-mic">
        {phase === 'preparing' || phase === 'processing' ? (
          <Loader2 size={25} className="animate-spin" />
        ) : (
          <Mic
            size={25}
            strokeWidth={1.5}
            className={phase === 'recording' ? 'animate-pulse' : ''}
          />
        )}
      </div>
      {!result && <p className="text-lg leading-relaxed mb-5">{c.practiceExample}</p>}
      <div className="flex justify-center gap-3">
        {phase === 'idle' && (
          <button
            className={result ? 'h-guide-secondary' : 'h-guide-primary'}
            onClick={() => void start()}
          >
            {result ? c.retry : c.start}
          </button>
        )}
        {phase === 'recording' && (
          <button
            className="h-guide-primary"
            onClick={() => {
              setPhase('processing')
              void invoke('control_tutorial_recording', {
                id: current.current,
                cancel: false,
              }).catch((e) => setError(String(e)))
            }}
          >
            <Square size={14} />
            {c.stop}
          </button>
        )}
        {phase !== 'idle' && (
          <button className="h-guide-secondary" onClick={cancel}>
            {c.cancel}
          </button>
        )}
      </div>
      <p role="status" className="h-guide-note">
        {phase === 'idle' ? (result ? c.practiced : c.untested) : c[phase]}
      </p>
      {error && (
        <p role="alert" className="h-guide-error mt-4">
          {error}
        </p>
      )}
      {result && (
        <div className="h-guide-practice-result" aria-live="polite">
          <h3>{c.polished}</h3>
          <p>{result.polished_text}</p>
          {result.warning && <p className="text-warning mt-3">{result.warning}</p>}
          <details className="h-guide-more">
            <summary>{c.raw}</summary>
            <div>
              <p>{result.raw_text}</p>
              <span>
                {c.processingTime}: {(result.processing_ms / 1000).toFixed(1)} s
              </span>
            </div>
          </details>
        </div>
      )}
      <details className="h-guide-more">
        <summary>{c.practiceDetails}</summary>
        <div>{c.practicePrivacy}</div>
      </details>
    </div>
  )
}

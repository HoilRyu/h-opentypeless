import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import './voiceFeedback.css'
export type FeedbackSettings = {
  enabled: boolean
  brightness: number
  reactive: boolean
  near_caret: boolean
}
export type FeedbackSnapshot = {
  phase: string
  settings: FeedbackSettings
  revision: number
  failed?: boolean
  preview?: boolean
}
export function VoiceEdge() {
  const [snapshot, setSnapshot] = useState<FeedbackSnapshot | null>(null)
  const edge = useRef<HTMLDivElement>(null)
  useEffect(() => {
    let live = true
    const apply = (next: FeedbackSnapshot) => {
      if (live) setSnapshot((old) => (!old || next.revision >= old.revision ? next : old))
    }
    const cleanups: (() => void)[] = []
    const register = async () => {
      const off = await listen<FeedbackSnapshot>('voice-feedback:state', (e) => {
        apply(e.payload)
        edge.current?.style.setProperty('--voice-level', '0')
      })
      if (!live) {
        off()
        return
      }
      cleanups.push(off)
      const value = await invoke<FeedbackSnapshot>('get_voice_feedback')
      apply(value)
      let last = 0
      const offVolume = await listen<number>('audio:volume', (e) => {
        const now = performance.now()
        if (now - last < 33) return
        last = now
        edge.current?.style.setProperty(
          '--voice-level',
          String(Math.min(1, Math.max(0, e.payload))),
        )
      })
      if (!live) offVolume()
      else cleanups.push(offVolume)
    }
    register().catch(console.error)
    return () => {
      live = false
      cleanups.forEach((off) => off())
    }
  }, [])
  const active =
    snapshot && snapshot.settings.enabled && !snapshot.failed && snapshot.phase !== 'idle'
  const recording = snapshot?.phase === 'recording' || snapshot?.phase === 'ask_recording'
  return (
    <div
      ref={edge}
      aria-hidden="true"
      className={`voice-edge ${active ? 'active' : ''} ${recording ? 'recording' : snapshot?.phase === 'outputting' ? 'complete' : 'processing'} ${snapshot?.settings.reactive ? 'reactive' : ''}`}
      style={{ '--voice-brightness': snapshot?.settings.brightness ?? 0.55 } as React.CSSProperties}
    >
      <i className="top" />
      <i className="right" />
      <i className="bottom" />
      <i className="left" />
    </div>
  )
}

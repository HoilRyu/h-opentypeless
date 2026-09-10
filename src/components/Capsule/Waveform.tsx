import { useEffect, useRef } from 'react'
import { useReducedMotion } from 'framer-motion'
import { isMacPlatform, useAppStore } from '../../stores/appStore'

const BAR_COUNT = 7
const MIN_HEIGHT = 3
const MAX_HEIGHT = 16

export function Waveform({ expanded = false }: { expanded?: boolean }) {
  const count = expanded ? 21 : BAR_COUNT
  const barsRef = useRef<(HTMLDivElement | null)[]>([])
  const rafRef = useRef<number>(0)
  const reduced = useReducedMotion()

  useEffect(() => {
    if (reduced) {
      // Static bars at mid-height when reduced motion is preferred
      barsRef.current.forEach((bar) => {
        if (!bar) return
        bar.style.height = `${(MIN_HEIGHT + MAX_HEIGHT) / 2}px`
        bar.style.opacity = '0.7'
      })
      return
    }

    const mac = isMacPlatform()
    let last = 0
    const history = Array<number>(count).fill(0)
    const animate = (now: number) => {
      if (mac && now - last < 33) {
        rafRef.current = requestAnimationFrame(animate)
        return
      }
      last = now
      const volume = useAppStore.getState().audioVolume
      history.push(Math.max(0, Math.min(1, volume)))
      history.shift()
      barsRef.current.forEach((bar, i) => {
        if (!bar) return
        const normalized = mac
          ? history[i]
          : Math.max(0, Math.min(1, volume + Math.sin(Date.now() / 200 + i * 0.9) * 0.15))
        const height = MIN_HEIGHT + (MAX_HEIGHT - MIN_HEIGHT) * normalized
        const opacity = Math.max(0.5, normalized)
        bar.style.height = `${height}px`
        bar.style.opacity = `${opacity}`
      })
      rafRef.current = requestAnimationFrame(animate)
    }

    rafRef.current = requestAnimationFrame(animate)
    return () => cancelAnimationFrame(rafRef.current)
  }, [reduced, count])

  return (
    <div
      className={`flex items-center h-4 ${expanded ? 'flex-1 min-w-8 justify-between overflow-hidden' : 'justify-center gap-[3px]'}`}
      aria-hidden="true"
    >
      {Array.from({ length: count }).map((_, i) => (
        <div
          key={i}
          ref={(el) => {
            barsRef.current[i] = el
          }}
          className="w-[2px] shrink-0 rounded-full bg-white/80"
          style={{
            height: `${MIN_HEIGHT}px`,
            opacity: 0.5,
            transition: 'height 75ms ease-out, opacity 75ms ease-out',
          }}
        />
      ))}
    </div>
  )
}

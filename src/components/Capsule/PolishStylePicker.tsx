import { useState } from 'react'
import { ChevronDown } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'
import { useAppStore } from '../../stores/appStore'

const styles = ['minimal', 'clean', 'structured', 'professional'] as const

export function PolishStylePicker() {
  const { t, i18n } = useTranslation()
  const ko = i18n?.language?.startsWith('ko') ?? false
  const config = useAppStore((s) => s.config)
  const open = useAppStore((s) => s.polishStyleMenuOpen)
  const setOpen = useAppStore((s) => s.setPolishStyleMenuOpen)
  const [style, setStyle] = useState(config.polish_style)
  const [pending, setPending] = useState(false)
  const [error, setError] = useState(false)
  if (!config.polish_enabled || config.active_scene) return null
  const label = (value: string) =>
    t(`settings.polishStyle${value[0].toUpperCase()}${value.slice(1)}`)
  const select = async (value: (typeof styles)[number]) => {
    setPending(true)
    setError(false)
    try {
      await invoke('set_recording_polish_style', { style: value })
      setStyle(value)
      setOpen(false)
    } catch {
      setError(true)
    } finally {
      setPending(false)
    }
  }
  return (
    <div
      onPointerDown={(e) => e.stopPropagation()}
      onPointerUp={(e) => e.stopPropagation()}
      onClick={(e) => e.stopPropagation()}
    >
      <button
        type="button"
        aria-expanded={open}
        aria-haspopup="menu"
        aria-label={ko ? '이번 녹음의 다듬기 스타일' : 'Style for this recording'}
        onClick={() => {
          useAppStore.getState().setTranslationTargetMenuOpen(false)
          useAppStore.getState().setContextMenuOpen(false)
          setOpen(!open)
        }}
        className="flex h-6 items-center gap-1 rounded-full border border-white/20 bg-white/10 px-2 text-[11px] whitespace-nowrap text-white/90 hover:bg-white/20"
      >
        {label(style)}
        <ChevronDown size={11} />
      </button>
      {open && (
        <div
          className="absolute left-3 right-3 top-10 z-30 rounded-xl bg-neutral-900 p-1 shadow-lg"
          role="menu"
          aria-label={ko ? '다듬기 스타일' : 'Polish style'}
        >
          <div className="flex gap-1">
            {styles.map((value) => (
              <button
                key={value}
                type="button"
                role="menuitemradio"
                aria-checked={value === style}
                disabled={pending}
                onClick={() => void select(value)}
                className={`flex-1 rounded-lg py-2 text-[11px] whitespace-nowrap disabled:opacity-50 ${value === style ? 'bg-white/20 text-white' : 'text-white/70 hover:bg-white/10'}`}
              >
                {label(value)}
              </button>
            ))}
          </div>
          {error && (
            <p role="alert" className="px-2 text-[10px] text-amber-200">
              {ko
                ? '변경하지 못했습니다. 다시 선택해 주세요.'
                : 'Could not change style. Try again.'}
            </p>
          )}
        </div>
      )}
    </div>
  )
}

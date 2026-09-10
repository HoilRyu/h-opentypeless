import { isMacPlatform, useAppStore } from '../../stores/appStore'
import { useTranslation } from 'react-i18next'
import { motion, useReducedMotion } from 'framer-motion'
import { X } from 'lucide-react'
import { abortRecording } from '../../lib/tauri'
import { Waveform } from './Waveform'
import { DurationTimer } from './DurationTimer'
import { PolishStylePicker } from './PolishStylePicker'
import { TranslateTargetChip } from './TranslateTargetChip'

export function CapsuleRecording() {
  const { t, i18n } = useTranslation()
  const styleMenu = useAppStore((s) => s.polishStyleMenuOpen)
  const reduced = useReducedMotion()
  const partial = useAppStore((s) =>
    s.config.stt_provider === 'builtin-stt' ? s.partialTranscript : '',
  )

  const handleCancel = async (e: React.MouseEvent) => {
    e.stopPropagation()
    try {
      await abortRecording()
    } catch (err) {
      console.error('Failed to abort recording:', err)
    }
  }

  const stopPointerPropagation = (e: React.PointerEvent) => {
    e.stopPropagation()
  }

  return (
    <div className="relative z-10">
      <motion.div className="flex items-center gap-2 h-9 px-3">
        {/* White pulse dot — gentle opacity loop */}
        <motion.div
          className="w-2 h-2 rounded-full bg-white/80 flex-shrink-0"
          animate={reduced ? undefined : { opacity: [1, 0.5, 1] }}
          transition={{ repeat: Infinity, duration: 1.5, ease: 'easeInOut' }}
        />
        {isMacPlatform() && (
          <span className="whitespace-nowrap text-[11px] font-semibold">
            {i18n.language.startsWith('ko') ? '듣는 중' : 'Listening'}
          </span>
        )}
        <Waveform expanded />
        <TranslateTargetChip />
        <DurationTimer />
        <PolishStylePicker />
        <button
          onPointerDown={stopPointerPropagation}
          onPointerUp={stopPointerPropagation}
          onClick={handleCancel}
          aria-label={t('capsule.cancelRecording')}
          className="flex-shrink-0 p-1 rounded-full text-white/70 hover:text-white hover:bg-white/15 transition-colors bg-transparent border-none cursor-pointer"
        >
          <X size={12} />
        </button>
      </motion.div>
      {partial && (
        <div
          style={styleMenu ? { marginTop: 64 } : undefined}
          role="status"
          aria-live="polite"
          className="mx-4 mb-3 h-10 overflow-hidden text-[12px] leading-5 text-white/85 flex flex-col justify-end"
        >
          <p className="shrink-0 break-words whitespace-pre-wrap">{partial.slice(-1200)}</p>
        </div>
      )}
    </div>
  )
}

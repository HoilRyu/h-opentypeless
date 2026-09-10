import { VoiceFeedbackSetting } from '../ForkSettings/VoiceFeedbackSetting'
import { AudioDuckingSetting } from '../ForkSettings/AudioDuckingSetting'
import { useState, useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { ChevronDown } from 'lucide-react'
import { isMacPlatform, useAppStore } from '../../stores/appStore'
import type { HotkeyMode, OutputMode, ShortcutBinding } from '../../stores/appStore'
import {
  getPlatformCapabilities,
  getHotkeyStatus,
  resumeHotkey,
} from '../../lib/tauri'
import type { HotkeyStatus } from '../../lib/tauri'
import { SegmentedControl } from './shared/SegmentedControl'
import { Toggle } from './shared/Toggle'
import { ShortcutBindingList } from './ShortcutBindingList'

const MAC_ACCESSIBILITY_HOTKEY_ERROR = 'Accessibility permission may be denied'

export function GeneralPane() {
  const config = useAppStore((s) => s.config)
  const updateConfig = useAppStore((s) => s.updateConfig)
  const platformCapabilities = useAppStore((s) => s.platformCapabilities)
  const setPlatformCapabilities = useAppStore((s) => s.setPlatformCapabilities)
  const hotkeyRegistrationError = useAppStore((s) => s.hotkeyRegistrationError)
  const setHotkeyRegistrationError = useAppStore((s) => s.setHotkeyRegistrationError)
  const accessibilityTrusted = useAppStore((s) => s.accessibilityTrusted)
  const { t } = useTranslation()
  const isMac = isMacPlatform()
  const [hotkeyStatus, setHotkeyStatus] = useState<HotkeyStatus | null>(null)
  const [advancedOpen, setAdvancedOpen] = useState(false)
  const accessibilityRecoveryAttemptedRef = useRef(false)

  useEffect(() => {
    if (platformCapabilities) return
    getPlatformCapabilities()
      .then(setPlatformCapabilities)
      .catch((err) => {
        console.error('Failed to load platform capabilities:', err)
      })
  }, [platformCapabilities, setPlatformCapabilities])

  useEffect(() => {
    let cancelled = false
    getHotkeyStatus()
      .then((status) => {
        if (!cancelled) {
          setHotkeyStatus(status)
          setHotkeyRegistrationError(status.registration_error)
        }
      })
      .catch((err) => {
        console.error('Failed to load hotkey status:', err)
      })
    return () => {
      cancelled = true
    }
  }, [config.hotkeys, hotkeyRegistrationError, setHotkeyRegistrationError])

  useEffect(() => {
    if (
      !isMac ||
      !accessibilityTrusted ||
      !hotkeyRegistrationError?.includes(MAC_ACCESSIBILITY_HOTKEY_ERROR)
    ) {
      if (!hotkeyRegistrationError) {
        accessibilityRecoveryAttemptedRef.current = false
      }
      return
    }
    if (accessibilityRecoveryAttemptedRef.current) return
    accessibilityRecoveryAttemptedRef.current = true

    resumeHotkey()
      .then(() => {
        setHotkeyRegistrationError(null)
      })
      .catch((err) => {
        const message = err instanceof Error ? err.message : String(err)
        setHotkeyRegistrationError(message)
      })
  }, [accessibilityTrusted, hotkeyRegistrationError, isMac, setHotkeyRegistrationError])

  const hotkeyStatusMessage = hotkeyStatus?.conflict
    ? t('settings.hotkeyConflict')
    : hotkeyStatus && (!hotkeyStatus.dictation.valid || !hotkeyStatus.ask.valid)
      ? t('settings.hotkeyInvalid')
      : null
  const registrationErrorCoveredByAccessibilityBanner = Boolean(
    isMac &&
    !accessibilityTrusted &&
    hotkeyRegistrationError?.includes('Accessibility permission may be denied'),
  )
  const dictationSpecialOptions = isMac ? [{ value: 'Fn', label: 'Fn' }] : []
  const translateSpecialOptions = isMac ? [{ value: 'Fn+LeftShift', label: 'Fn + Left Shift' }] : []
  const dictationBindings = config.hotkeys.dictationBindings?.length
    ? config.hotkeys.dictationBindings
    : [config.hotkeys.dictation]
  const translateBindings =
    config.hotkeys.translateBindings ?? (config.hotkeys.translate ? [config.hotkeys.translate] : [])
  const secondaryBindings = [
    config.hotkeys.editSelection,
    config.hotkeys.switchScene,
    config.hotkeys.openApp,
    config.hotkeys.copyResult,
  ].filter((binding): binding is ShortcutBinding => Boolean(binding))
  const otherBindingsFor = (role: 'dictation' | 'translate') => [
    ...(role === 'dictation' ? [] : dictationBindings),
    ...(role === 'translate' ? [] : translateBindings),
    ...secondaryBindings,
  ]
  const updateCoreBindings = (
    role: 'dictation' | 'translate',
    bindings: ShortcutBinding[],
  ) => {
    const nextHotkeys = { ...config.hotkeys }
    if (role === 'dictation') {
      if (bindings.length === 0) return
      nextHotkeys.dictationBindings = bindings
      nextHotkeys.dictation = bindings[0]

    } else {
      nextHotkeys.translateBindings = bindings
      nextHotkeys.translate = bindings[0] ?? null
    }
    updateConfig({ hotkeys: nextHotkeys })
  }

  return (
    <div className="space-y-6">
      <AudioDuckingSetting />
      {isMac && <VoiceFeedbackSetting />}
      <Section title={t('settings.hotkey')}>
        <div className="space-y-3">
          <ShortcutBindingList
            role="dictation"
            label={t('settings.dictationHotkey')}
            bindings={dictationBindings}
            otherBindings={otherBindingsFor('dictation')}
            required
            specialOptions={dictationSpecialOptions}
            onChange={(bindings) => updateCoreBindings('dictation', bindings)}
          />
          <ShortcutBindingList
            role="translate"
            label={t('settings.translateHotkey')}
            bindings={translateBindings}
            otherBindings={otherBindingsFor('translate')}
            required={false}
            specialOptions={translateSpecialOptions}
            onChange={(bindings) => updateCoreBindings('translate', bindings)}
          />
          <ShortcutBindingList
            role="copyResult"
            label={t('settings.copyResultHotkey')}
            bindings={config.hotkeys.copyResult ? [config.hotkeys.copyResult] : []}
            otherBindings={[
              ...dictationBindings,
              ...translateBindings,
              ...[
                config.hotkeys.editSelection,
                config.hotkeys.switchScene,
                config.hotkeys.openApp,
              ].filter((binding): binding is ShortcutBinding => Boolean(binding)),
            ]}
            required={false}
            specialOptions={[]}
            maxBindings={1}
            onChange={(bindings) =>
              updateConfig({
                hotkeys: {
                  ...config.hotkeys,
                  copyResult: bindings[0] ?? null,
                },
              })
            }
          />
          <p className="text-xs text-text-tertiary">{t('settings.copyResultHotkeyHint')}</p>
        </div>
        {platformCapabilities && !platformCapabilities.globalHotkeyReliable && (
          <p className="mt-2 rounded-[8px] border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-[12px] leading-relaxed text-text-secondary">
            {t('settings.waylandHotkeyLimited')}
          </p>
        )}
        {hotkeyRegistrationError && !registrationErrorCoveredByAccessibilityBanner && (
          <p className="mt-2 rounded-[8px] border border-error/30 bg-error/10 px-3 py-2 text-[12px] leading-relaxed text-error">
            {t('settings.hotkeyRegistrationFailed')}
          </p>
        )}
        {hotkeyStatusMessage && (
          <p className="mt-2 rounded-[8px] border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-[12px] leading-relaxed text-text-secondary">
            {hotkeyStatusMessage}
          </p>
        )}
      </Section>

      <Section title={t('settings.dictationMode')}>
        <SegmentedControl
          options={[
            { value: 'hold', label: t('settings.holdToTalk') },
            { value: 'toggle', label: t('settings.toggleOnOff') },
          ]}
          value={config.hotkey_mode}
          onChange={(v) => updateConfig({ hotkey_mode: v as HotkeyMode })}
        />
      </Section>

      <Section title={t('settings.outputMode')}>
        <SegmentedControl
          options={[
            { value: 'keyboard', label: t('settings.keyboardSimulation') },
            { value: 'clipboard', label: t('settings.clipboardPaste') },
          ]}
          value={config.output_mode}
          onChange={(v) => {
            const outputMode = v as OutputMode
            updateConfig({
              output_mode: outputMode,
              insertion_strategy: outputMode === 'clipboard' ? 'clipboardPaste' : 'auto',
            })
          }}
        />
        {config.output_mode === 'clipboard' &&
          platformCapabilities &&
          !platformCapabilities.clipboardAutoPasteReliable && (
            <p className="mt-2 rounded-[8px] border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-[12px] leading-relaxed text-text-secondary">
              {t('settings.waylandClipboardCopyOnly')}
            </p>
          )}
      </Section>

      <div>
        <button
          type="button"
          aria-expanded={advancedOpen}
          onClick={() => setAdvancedOpen((open) => !open)}
          className="flex w-full items-center justify-between rounded-[10px] border border-border bg-bg-secondary/40 px-3 py-2 text-[13px] font-medium text-text-primary transition-colors hover:border-border-focus"
        >
          <span>{t('settings.advancedGeneral')}</span>
          <ChevronDown
            size={14}
            className={`text-text-tertiary transition-transform ${advancedOpen ? 'rotate-180' : ''}`}
          />
        </button>

        {advancedOpen && (
          <div className="mt-4 space-y-3">
            <Toggle
              checked={config.auto_start}
              onChange={(checked) => updateConfig({ auto_start: checked })}
              label={t('settings.launchAtStartup')}
            />
            <Toggle
              checked={config.history_enabled}
              onChange={(checked) => updateConfig({ history_enabled: checked })}
              label={t('settings.saveHistory')}
            />
            <Toggle
              checked={config.capsule_auto_hide}
              onChange={(checked) => updateConfig({ capsule_auto_hide: checked })}
              label={t('settings.hideCapsuleWhenIdle')}
            />
          </div>
        )}
      </div>
    </div>
  )
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h3 className="text-[11px] font-medium text-text-tertiary uppercase tracking-wider mb-2.5">
        {title}
      </h3>
      {children}
    </div>
  )
}

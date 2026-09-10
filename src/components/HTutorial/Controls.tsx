import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useAppStore, type ShortcutBinding } from '../../stores/appStore'
import { needsMacAccessibility, isMacPlatform } from '../../lib/accessibility'
import {
  checkAccessibilityPermission,
  requestAccessibilityPermission,
  resumeHotkey,
} from '../../lib/tauri'
import { ShortcutBindingList } from '../Settings/ShortcutBindingList'
import type { ko } from './copy'

export function Controls({ copy: c }: { copy: typeof ko }) {
  const { t } = useTranslation()
  const config = useAppStore((s) => s.config)
  const update = useAppStore((s) => s.updateConfig)
  const [trusted, setTrusted] = useState<boolean | null>(null)
  const [error, setError] = useState('')
  const needs = needsMacAccessibility(config)
  useEffect(() => {
    let live = true
    const check = () => {
      if (!needs) return
      checkAccessibilityPermission()
        .then((value) => {
          if (live) {
            setTrusted(value)
            useAppStore.getState().setAccessibilityTrusted(value)
          }
        })
        .catch((e) => {
          if (live) setError(String(e))
        })
    }
    check()
    window.addEventListener('focus', check)
    return () => {
      live = false
      window.removeEventListener('focus', check)
    }
  }, [needs])
  const h = config.hotkeys
  const other = [
    ...(h.askBindings ?? (h.ask ? [h.ask] : [])),
    ...(h.translateBindings ?? (h.translate ? [h.translate] : [])),
    h.editSelection,
    h.switchScene,
    h.openApp,
    h.copyResult,
  ].filter((b): b is ShortcutBinding => !!b)
  return (
    <div className="space-y-5">
      <div className="rounded-2xl bg-accent/10 p-5 space-y-2">
        <p className="text-xs text-text-secondary">{c.hotkey}</p>
        <kbd className="block text-xl font-semibold">{config.hotkey.split('+').join(' + ')}</kbd>
        <p className="text-sm">{config.hotkey_mode === 'hold' ? c.hold : c.toggle}</p>
      </div>
      <p className="text-sm text-text-secondary">{c.controlHelp}</p>
      <details className="h-guide-details">
        <summary>{c.hotkeySettings}</summary>
        <div className="space-y-4 pt-4">
          <ShortcutBindingList
            role="dictation"
            label={c.hotkey}
            bindings={h.dictationBindings?.length ? h.dictationBindings : [h.dictation]}
            otherBindings={other}
            required
            specialOptions={isMacPlatform() ? [{ value: 'Fn', label: 'Fn' }] : []}
            onChange={(bindings) => {
              if (bindings.length)
                update({ hotkeys: { ...h, dictationBindings: bindings, dictation: bindings[0] } })
            }}
          />
          <label className="block text-sm">
            {c.mode}
            <select
              className="ml-3 rounded-lg border border-border bg-bg-primary p-2"
              value={config.hotkey_mode}
              onChange={(e) => update({ hotkey_mode: e.target.value as 'hold' | 'toggle' })}
            >
              <option value="toggle">{c.toggle}</option>
              <option value="hold">{c.hold}</option>
            </select>
          </label>
        </div>
      </details>
      <div className="rounded-xl border border-border p-4 space-y-2">
        <h3 className="font-medium text-sm">{t('onboarding.permissions.microphone')}</h3>
        <p className="text-sm text-text-secondary">{t('onboarding.permissions.microphoneDesc')}</p>
      </div>
      {needs && (
        <div className="rounded-xl border border-border p-4 space-y-3">
          <h3 className="font-medium text-sm">{t('onboarding.permissions.textOutput')}</h3>
          <p className="text-sm text-text-secondary">
            {t('onboarding.permissions.textOutputDesc')}
          </p>
          <p role="status" className="text-sm">
            {trusted === null
              ? '…'
              : t(`onboarding.permissions.status.${trusted ? 'ready' : 'needed'}`)}
          </p>
          {trusted !== true && (
            <button
              className="h-guide-secondary"
              onClick={async () => {
                try {
                  setError('')
                  await requestAccessibilityPermission()
                  const value = await checkAccessibilityPermission()
                  setTrusted(value)
                  if (value) await resumeHotkey()
                } catch (e) {
                  setError(String(e))
                }
              }}
            >
              {t('onboarding.permissions.fix')}
            </button>
          )}
        </div>
      )}
      {error && (
        <p role="alert" className="text-error text-sm">
          {error}
        </p>
      )}
    </div>
  )
}

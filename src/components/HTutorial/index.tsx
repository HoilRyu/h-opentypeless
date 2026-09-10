import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { invoke } from '@tauri-apps/api/core'
import { ArrowLeft, ArrowRight, BookOpen, Check, ChevronRight, X } from 'lucide-react'
import { useAppStore } from '../../stores/appStore'
import { H_MANAGED_CLOUD_ENABLED } from '../../lib/h-features'
import { useRoute } from '../../lib/router'
import { needsMacAccessibility } from '../../lib/accessibility'
import { SttPane } from '../Settings/SttPane'
import { LlmPane } from '../Settings/LlmPane'
import { Controls } from './Controls'
import { Practice } from './Practice'
import { ko, en } from './copy'
import {
  completeTutorial,
  modelReady,
  readStep,
  rememberStep,
  saveTutorialSettings,
  steps,
  type Step,
  type ModelStatus,
} from './state'
import './tutorial.css'

export function TutorialLink() {
  const { i18n } = useTranslation()
  const { navigate } = useRoute()
  if (H_MANAGED_CLOUD_ENABLED) return null
  return (
    <button
      onClick={() => navigate('tutorial')}
      className="inline-flex items-center gap-2 rounded-xl border border-border px-4 py-3 text-sm text-text-secondary hover:bg-bg-secondary"
    >
      <BookOpen size={16} />
      {i18n.resolvedLanguage?.startsWith('ko') ? ko.revisit : en.revisit}
    </button>
  )
}

export function HTutorial({ review = false, onClose }: { review?: boolean; onClose: () => void }) {
  const { i18n } = useTranslation()
  const c = i18n.resolvedLanguage?.startsWith('ko') ? ko : en
  const config = useAppStore((s) => s.config)
  const trusted = useAppStore((s) => s.accessibilityTrusted)
  const saved = useAppStore((s) => s.savedConfig)
  const update = useAppStore((s) => s.updateConfig)
  const sttTest = useAppStore((s) => s.sttTestStatus)
  const llmTest = useAppStore((s) => s.llmTestStatus)
  const hotkeyError = useAppStore((s) => s.hotkeyRegistrationError)
  const [step, setStep] = useState<Step>(() => (review ? 'welcome' : readStep()))
  const [panel, setPanel] = useState<'stt' | 'polish' | 'controls' | null>(null)
  const [saving, setSaving] = useState(false)
  const [busy, setBusy] = useState(false)
  const [practiced, setPracticed] = useState(false)
  const [error, setError] = useState('')
  const [status, setStatus] = useState<ModelStatus | null>(null)
  const content = useRef<HTMLElement>(null)
  const index = steps.indexOf(step)
  const local = config.stt_provider === 'builtin-stt'
  // Reviewing an unchanged external setup must not force a new connection test.
  const unchangedExternal =
    review && !local && saved !== null && JSON.stringify(config) === JSON.stringify(saved)
  const ready = local ? modelReady(status) : sttTest === 'success' || unchangedExternal
  const selected = status?.models.find((m) => m.id === status.selected)
  const providerName = local
    ? selected?.name || status?.selected || c.notSelected
    : config.stt_provider
  useEffect(() => {
    content.current?.scrollTo?.(0, 0)
    if (!review) rememberStep(step)
  }, [step, review])
  useEffect(() => {
    if (step !== 'stt' || !local) return
    let live = true
    let timer: ReturnType<typeof setTimeout>
    const poll = async () => {
      try {
        const result = await invoke<ModelStatus>('get_local_stt_status')
        if (live) setStatus(result)
      } catch (e) {
        if (live) {
          setStatus(null)
          setError(String(e))
        }
      } finally {
        if (live) timer = setTimeout(poll, 1000)
      }
    }
    void poll()
    return () => {
      live = false
      clearTimeout(timer)
    }
  }, [step, local])
  const saveAnd = async (action: 'next' | 'back' | 'later') => {
    if (saving || busy) return
    setSaving(true)
    setError('')
    try {
      if (step === 'done' && action === 'next') {
        await completeTutorial()
        onClose()
        return
      }
      await saveTutorialSettings()
      if (action === 'later') {
        useAppStore.getState().setOnboardingCompleted(true)
        onClose()
        return
      }
      setPanel(null)
      setStep(steps[index + (action === 'back' ? -1 : 1)])
    } catch (e) {
      setError(String(e))
    } finally {
      setSaving(false)
    }
  }
  const disabled = saving || busy || (step === 'stt' && !ready)
  return (
    <div className="h-guide h-screen flex flex-col bg-bg-primary text-text-primary">
      <header className="h-guide-header">
        <span>H-OpenTypeless</span>
        <button
          aria-label={review ? c.close : c.later}
          title={review ? c.close : c.later}
          disabled={saving || busy}
          onClick={() => void saveAnd('later')}
        >
          <X size={18} />
        </button>
      </header>
      <main ref={content} className={`h-guide-main ${panel ? 'has-panel' : ''}`}>
        <div className="h-guide-page">
          <p className="h-guide-eyebrow">{c.labels[index]}</p>
          <h1>{c.titles[index]}</h1>
          <p className="h-guide-subtitle">{c.subtitles[index]}</p>
          {step === 'welcome' && (
            <>
              <div className="h-guide-scene" aria-label={c.exampleLabel}>
                <div className="h-guide-editor">
                  <div className="h-guide-editor-label">{c.exampleLabel}</div>
                  <p>
                    {c.examplePolished}
                    <span className="h-guide-caret" />
                  </p>
                  <div className="h-guide-editor-lines">
                    <span />
                    <span />
                  </div>
                </div>
                <div className="h-guide-mini-capsule" aria-hidden="true">
                  <span className="h-guide-record-dot" />
                  <span />
                  <span />
                  <span />
                  <span />
                  <span />
                  <span />
                  <span />
                </div>
              </div>
              <div className="h-guide-keyline">
                <kbd>{config.hotkey.split('+').join(' + ')}</kbd>
                <span>{config.hotkey_mode === 'hold' ? c.hold : c.toggle}</span>
              </div>
              <p className="h-guide-note">{c.welcomeText}</p>
            </>
          )}
          {step === 'stt' && (
            <div className="h-guide-setup">
              <SetupRow
                title={c.speechTitle}
                detail={providerName}
                action={ready ? c.change : c.configure}
                onClick={() => setPanel(panel === 'stt' ? null : 'stt')}
                open={panel === 'stt'}
                status={ready ? c.savedSetup : c.setupNeeded}
              />
              {panel === 'stt' && (
                <section className="h-guide-panel" aria-label={c.speechTitle}>
                  <div className="h-guide-provider-choice">
                    <button
                      aria-pressed={local}
                      onClick={() => {
                        update({ stt_provider: 'builtin-stt' })
                        useAppStore.getState().setSttTestStatus('idle')
                      }}
                    >
                      {c.local}
                    </button>
                    <button
                      aria-pressed={!local}
                      onClick={() => {
                        if (local) {
                          update({ stt_provider: 'custom-whisper' })
                          useAppStore.getState().setSttTestStatus('idle')
                        }
                      }}
                    >
                      {c.external}
                    </button>
                  </div>
                  <p className="h-guide-note">{local ? c.modelHelp : c.testNeeded}</p>
                  <SttPane />
                </section>
              )}
              <SetupRow
                title={c.aiTitle}
                detail={
                  config.polish_enabled
                    ? `${config.llm_provider} · ${config.llm_model}`
                    : c.polishOff
                }
                action={c.change}
                onClick={() => setPanel(panel === 'polish' ? null : 'polish')}
                open={panel === 'polish'}
              />
              {panel === 'polish' && (
                <section className="h-guide-panel" aria-label={c.aiTitle}>
                  <div className="h-guide-provider-choice">
                    <button
                      aria-pressed={config.polish_enabled}
                      onClick={() => update({ polish_enabled: true })}
                    >
                      {c.polishOn}
                    </button>
                    <button
                      aria-pressed={!config.polish_enabled}
                      onClick={() => update({ polish_enabled: false })}
                    >
                      {c.polishOff}
                    </button>
                  </div>
                  {config.polish_enabled && (
                    <>
                      <p className="h-guide-note">{c.polishHelp}</p>
                      <LlmPane />
                      <p className="h-guide-note">
                        {llmTest === 'success' ? c.testPassed : c.aiOptional}
                      </p>
                    </>
                  )}
                </section>
              )}
              <SetupRow
                title={c.hotkey}
                status={needsMacAccessibility(config) && !trusted ? c.permissionNeeded : undefined}
                detail={config.hotkey.split('+').join(' + ')}
                action={c.change}
                onClick={() => setPanel(panel === 'controls' ? null : 'controls')}
                open={panel === 'controls'}
              />
              {panel === 'controls' && (
                <section className="h-guide-panel" aria-label={c.hotkey}>
                  <Controls copy={c} />
                  {hotkeyError && (
                    <p role="alert" className="text-warning text-sm">
                      {hotkeyError}
                    </p>
                  )}
                </section>
              )}
              <p role="status" className="h-guide-note h-guide-setup-note">
                {ready ? c.setupReady : local ? c.modelNeeded : c.testNeeded}
              </p>
              <p className="h-guide-note">{c.permissionHint}</p>
            </div>
          )}
          {step === 'practice' &&
            ([
              'builtin-stt',
              'custom-whisper',
              'glm-asr',
              'openai-whisper',
              'groq-whisper',
              'siliconflow',
            ].includes(config.stt_provider) ? (
              <Practice copy={c} onBusy={setBusy} onSuccess={() => setPracticed(true)} />
            ) : (
              <p className="h-guide-note">{c.unsupported}</p>
            ))}
          {step === 'done' && (
            <>
              <div className="h-guide-finish">
                <span className="h-guide-finish-mark">
                  <Check size={28} strokeWidth={1.5} />
                </span>
                <kbd>{config.hotkey.split('+').join(' + ')}</kbd>
                <p>{config.hotkey_mode === 'hold' ? c.hold : c.toggle}</p>
              </div>
              <p className="h-guide-note">{practiced ? c.practiced : c.noTest}</p>
              <details className="h-guide-more">
                <summary>{c.moreTips}</summary>
                <div>
                  <p>{c.tips[1]}</p>
                  <p>{c.cancelTip}</p>
                  {c.featureDescriptions.map((f) => (
                    <p key={f}>{f}</p>
                  ))}
                  <h2>{c.mobileTitle}</h2>
                  <ol>
                    {c.mobileSteps.map((s) => (
                      <li key={s}>{s}</li>
                    ))}
                  </ol>
                </div>
              </details>
            </>
          )}
        </div>
      </main>
      <footer className="h-guide-footer">
        {error && (
          <p role="alert" className="h-guide-error">
            {error}
          </p>
        )}
        <div className="h-guide-footer-actions">
          <div className="h-guide-back">
            {index > 0 && (
              <button
                disabled={saving || busy}
                className="h-guide-text-button"
                onClick={() => void saveAnd('back')}
              >
                <ArrowLeft size={15} />
                {c.back}
              </button>
            )}
          </div>
          <ol className="h-guide-progress" aria-label={c.name}>
            {steps.map((id, i) => (
              <li key={id} aria-current={id === step ? 'step' : undefined} aria-label={c.labels[i]}>
                <span />
              </li>
            ))}
          </ol>
          <button
            disabled={disabled}
            className={
              step === 'practice' && !practiced ? 'h-guide-text-button' : 'h-guide-primary'
            }
            onClick={() => void saveAnd('next')}
          >
            {saving
              ? c.saving
              : step === 'welcome'
                ? c.begin
                : step === 'done'
                  ? c.finish
                  : step === 'practice' && !practiced
                    ? c.skip
                    : c.next}
            <ArrowRight size={15} />
          </button>
        </div>
      </footer>
    </div>
  )
}
function SetupRow({
  title,
  detail,
  action,
  open,
  status,
  onClick,
}: {
  title: string
  detail: string
  action: string
  open: boolean
  status?: string
  onClick: () => void
}) {
  return (
    <button
      className="h-guide-setup-row"
      onClick={onClick}
      aria-expanded={open}
      aria-label={`${title} ${action}`}
      aria-description={[detail, status].filter(Boolean).join(' · ')}
    >
      <span>
        <strong>{title}</strong>
        <span className="h-guide-row-detail">{detail}</span>
      </span>
      <span className="h-guide-row-action">
        {status && <small>{status}</small>}
        {action}
        <ChevronRight size={15} className={open ? 'rotate-90' : ''} />
      </span>
    </button>
  )
}

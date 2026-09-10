import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { invoke } from '@tauri-apps/api/core'
import { load } from '@tauri-apps/plugin-store'
import { HTutorial } from '../index'
import { useAppStore } from '../../../stores/appStore'
import { updateConfig, getHotkeyRegistrationError } from '../../../lib/tauri'
import { completeTutorial, modelReady, readStep, rememberStep } from '../state'
import { ko } from '../copy'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/plugin-store', () => ({ load: vi.fn() }))
vi.mock('../../../lib/tauri', () => ({
  updateConfig: vi.fn(),
  getHotkeyRegistrationError: vi.fn(),
  setAutoStart: vi.fn(),
}))
vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (s: string) => s, i18n: { resolvedLanguage: 'ko' } }),
}))
vi.mock('../../Settings/SttPane', () => ({ SttPane: () => <p>STT settings</p> }))
vi.mock('../../Settings/LlmPane', () => ({ LlmPane: () => <p>LLM settings</p> }))
vi.mock('../Controls', () => ({ Controls: () => <p>Controls</p> }))
vi.mock('../Practice', () => ({ Practice: () => <p>Practice</p> }))
const initial = structuredClone(useAppStore.getState().config)
beforeEach(() => {
  vi.resetAllMocks()
  localStorage.clear()
  useAppStore.setState({
    config: structuredClone(initial),
    savedConfig: structuredClone(initial),
    onboardingCompleted: false,
    sttTestStatus: 'idle',
    llmTestStatus: 'idle',
    hotkeyRegistrationError: null,
  })
  vi.mocked(updateConfig).mockResolvedValue(undefined)
  vi.mocked(getHotkeyRegistrationError).mockResolvedValue(null)
})
afterEach(cleanup)
it('preserves existing provider and starts review at welcome without discarding resume state', () => {
  rememberStep('stt')
  useAppStore.getState().updateConfig({ stt_provider: 'groq-whisper' })
  render(<HTutorial review onClose={vi.fn()} />)
  expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(ko.titles[0])
  expect(useAppStore.getState().config.stt_provider).toBe('groq-whisper')
  expect(readStep()).toBe('stt')
})
it('blocks navigation on save failure and retains draft', async () => {
  vi.mocked(updateConfig).mockRejectedValue('Disk full')
  render(<HTutorial onClose={vi.fn()} />)
  fireEvent.click(screen.getByText(ko.begin))
  expect(await screen.findByRole('alert')).toHaveTextContent('Disk full')
  expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(ko.titles[0])
  expect(load).not.toHaveBeenCalled()
})
it('can explicitly disable optional polish without clearing provider or prompts', async () => {
  rememberStep('stt')
  useAppStore
    .getState()
    .updateConfig({ polish_enabled: true, polish_custom_prompt: 'keep my prompt' })
  render(<HTutorial onClose={vi.fn()} />)
  useAppStore.setState({ sttTestStatus: 'success' })
  fireEvent.click(screen.getByRole('button', { name: `${ko.aiTitle} ${ko.change}` }))
  fireEvent.click(screen.getByText(ko.polishOff))
  fireEvent.click(screen.getByText(ko.next))
  await screen.findByText('Practice')
  expect(useAppStore.getState().config.polish_enabled).toBe(false)
  expect(useAppStore.getState().config.polish_custom_prompt).toBe('keep my prompt')
})
it('requires an available installed selected model and no active model operation', async () => {
  rememberStep('stt')
  useAppStore.getState().updateConfig({ stt_provider: 'builtin-stt' })
  const status = {
    selected: 'base',
    busy: true,
    models: [{ id: 'base', installed: true, available: true }],
  }
  vi.mocked(invoke).mockResolvedValue(status)
  render(<HTutorial onClose={vi.fn()} />)
  await waitFor(() => expect(invoke).toHaveBeenCalledWith('get_local_stt_status'))
  expect(screen.getByText(ko.next)).toBeDisabled()
  expect(modelReady({ ...status, busy: false })).toBe(true)
  expect(modelReady({ ...status, busy: false, selected: 'missing' })).toBe(false)
})
it('saves for later without persisting completion and resumes the step', async () => {
  rememberStep('stt')
  const close = vi.fn()
  render(<HTutorial onClose={close} />)
  fireEvent.click(screen.getByRole('button', { name: ko.later }))
  await waitFor(() => expect(close).toHaveBeenCalled())
  expect(updateConfig).toHaveBeenCalledOnce()
  expect(load).not.toHaveBeenCalled()
  expect(readStep()).toBe('stt')
})
it('does not mark skipped practice as a recording success', async () => {
  rememberStep('practice')
  render(<HTutorial onClose={vi.fn()} />)
  fireEvent.click(screen.getByText(ko.skip))
  expect(await screen.findByText(ko.noTest)).toBeInTheDocument()
  expect(screen.queryByText(ko.practiced)).toBeNull()
})
it('only completes after settings and completion flag are flushed', async () => {
  const save = vi.fn().mockRejectedValueOnce('Cannot flush').mockResolvedValue(undefined)
  const set = vi.fn().mockResolvedValue(undefined)
  vi.mocked(load).mockResolvedValue({ set, save, get: vi.fn().mockResolvedValue(false) } as never)
  await expect(completeTutorial()).rejects.toBe('Cannot flush')
  expect(useAppStore.getState().onboardingCompleted).toBe(false)
  await completeTutorial()
  expect(useAppStore.getState().onboardingCompleted).toBe(true)
  expect(set).toHaveBeenCalledWith('onboarding_completed', true)
})
it('rejects obsolete or corrupt resume IDs', () => {
  localStorage.setItem('h-tutorial-step-v1', '99')
  expect(readStep()).toBe('welcome')
})

it('keeps setup forms hidden until the matching row is opened', () => {
  rememberStep('stt')
  render(<HTutorial onClose={vi.fn()} />)
  expect(screen.queryByText('STT settings')).toBeNull()
  expect(screen.queryByText('LLM settings')).toBeNull()
  fireEvent.click(screen.getByRole('button', { name: `${ko.speechTitle} ${ko.configure}` }))
  expect(screen.getByText('STT settings')).toBeInTheDocument()
  fireEvent.click(screen.getByRole('button', { name: `${ko.hotkey} ${ko.change}` }))
  expect(screen.queryByText('STT settings')).toBeNull()
  expect(screen.getByText('Controls')).toBeInTheDocument()
})
it('maps older polish and controls progress to compact setup', () => {
  for (const old of ['polish', 'controls']) {
    localStorage.setItem('h-tutorial-step-v1', old)
    expect(readStep()).toBe('stt')
  }
})
it('lets returning users keep an unchanged external setup without retesting', async () => {
  const config = { ...initial, stt_provider: 'groq-whisper' as const }
  useAppStore.setState({ config, savedConfig: config })
  render(<HTutorial review onClose={vi.fn()} />)
  fireEvent.click(screen.getByText(ko.begin))
  await screen.findByText(ko.setupReady)
  expect(screen.getByText(ko.next)).not.toBeDisabled()
  expect(screen.queryByText('STT settings')).toBeNull()
})

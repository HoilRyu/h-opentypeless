import { updateConfig, getHotkeyRegistrationError, setAutoStart } from '../../lib/tauri'
import { useAppStore } from '../../stores/appStore'

export const steps = ['welcome', 'stt', 'practice', 'done'] as const
export type Step = (typeof steps)[number]
const progressKey = 'h-tutorial-step-v1'
export function readStep(): Step {
  try {
    const value = localStorage.getItem(progressKey)
    if (value === 'polish' || value === 'controls') return 'stt'
    return steps.includes(value as Step) ? (value as Step) : 'welcome'
  } catch {
    return 'welcome'
  }
}
export function rememberStep(step: Step) {
  try {
    localStorage.setItem(progressKey, step)
  } catch {
    /* Optional resume hint. */
  }
}
export async function saveTutorialSettings() {
  const state = useAppStore.getState()
  const autoStartChanged =
    state.savedConfig !== null && state.savedConfig.auto_start !== state.config.auto_start
  if (autoStartChanged) await setAutoStart(state.config.auto_start)
  try {
    await updateConfig(state.config)
  } catch (error) {
    if (autoStartChanged && state.savedConfig)
      await setAutoStart(state.savedConfig.auto_start).catch(() => {})
    throw error
  }
  state.setSavedConfig(state.config)
  state.setHotkeyRegistrationError(await getHotkeyRegistrationError().catch(() => null))
}
export async function completeTutorial() {
  await saveTutorialSettings()
  const { load } = await import('@tauri-apps/plugin-store')
  const store = await load('settings.json')
  const previous = await store.get<boolean>('onboarding_completed')
  await store.set('onboarding_completed', true)
  try {
    await store.save()
  } catch (error) {
    await store.set('onboarding_completed', previous ?? false).catch(() => {})
    throw error
  }
  rememberStep('welcome')
  useAppStore.getState().setOnboardingCompleted(true)
}
export interface ModelStatus {
  selected: string | null
  busy: boolean
  models: { id: string; name?: string; installed: boolean; available: boolean }[]
}
export function modelReady(status: ModelStatus | null) {
  return (
    !!status &&
    !status.busy &&
    status.models.some((m) => m.id === status.selected && m.installed && m.available)
  )
}

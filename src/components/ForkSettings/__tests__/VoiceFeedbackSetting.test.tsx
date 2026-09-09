import { afterEach, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { invoke } from '@tauri-apps/api/core'
import { VoiceFeedbackSetting } from '../VoiceFeedbackSetting'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('react-i18next', () => ({ useTranslation: () => ({ i18n: { language: 'ko' } }) }))
const settings = { enabled: true, brightness: 0.55, reactive: true, near_caret: true }
afterEach(() => {
  cleanup()
  vi.resetAllMocks()
})
it('keeps saved settings when persistence fails', async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce({ settings })
    .mockRejectedValueOnce(new Error('disk unavailable'))
  render(<VoiceFeedbackSetting />)
  fireEvent.click(await screen.findByLabelText('화면 그라데이션 테두리'))
  await screen.findByRole('alert')
  expect(screen.getByLabelText('화면 그라데이션 테두리')).toBeChecked()
})
it('previews through the visual command without starting microphone capture', async () => {
  vi.mocked(invoke).mockResolvedValue({ settings })
  render(<VoiceFeedbackSetting />)
  fireEvent.click(await screen.findByRole('button', { name: '테두리 3초 미리 보기' }))
  expect(vi.mocked(invoke).mock.calls.map((c) => c[0])).toEqual([
    'get_voice_feedback',
    'preview_voice_feedback',
  ])
})

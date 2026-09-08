import { afterEach, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { invoke } from '@tauri-apps/api/core'
import { AudioDuckingSetting } from '../AudioDuckingSetting'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('react-i18next', () => ({ useTranslation: () => ({ i18n: { language: 'ko' } }) }))
afterEach(() => {
  cleanup()
  vi.resetAllMocks()
})
it('loads saved mode and saves changes without the upstream dirty bar', async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce({ mode: 'off', volume_percent: 20, active: false, warning: null })
    .mockResolvedValueOnce({ mode: 'reduce', volume_percent: 20, active: false, warning: null })
  render(<AudioDuckingSetting />)
  const select = screen.getByRole('combobox')
  await waitFor(() => expect(select).not.toBeDisabled())
  fireEvent.change(select, { target: { value: 'reduce' } })
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith('set_audio_ducking', { mode: 'reduce', volumePercent: 20 }),
  )
  await waitFor(() => expect(select).toHaveValue('reduce'))
})
it('keeps the saved mode when saving fails', async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce({ mode: 'off', volume_percent: 20, active: false, warning: null })
    .mockRejectedValueOnce(new Error('disk unavailable'))
  render(<AudioDuckingSetting />)
  const select = screen.getByRole('combobox')
  await waitFor(() => expect(select).not.toBeDisabled())
  fireEvent.change(select, { target: { value: 'mute' } })
  await waitFor(() => expect(screen.getByRole('status')).toBeInTheDocument())
  expect(select).toHaveValue('off')
})

it('saves a custom percentage and rejects values above 100', async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce({ mode: 'reduce', volume_percent: 20, active: false, warning: null })
    .mockResolvedValueOnce({ mode: 'reduce', volume_percent: 45, active: false, warning: null })
  render(<AudioDuckingSetting />)
  const input = await screen.findByRole('spinbutton')
  fireEvent.change(input, { target: { value: '45' } })
  fireEvent.blur(input)
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith('set_audio_ducking', { mode: 'reduce', volumePercent: 45 }),
  )
  await waitFor(() => expect(input).not.toBeDisabled())
  fireEvent.change(input, { target: { value: '120' } })
  fireEvent.blur(input)
  expect(screen.getByRole('alert')).toBeInTheDocument()
  expect(invoke).toHaveBeenCalledTimes(2)
})

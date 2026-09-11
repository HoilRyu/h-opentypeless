import { afterEach, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { invoke } from '@tauri-apps/api/core'
import { LocalLlmSetting } from '../LocalLlmSetting'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('react-i18next', () => ({ useTranslation: () => ({ i18n: { language: 'ko' } }) }))
const state = {
  models: ['e2b', 'e4b', '12b'].map((id) => ({
    id,
    name: 'Gemma 4 ' + id.toUpperCase(),
    size: 7e9,
    installed: false,
  })),
  selected: null,
  available: true,
  busy: false,
  running: false,
  progress: { model: '', downloaded: 0, total: 0, phase: '', error: null },
}
afterEach(() => {
  cleanup()
  vi.resetAllMocks()
})
it('only lists approved models and browsing does not download or select', async () => {
  vi.mocked(invoke).mockResolvedValue(state)
  render(<LocalLlmSetting />)
  await waitFor(() => expect(screen.getByRole('button', { name: '다운로드' })).toBeEnabled())
  expect(screen.getAllByRole('option')).toHaveLength(3)
  fireEvent.change(screen.getByRole('combobox'), { target: { value: 'e2b' } })
  expect(invoke).toHaveBeenCalledTimes(1)
  fireEvent.click(screen.getByRole('button', { name: '다운로드' }))
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith('download_local_llm_model', { id: 'e2b' }),
  )
})
it('does not offer activation until the model has downloaded', async () => {
  vi.mocked(invoke).mockResolvedValue(state)
  render(<LocalLlmSetting />)
  await screen.findByText(/미설치/)
  expect(screen.queryByRole('button', { name: '이 모델 사용' })).not.toBeInTheDocument()
})
it('shows errors and allows cancelling downloads while controls are busy', async () => {
  vi.mocked(invoke).mockResolvedValue({
    ...state,
    busy: true,
    progress: { ...state.progress, model: '12b', phase: 'downloading' },
  })
  render(<LocalLlmSetting />)
  const pause = await screen.findByRole('button', { name: '중단 · 나중에 이어받기' })
  expect(screen.getByRole('button', { name: '다운로드' })).toBeDisabled()
  fireEvent.click(pause)
  await waitFor(() => expect(invoke).toHaveBeenCalledWith('cancel_local_llm_download'))
})

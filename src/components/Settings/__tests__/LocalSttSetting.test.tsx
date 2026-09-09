import { describe, it, expect, vi, afterEach } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { invoke } from '@tauri-apps/api/core'
import { LocalSttSetting } from '../LocalSttSetting'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('react-i18next', () => ({ useTranslation: () => ({ t: (key: string) => key }) }))
afterEach(() => {
  cleanup()
  vi.resetAllMocks()
})
const state = (extra = {}) => ({
  models: [
    {
      id: 'base',
      name: 'Whisper Base',
      engine: 'whisper',
      recommended_ram_gb: 4,
      license: 'MIT',
      files: [{ size: 147951465 }],
      installed: false,
      available: true,
      partial_bytes: 0,
    },
  ],
  selected: null,
  busy: false,
  progress: { model: '', phase: '', downloaded: 0, total: 0, error: null },
  ...extra,
})
describe('native model manager', () => {
  it('downloads only after the user requests it', async () => {
    vi.mocked(invoke).mockResolvedValue(state())
    render(<LocalSttSetting />)
    await screen.findByText('Whisper Base')
    expect(invoke).not.toHaveBeenCalledWith('download_local_stt_model', expect.anything())
    fireEvent.click(screen.getByText('h.localStt.download'))
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith('download_local_stt_model', { id: 'base' }),
    )
  })
  it('offers pause during download and forbids selecting/deleting busy files', async () => {
    vi.mocked(invoke).mockResolvedValue(
      state({
        busy: true,
        progress: { model: 'base', phase: 'downloading', downloaded: 10, total: 100, error: null },
      }),
    )
    render(<LocalSttSetting />)
    fireEvent.click(await screen.findByText('h.localStt.pause'))
    await waitFor(() => expect(invoke).toHaveBeenCalledWith('cancel_local_stt_download', {}))
    expect(screen.queryByText('h.localStt.use')).toBeNull()
  })
  it('does not offer a download for an unavailable engine', async () => {
    const s = state()
    s.models[0].available = false
    vi.mocked(invoke).mockResolvedValue(s)
    render(<LocalSttSetting />)
    await screen.findByText('h.localStt.unavailable')
    expect(screen.queryByRole('button')).toBeNull()
  })
  it('displays backend failures', async () => {
    vi.mocked(invoke).mockRejectedValue('Disk full')
    render(<LocalSttSetting />)
    expect(await screen.findByRole('alert')).toHaveTextContent('Disk full')
  })
})

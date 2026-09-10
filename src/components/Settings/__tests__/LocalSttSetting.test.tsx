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
  it('persists preview preference and disables it while recording', async () => {
    vi.mocked(invoke).mockResolvedValue(state({ preview_enabled: true }))
    const view = render(<LocalSttSetting />)
    const checkbox = await screen.findByRole('checkbox')
    expect(checkbox).toBeChecked()
    fireEvent.click(checkbox)
    await waitFor(() => expect(invoke).toHaveBeenCalledWith('set_local_stt_preview', { id: 'off' }))
    view.unmount()
    vi.mocked(invoke).mockResolvedValue(state({ preview_enabled: false, busy: true }))
    render(<LocalSttSetting />)
    expect(await screen.findByRole('checkbox')).toBeDisabled()
  })
  it('downloads only after the user requests it', async () => {
    vi.mocked(invoke).mockResolvedValue(state())
    render(<LocalSttSetting />)
    await screen.findByRole('heading', { name: 'Whisper Base' })
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
  it('lets users explicitly choose CPU and surfaces selection failures', async () => {
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === 'set_local_stt_engine') throw new Error('STT is busy')
      return state({ engine: { preference: 'auto', active: 'mlx', reason: null, resident: false } })
    })
    render(<LocalSttSetting />)
    fireEvent.change(await screen.findByRole('combobox', { name: 'h.localStt.engineTitle' }), {
      target: { value: 'cpu' },
    })
    await waitFor(() => expect(invoke).toHaveBeenCalledWith('set_local_stt_engine', { id: 'cpu' }))
    expect(await screen.findByRole('alert')).toHaveTextContent('STT is busy')
  })
  it('browses one model without downloading or switching the active model', async () => {
    const s = state({ selected: 'base' })
    s.models[0].installed = true
    s.models.push({
      ...s.models[0],
      id: 'large-v3-turbo',
      name: 'Whisper Large-v3 Turbo',
      installed: false,
    })
    vi.mocked(invoke).mockResolvedValue(s)
    render(<LocalSttSetting />)
    const picker = await screen.findByRole('combobox', { name: 'h.localStt.modelLabel' })
    fireEvent.change(picker, { target: { value: 'large-v3-turbo' } })
    expect(screen.getAllByRole('article')).toHaveLength(1)
    expect(screen.getByRole('heading', { name: 'Whisper Large-v3 Turbo' })).toBeInTheDocument()
    expect(invoke).not.toHaveBeenCalledWith('select_local_stt_model', expect.anything())
    expect(invoke).not.toHaveBeenCalledWith('download_local_stt_model', expect.anything())
    fireEvent.click(screen.getByText('h.localStt.download'))
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith('download_local_stt_model', { id: 'large-v3-turbo' }),
    )
  })
  it('keeps a transfer reachable while browsing another model', async () => {
    const s = state({
      busy: true,
      progress: { model: 'base', phase: 'downloading', downloaded: 10, total: 100, error: null },
    })
    s.models.push({ ...s.models[0], id: 'small', name: 'Whisper Small' })
    vi.mocked(invoke).mockResolvedValue(s)
    render(<LocalSttSetting />)
    fireEvent.change(await screen.findByRole('combobox', { name: 'h.localStt.modelLabel' }), {
      target: { value: 'small' },
    })
    fireEvent.click(screen.getByText('h.localStt.showDownload'))
    expect(screen.getByRole('heading', { name: 'Whisper Base' })).toBeInTheDocument()
    expect(screen.getByText('h.localStt.pause')).toBeEnabled()
  })
  it('displays backend failures', async () => {
    vi.mocked(invoke).mockRejectedValue('Disk full')
    render(<LocalSttSetting />)
    expect(await screen.findByRole('alert')).toHaveTextContent('Disk full')
  })
})

import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { RepolishDialog } from '../RepolishDialog'
import { repolishHistory } from '../../../lib/tauri'
import type { HistoryEntry } from '../../../stores/appStore'

vi.mock('react-i18next', () => ({ useTranslation: () => ({ t: (key: string) => key }) }))
vi.mock('../../../lib/tauri', () => ({ repolishHistory: vi.fn() }))
const entry = {
  id: 7,
  raw_text: '이름은 정하지 말자',
  polished_text: '이름은 정하지 마',
} as HistoryEntry
const result = {
  history_id: 7,
  polished_text: '이름은 정하지 말자.',
  model: 'local',
  style: 'minimal',
  prompt_sha256: 'abc',
  elapsed_ms: 120,
}
const clipboard = vi.fn().mockResolvedValue(undefined)

beforeEach(() => {
  vi.clearAllMocks()
  Object.defineProperty(navigator, 'clipboard', {
    configurable: true,
    value: { writeText: clipboard },
  })
  vi.mocked(repolishHistory).mockResolvedValue(result)
})
afterEach(cleanup)

describe('History replay preview', () => {
  it('shows the original and old result without making an automatic request', () => {
    render(<RepolishDialog entry={entry} onClose={vi.fn()} />)
    expect(screen.getByLabelText('history.originalTranscript')).toHaveValue(entry.raw_text)
    expect(screen.getByLabelText('history.previousResult')).toHaveValue(entry.polished_text)
    expect(repolishHistory).not.toHaveBeenCalled()
  })
  it('replays by identity with an explicit style and copies without replacing history', async () => {
    render(<RepolishDialog entry={entry} onClose={vi.fn()} />)
    fireEvent.change(screen.getByRole('combobox'), { target: { value: 'minimal' } })
    fireEvent.click(screen.getByText('history.repolishRun'))
    await waitFor(() =>
      expect(screen.getByLabelText('history.newResult')).toHaveValue(result.polished_text),
    )
    expect(repolishHistory).toHaveBeenCalledWith(7, 'minimal')
    expect(screen.getByLabelText('history.previousResult')).toHaveValue('이름은 정하지 마')
    fireEvent.click(screen.getByText('history.copyOriginal'))
    expect(clipboard).toHaveBeenCalledWith(entry.raw_text)
    fireEvent.click(screen.getByText('history.copyNewResult'))
    expect(clipboard).toHaveBeenCalledWith(result.polished_text)
  })
  it('keeps the previous candidate and original when a later request fails', async () => {
    render(<RepolishDialog entry={entry} onClose={vi.fn()} />)
    fireEvent.click(screen.getByText('history.repolishRun'))
    await screen.findByLabelText('history.newResult')
    vi.mocked(repolishHistory).mockRejectedValue('history.repolishTimeout')
    fireEvent.click(screen.getByText('history.repolishRun'))
    expect(await screen.findByRole('alert')).toHaveTextContent('history.repolishTimeout')
    expect(screen.getByLabelText('history.originalTranscript')).toHaveValue(entry.raw_text)
    expect(screen.getByLabelText('history.newResult')).toHaveValue(result.polished_text)
  })
  it('prevents duplicate requests and lets the user dismiss a pending request', async () => {
    let resolve!: (value: typeof result) => void
    vi.mocked(repolishHistory).mockReturnValue(
      new Promise((r) => {
        resolve = r
      }),
    )
    const close = vi.fn()
    const view = render(<RepolishDialog entry={entry} onClose={close} />)
    fireEvent.click(screen.getByText('history.repolishRun'))
    expect(screen.getByText('history.repolishing')).toBeDisabled()
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' })
    expect(close).toHaveBeenCalledOnce()
    view.unmount()
    resolve(result)
    await Promise.resolve()
    expect(clipboard).not.toHaveBeenCalled()
  })
})

import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Practice } from '../Practice'
import { ko } from '../copy'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }))
let callback: (event: { payload: { id: string; phase: string } }) => void
let resolve: (value: unknown) => void
const off = vi.fn()
beforeEach(() => {
  vi.resetAllMocks()
  vi.mocked(listen).mockImplementation(async (_name, cb) => {
    callback = cb as typeof callback
    return off
  })
  vi.mocked(invoke).mockImplementation((name) =>
    name === 'run_tutorial_recording'
      ? new Promise((r) => {
          resolve = r
        })
      : Promise.resolve(undefined),
  )
})
afterEach(cleanup)
async function begin() {
  fireEvent.click(screen.getByText(ko.start))
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith('run_tutorial_recording', expect.anything()),
  )
  return (
    vi.mocked(invoke).mock.calls.find((call) => call[0] === 'run_tutorial_recording')![1] as {
      id: string
    }
  ).id
}
it('shows only actual nonempty provider results, ignores events from older sessions', async () => {
  const success = vi.fn()
  render(<Practice copy={ko} onBusy={vi.fn()} onSuccess={success} />)
  const id = await begin()
  act(() => callback({ payload: { id: 'old', phase: 'recording' } }))
  expect(screen.queryByText(ko.stop)).toBeNull()
  act(() => callback({ payload: { id, phase: 'recording' } }))
  fireEvent.click(screen.getByText(ko.stop))
  expect(invoke).toHaveBeenCalledWith('control_tutorial_recording', { id, cancel: false })
  await act(async () =>
    resolve({ raw_text: '원문', polished_text: '다듬은 문장', warning: null, processing_ms: 500 }),
  )
  expect(screen.getByText('다듬은 문장')).toBeInTheDocument()
  expect(success).toHaveBeenCalledOnce()
  expect(off).toHaveBeenCalledOnce()
  expect(
    vi
      .mocked(invoke)
      .mock.calls.every(
        ([name]) => name === 'run_tutorial_recording' || name === 'control_tutorial_recording',
      ),
  ).toBe(true)
})
it('cancels on unmount and ignores a late result', async () => {
  const success = vi.fn()
  const view = render(<Practice copy={ko} onBusy={vi.fn()} onSuccess={success} />)
  const id = await begin()
  view.unmount()
  expect(invoke).toHaveBeenCalledWith('control_tutorial_recording', { id, cancel: true })
  await act(async () => resolve({ raw_text: 'late', polished_text: 'late' }))
  expect(success).not.toHaveBeenCalled()
  expect(off).toHaveBeenCalledOnce()
})
it('does not count an empty transcript as success', async () => {
  const success = vi.fn()
  render(<Practice copy={ko} onBusy={vi.fn()} onSuccess={success} />)
  await begin()
  await act(async () =>
    resolve({ raw_text: ' ', polished_text: '', warning: null, processing_ms: 1 }),
  )
  expect(screen.getByRole('alert')).toHaveTextContent(ko.empty)
  expect(success).not.toHaveBeenCalled()
})
it('does not accept a result after explicit cancellation', async () => {
  const success = vi.fn()
  render(<Practice copy={ko} onBusy={vi.fn()} onSuccess={success} />)
  const id = await begin()
  fireEvent.click(screen.getByText(ko.cancel))
  expect(invoke).toHaveBeenCalledWith('control_tutorial_recording', { id, cancel: true })
  await act(async () => resolve({ raw_text: 'late', polished_text: 'late' }))
  expect(screen.getByRole('alert')).toHaveTextContent(ko.cancelled)
  expect(success).not.toHaveBeenCalled()
})

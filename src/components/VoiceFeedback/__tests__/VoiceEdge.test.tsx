import { act, cleanup, render } from '@testing-library/react'
import { afterEach, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { VoiceEdge, type FeedbackSnapshot } from '..'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }))
afterEach(() => {
  cleanup()
  vi.resetAllMocks()
})
it('ignores a late initial snapshot after recording has stopped and removes listeners', async () => {
  const settings = { enabled: true, brightness: 0.55, reactive: true, near_caret: true }
  let resolve!: (s: FeedbackSnapshot) => void
  let event!: (e: { payload: FeedbackSnapshot }) => void
  const off = vi.fn()
  vi.mocked(invoke).mockReturnValue(
    new Promise((r) => {
      resolve = r
    }),
  )
  vi.mocked(listen).mockImplementation(async (name, callback) => {
    if (name === 'voice-feedback:state') event = callback as typeof event
    return off
  })
  const { container, unmount } = render(<VoiceEdge />)
  await act(async () => {})
  await act(async () => event({ payload: { phase: 'idle', revision: 3, settings } }))
  await act(async () => resolve({ phase: 'recording', revision: 2, settings }))
  expect(container.firstChild).not.toHaveClass('active')
  await act(async () => event({ payload: { phase: 'recording', revision: 4, settings } }))
  expect(container.firstChild).toHaveClass('active', 'recording')
  unmount()
  expect(off).toHaveBeenCalledTimes(2)
})

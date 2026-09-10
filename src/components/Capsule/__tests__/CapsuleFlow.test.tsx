import React from 'react'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../../../stores/appStore'
import { setActiveTranslationTarget, stopAskFlow } from '../../../lib/tauri'
import { Capsule } from '../index'

vi.mock('framer-motion', () => ({
  AnimatePresence: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  motion: new Proxy(
    {},
    {
      get:
        (_target, tag: string) =>
        ({ children, ...props }: React.HTMLAttributes<HTMLElement>) =>
          React.createElement(tag, props, children),
    },
  ),
  useReducedMotion: () => true,
}))

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}))

vi.mock('../../../hooks/useCapsuleResize', () => ({
  useCapsuleResize: () => ({ width: 200, height: 36 }),
}))

vi.mock('../../../lib/tauri', () => ({
  abortAskDictation: vi.fn().mockResolvedValue(undefined),
  abortRecording: vi.fn().mockResolvedValue(undefined),
  setActiveTranslationTarget: vi.fn().mockResolvedValue({
    targets: ['en', 'ja'],
    active_target: 'ja',
  }),
  setCapsuleAutoHide: vi.fn().mockResolvedValue(undefined),
  stopAskFlow: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}))

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
  useAppStore.setState(useAppStore.getInitialState())
})

describe('Capsule flow states', () => {
  beforeEach(() => {
    useAppStore.setState({
      pipelineState: 'idle',
      pipelineError: null,
      contextMenuOpen: false,
      contextMenuReady: false,
      translationTargetMenuOpen: false,
      activeVoiceMode: null,
      partialTranscript: '',
    })
  })

  it('changes this recording style without stopping recording or saving global config', async () => {
    useAppStore.setState({
      pipelineState: 'recording',
      config: {
        ...useAppStore.getState().config,
        polish_enabled: true,
        polish_style: 'clean',
        active_scene: null,
      },
    })
    render(<Capsule />)
    fireEvent.pointerDown(screen.getByRole('button', { name: 'Style for this recording' }))
    fireEvent.pointerUp(screen.getByRole('button', { name: 'Style for this recording' }))
    fireEvent.click(screen.getByRole('button', { name: 'Style for this recording' }))
    fireEvent.click(screen.getByRole('menuitemradio', { name: 'settings.polishStyleStructured' }))
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith('set_recording_polish_style', { style: 'structured' }),
    )
    await waitFor(() => expect(useAppStore.getState().polishStyleMenuOpen).toBe(false))
    expect(useAppStore.getState().config.polish_style).toBe('clean')
    expect(invoke).not.toHaveBeenCalledWith('stop_recording')
  })

  it('shows local preview while recording and keeps its size bounded', () => {
    useAppStore.setState({
      pipelineState: 'recording',
      partialTranscript: '설정 화면에 버튼을 추가해 주세요',
      config: { ...useAppStore.getState().config, stt_provider: 'builtin-stt' },
    })
    render(<Capsule />)
    expect(screen.getByRole('status')).toHaveTextContent('설정 화면에 버튼을 추가해 주세요')
    expect(screen.getByRole('status').className).toContain('overflow-hidden')
  })

  it('renders preparing state', () => {
    useAppStore.setState({ pipelineState: 'preparing' })

    render(<Capsule />)

    expect(screen.getByText('capsule.preparing')).toBeInTheDocument()
  })

  it('renders transcribing state with partial transcript when available', () => {
    useAppStore.setState({
      pipelineState: 'transcribing',
      partialTranscript: 'hello world',
    })

    render(<Capsule />)

    expect(screen.getByText(/hello world/)).toBeInTheDocument()
  })

  it('renders thinking state during polishing', () => {
    useAppStore.setState({ pipelineState: 'polishing' })

    render(<Capsule />)

    expect(screen.getByText('capsule.thinking')).toBeInTheDocument()
  })

  it('does not start dictation when the idle capsule is clicked', () => {
    const { container } = render(<Capsule />)
    const shell = container.querySelector('.jelly-capsule')
    expect(shell).toBeTruthy()

    const pointerUp = new Event('pointerup', { bubbles: true })
    Object.defineProperty(pointerUp, 'button', { value: 0 })
    fireEvent(shell as Element, pointerUp)

    expect(invoke).not.toHaveBeenCalledWith('start_recording')
  })

  it('renders Ask recording in the capsule and stops Ask when clicked', () => {
    useAppStore.setState({ pipelineState: 'ask_recording' })

    render(<Capsule />)

    expect(screen.getByText('ask.title')).toBeInTheDocument()
    expect(screen.getByText('ask.title')).toHaveClass('whitespace-nowrap')
    expect(screen.getByText('00:00')).toBeInTheDocument()

    const pointerUp = new Event('pointerup', { bubbles: true })
    Object.defineProperty(pointerUp, 'button', { value: 0 })
    fireEvent(screen.getByText('ask.title'), pointerUp)

    expect(stopAskFlow).toHaveBeenCalledTimes(1)
  })

  it('renders Ask thinking in the capsule', () => {
    useAppStore.setState({ pipelineState: 'ask_thinking' })

    render(<Capsule />)

    expect(screen.getByText('ask.title')).toBeInTheDocument()
    expect(screen.getByText('ask.title')).toHaveClass('whitespace-nowrap')
    expect(screen.getByText('ask.thinking')).toBeInTheDocument()
  })

  it('shows the language chip only while Translate is recording', () => {
    useAppStore.setState({
      pipelineState: 'recording',
      activeVoiceMode: 'dictate',
      config: {
        ...useAppStore.getState().config,
        translation: { targets: ['en', 'ja'], active_target: 'en' },
      },
    })
    const { rerender } = render(<Capsule />)

    expect(screen.queryByRole('button', { name: 'capsule.translationTarget en' })).toBeNull()

    useAppStore.setState({ activeVoiceMode: 'translate' })
    rerender(<Capsule />)
    expect(screen.getByRole('button', { name: 'capsule.translationTarget en' })).toBeInTheDocument()

    useAppStore.setState({ pipelineState: 'transcribing' })
    rerender(<Capsule />)
    expect(screen.queryByRole('button', { name: 'capsule.translationTarget en' })).toBeNull()
  })

  it('switches the active target without restarting or stopping recording', async () => {
    useAppStore.setState({
      pipelineState: 'recording',
      activeVoiceMode: 'translate',
      config: {
        ...useAppStore.getState().config,
        translation: { targets: ['en', 'ja'], active_target: 'en' },
      },
    })
    render(<Capsule />)

    fireEvent.pointerDown(screen.getByRole('button', { name: 'capsule.translationTarget en' }))
    fireEvent.click(screen.getByRole('button', { name: 'capsule.translationTarget en' }))
    fireEvent.click(screen.getByRole('menuitemradio', { name: '日本語' }))

    await waitFor(() => expect(setActiveTranslationTarget).toHaveBeenCalledWith('ja'))
    expect(invoke).not.toHaveBeenCalledWith('stop_recording')
    expect(invoke).not.toHaveBeenCalledWith('start_recording')
    expect(useAppStore.getState().config.translation.active_target).toBe('ja')
  })

  it('keeps the capsule shell at 200 by 36 and closes the target menu on Escape', async () => {
    useAppStore.setState({
      pipelineState: 'recording',
      activeVoiceMode: 'translate',
      config: {
        ...useAppStore.getState().config,
        translation: { targets: ['en', 'ja'], active_target: 'en' },
      },
    })
    const { container } = render(<Capsule />)
    const chip = screen.getByRole('button', { name: 'capsule.translationTarget en' })

    fireEvent.click(chip)
    expect(screen.getByRole('menu', { name: 'capsule.translationTargets' })).toBeInTheDocument()

    fireEvent.keyDown(window, { key: 'Escape' })
    expect(screen.queryByRole('menu', { name: 'capsule.translationTargets' })).toBeNull()
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'capsule.translationTarget en' })).toHaveFocus(),
    )

    const shell = container.querySelector('.jelly-capsule-active') as HTMLElement
    expect(shell.style.width).toBe('360px')
    expect(shell.style.height).toBe('36px')
  })
})

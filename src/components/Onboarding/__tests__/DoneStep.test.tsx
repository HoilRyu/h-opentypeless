import { cleanup, render, screen } from '@testing-library/react'
import React from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { DoneStep } from '../DoneStep'
import * as tauri from '../../../lib/tauri'

const mockConfig = {
  hotkey: 'Fn',
  hotkey_mode: 'toggle',
  output_mode: 'clipboard',
}

vi.mock('framer-motion', () => ({
  motion: new Proxy(
    {},
    {
      get:
        (_target, tag: string) =>
        ({ children, ...props }: any) =>
          React.createElement(tag, props, children),
    },
  ),
}))

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) =>
      ({
        'onboarding.done.title': "You're all set!",
        'onboarding.done.capsuleAppearsWhenRecording':
          'The capsule appears while recording and processing.',
        'onboarding.done.holdPress': 'Hold / Press',
        'onboarding.done.holdPressSub': 'Use your hotkey to start and stop recording',
        'onboarding.test.hold': 'Hold',
        'onboarding.test.press': 'Press',
        'onboarding.done.dragToReposition': 'Drag to Reposition',
        'onboarding.done.dragToRepositionSub': 'When visible, drag the capsule anywhere on screen.',
        'onboarding.done.rightClickMenu': 'Right-click Menu',
        'onboarding.done.rightClickMenuSub': 'Right-click the capsule for more options',
        'onboarding.done.restoreCapsuleSub':
          'Use the tray or Settings to keep the capsule visible while idle.',
      })[key] ?? key,
  }),
}))

vi.mock('../../../stores/appStore', () => ({
  useAppStore: (selector: any) =>
    selector({
      config: mockConfig,
    }),
}))

vi.mock('../../../lib/tauri', () => ({
  checkAccessibilityPermission: vi.fn().mockResolvedValue(true),
  requestAccessibilityPermission: vi.fn().mockResolvedValue(true),
  waitForAccessibilityPermission: vi.fn().mockResolvedValue(true),
  resumeHotkey: vi.fn().mockResolvedValue(undefined),
}))

beforeEach(() => {
  vi.clearAllMocks()
  Object.assign(mockConfig, {
    hotkey: 'Fn',
    hotkey_mode: 'toggle',
    output_mode: 'clipboard',
  })
})

afterEach(() => cleanup())

describe('DoneStep', () => {
  it.each([
    ['macOS', 'Fn', 'toggle', 'Press Fn'],
    ['Windows', 'Ctrl+/', 'hold', 'Hold Ctrl+/'],
    ['Linux', 'Ctrl+/', 'hold', 'Hold Ctrl+/'],
  ])(
    'teaches the current %s dictation shortcut and capsule controls during onboarding',
    (_platform, hotkey, mode, dictationTitle) => {
      Object.assign(mockConfig, {
        hotkey,
        hotkey_mode: mode,
      })

      render(<DoneStep />)

      expect(screen.getByText(dictationTitle)).toBeInTheDocument()
      expect(screen.queryByText(/Ask Anything/)).not.toBeInTheDocument()
      expect(screen.getByText('Drag to Reposition')).toBeInTheDocument()
      expect(screen.getByText('Right-click the capsule for more options')).toBeInTheDocument()
      expect(screen.queryByText('Click Capsule')).not.toBeInTheDocument()
    },
  )

  it('does not show Ask Anything guidance', () => {
    render(<DoneStep />)

    expect(screen.getByText('Press Fn')).toBeInTheDocument()
    expect(screen.queryByText(/Ask Anything/)).not.toBeInTheDocument()
  })

  it('does not repeat the Accessibility grant already handled by Permissions', () => {
    const originalPlatform = window.navigator.platform
    Object.defineProperty(window.navigator, 'platform', {
      value: 'MacIntel',
      configurable: true,
    })
    Object.assign(mockConfig, { output_mode: 'keyboard' })

    try {
      render(<DoneStep />)

      expect(tauri.checkAccessibilityPermission).not.toHaveBeenCalled()
      expect(screen.queryByText('onboarding.done.accessibilityRequired')).not.toBeInTheDocument()
    } finally {
      Object.defineProperty(window.navigator, 'platform', {
        value: originalPlatform,
        configurable: true,
      })
    }
  })
})

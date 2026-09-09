import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import { MobileConnectionSetting } from '../../ForkSettings/MobileConnectionSetting'
const { invoke, writeText } = vi.hoisted(() => ({ invoke: vi.fn(), writeText: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke }))
vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText }))
vi.mock('react-i18next', () => ({ useTranslation: () => ({ i18n: { language: 'ko' } }) }))
const status = {
  config: { enabled: true, address: '192.168.0.123', port: 8787 },
  running: true,
  warning: null,
  addresses: ['192.168.0.123'],
}
beforeEach(() => {
  invoke.mockReset()
  invoke.mockResolvedValue(status)
  writeText.mockReset()
  writeText.mockResolvedValue(undefined)
})
afterEach(cleanup)
it('reads status on mount and refresh without changing the server, including on unmount', async () => {
  const view = render(<MobileConnectionSetting />)
  await screen.findByText('http://192.168.0.123:8787')
  fireEvent.click(screen.getByText('상태 새로고침'))
  await waitFor(() => expect(invoke).toHaveBeenCalledTimes(2))
  view.unmount()
  expect(invoke.mock.calls.every(([command]) => command === 'get_mobile_status')).toBe(true)
})
it('copies the applied address instead of unsaved edits and rejects invalid ports', async () => {
  render(<MobileConnectionSetting />)
  await screen.findByText('http://192.168.0.123:8787')
  fireEvent.click(screen.getByText('고급 설정 · IP와 포트'))
  fireEvent.change(screen.getByLabelText('모바일 연결 IP'), { target: { value: '192.168.0.200' } })
  fireEvent.click(screen.getByText('주소 복사'))
  await waitFor(() => expect(writeText).toHaveBeenCalledWith('http://192.168.0.123:8787'))
  fireEvent.change(screen.getByLabelText('모바일 연결 포트'), { target: { value: '65536' } })
  fireEvent.click(screen.getByText('설정 저장·적용'))
  expect(await screen.findByRole('alert')).toHaveTextContent('1~65535')
  expect(invoke.mock.calls.every(([command]) => command === 'get_mobile_status')).toBe(true)
})
it('disables using the saved config even when the draft is invalid', async () => {
  render(<MobileConnectionSetting />)
  await screen.findByText('http://192.168.0.123:8787')
  fireEvent.change(screen.getByLabelText('모바일 연결 포트'), { target: { value: '0' } })
  fireEvent.click(screen.getByText('연결 끄기'))
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith('set_mobile_config', {
      config: { ...status.config, enabled: false },
    }),
  )
})

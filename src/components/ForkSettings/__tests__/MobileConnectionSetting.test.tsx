import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import { MobileConnectionSetting } from '../MobileConnectionSetting'
import { invoke } from '@tauri-apps/api/core'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('react-i18next', () => ({ useTranslation: () => ({ i18n: { language: 'ko' } }) }))
afterEach(cleanup)
const initial = {
  config: { enabled: false, address: '127.0.0.1', port: 8787 },
  running: false,
  warning: null,
  addresses: ['192.168.0.123'],
}
beforeEach(() => {
  vi.mocked(invoke).mockReset()
  vi.mocked(invoke).mockResolvedValue(initial)
})
it('saves explicit network address and shows the actual listening URL', async () => {
  render(<MobileConnectionSetting />)
  await waitFor(() => expect(screen.getByText('연결 켜기')).not.toBeDisabled())
  fireEvent.change(screen.getByLabelText('모바일 연결 IP'), { target: { value: '192.168.0.123' } })
  vi.mocked(invoke).mockResolvedValue({
    ...initial,
    config: { enabled: false, address: '192.168.0.123', port: 8787 },
    running: false,
  })
  fireEvent.click(screen.getByText('설정 저장·적용'))
  await screen.findByText('http://192.168.0.123:8787')
  expect(invoke).toHaveBeenLastCalledWith('set_mobile_config', {
    config: { enabled: false, address: '192.168.0.123', port: 8787 },
  })
})
it('validates ports and displays a bind failure without claiming to be running', async () => {
  render(<MobileConnectionSetting />)
  await waitFor(() => expect(screen.getByText('연결 켜기')).not.toBeDisabled())
  fireEvent.change(screen.getByLabelText('모바일 연결 포트'), { target: { value: '0' } })
  fireEvent.click(screen.getByText('설정 저장·적용'))
  expect(screen.getByRole('alert').textContent).toContain('1~65535')
  expect(invoke).toHaveBeenCalledTimes(1)
  fireEvent.change(screen.getByLabelText('모바일 연결 포트'), { target: { value: '8787' } })
  vi.mocked(invoke).mockRejectedValue('주소를 사용할 수 없습니다.')
  fireEvent.click(screen.getByText('설정 저장·적용'))
  await screen.findByText('주소를 사용할 수 없습니다.')
  expect(screen.getByText('확인 필요')).toBeTruthy()
  expect(screen.queryByText('연결 대기 중')).toBeNull()
})

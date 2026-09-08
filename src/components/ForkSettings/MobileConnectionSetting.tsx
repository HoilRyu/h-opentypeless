import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'
type Config = { enabled: boolean; address: string; port: number }
type Status = { config: Config; running: boolean; warning: string | null; addresses: string[] }
export function MobileConnectionSetting() {
  const { i18n } = useTranslation()
  const ko = i18n.language.startsWith('ko')
  const [status, setStatus] = useState<Status | null>(null)
  const [address, setAddress] = useState('127.0.0.1')
  const [port, setPort] = useState('8787')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  useEffect(() => {
    let live = true
    invoke<Status>('get_mobile_status')
      .then((s) => {
        if (live) {
          setStatus(s)
          setAddress(s.config.address)
          setPort(String(s.config.port))
        }
      })
      .catch(() => {
        if (live)
          setError(ko ? '모바일 설정을 읽지 못했습니다.' : 'Could not load mobile settings.')
      })
    return () => {
      live = false
    }
  }, [ko])
  async function save(enabled: boolean) {
    const number = Number(port)
    if (!Number.isInteger(number) || number < 1 || number > 65535) {
      setError(ko ? '포트는 1~65535로 입력하세요.' : 'Use a port between 1 and 65535.')
      return
    }
    setBusy(true)
    setError('')
    try {
      const s = await invoke<Status>('set_mobile_config', {
        config: { enabled, address: address.trim(), port: number },
      })
      setStatus(s)
      setAddress(s.config.address)
      setPort(String(s.config.port))
    } catch (e) {
      setError(String(e))
    } finally {
      setBusy(false)
    }
  }
  return (
    <section
      className="space-y-3 rounded-xl border border-border p-4"
      aria-labelledby="h-mobile-label"
    >
      <div className="flex items-center justify-between gap-3">
        <h3 id="h-mobile-label" className="text-sm font-medium text-text-primary">
          {ko ? 'Android 연결' : 'Android connection'}
        </h3>
        <span className="text-xs text-text-secondary">
          {status?.running ? (ko ? '연결 대기 중' : 'Listening') : ko ? '꺼짐' : 'Off'}
        </span>
      </div>
      <p className="text-xs text-text-tertiary">
        {ko
          ? 'H 앱을 켜 둔 상태에서 같은 내부 네트워크나 WireGuard로 연결하세요. 연결 키 없이 이 주소에 접근 가능한 기기가 사용할 수 있습니다.'
          : 'Connect over your local network or WireGuard while H is open. Devices that can reach this address can use it without a connection key.'}
      </p>
      <div className="flex gap-3">
        <label className="flex-1 text-xs text-text-secondary">
          {ko ? '이 컴퓨터의 내부 IP' : 'This computer’s private IP'}
          <input
            aria-label={ko ? '모바일 연결 IP' : 'Mobile connection IP'}
            list="h-mobile-addresses"
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            disabled={busy}
            className="mt-1 w-full rounded-lg border border-border bg-bg-secondary px-3 py-2 text-sm text-text-primary"
          />
        </label>
        <label className="w-24 text-xs text-text-secondary">
          {ko ? '포트' : 'Port'}
          <input
            aria-label={ko ? '모바일 연결 포트' : 'Mobile connection port'}
            value={port}
            type="number"
            min={1}
            max={65535}
            onChange={(e) => setPort(e.target.value)}
            disabled={busy}
            className="mt-1 w-full rounded-lg border border-border bg-bg-secondary px-3 py-2 text-sm text-text-primary"
          />
        </label>
      </div>
      <datalist id="h-mobile-addresses">
        {status?.addresses.map((ip) => (
          <option key={ip} value={ip} />
        ))}
      </datalist>
      {status?.running && (
        <p className="text-sm text-text-primary select-all">
          http://{status.config.address}:{status.config.port}
        </p>
      )}
      {address === '127.0.0.1' && (
        <p className="text-xs text-text-tertiary">
          {ko
            ? '127.0.0.1은 이 컴퓨터 전용입니다. 휴대폰에서 연결하려면 내부 IP를 선택하세요.'
            : '127.0.0.1 is local to this computer. Choose a private IP for your phone.'}
        </p>
      )}
      <p className="text-xs text-text-tertiary">
        {ko
          ? '현재 모바일 음성 인식: Local / Custom Whisper 또는 Whisper 호환 제공자. AI 다듬기는 H의 기존 설정을 사용합니다.'
          : 'Mobile STT: Local / Custom Whisper or a Whisper-compatible provider. Polish uses your H settings.'}
      </p>
      <div className="flex gap-2">
        <button
          disabled={!status || busy}
          onClick={() => void save(true)}
          className="rounded-lg bg-accent px-3 py-2 text-sm text-white disabled:opacity-50"
        >
          {ko ? '저장하고 연결 켜기' : 'Save and enable'}
        </button>
        {status?.config.enabled && (
          <button
            disabled={busy}
            onClick={() => {
              setBusy(true)
              invoke<Status>('set_mobile_config', { config: { ...status.config, enabled: false } })
                .then(setStatus)
                .catch((e) => setError(String(e)))
                .finally(() => setBusy(false))
            }}
            className="rounded-lg border border-border px-3 py-2 text-sm text-text-primary"
          >
            {ko ? '연결 끄기' : 'Disable'}
          </button>
        )}
      </div>
      {(error || status?.warning) && (
        <p role="alert" className="text-xs text-amber-500">
          {error || status?.warning}
        </p>
      )}
    </section>
  )
}

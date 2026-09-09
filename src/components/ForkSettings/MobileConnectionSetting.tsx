import { writeText } from '@tauri-apps/plugin-clipboard-manager'
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
  const [copied, setCopied] = useState(false)
  async function refresh() {
    setBusy(true)
    setError('')
    setCopied(false)
    try {
      setStatus(await invoke<Status>('get_mobile_status'))
    } catch (e) {
      setError(String(e))
    } finally {
      setBusy(false)
    }
  }
  const url = status
    ? `http://${status.config.address.includes(':') ? `[${status.config.address}]` : status.config.address}:${status.config.port}`
    : ''
  async function copyAddress() {
    try {
      await writeText(url)
      setCopied(true)
    } catch (e) {
      setError(String(e))
    }
  }
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
    setCopied(false)
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
          {ko ? '연결 상태' : 'Connection status'}
        </h3>
        <span className="text-xs text-text-secondary">
          {error || status?.warning
            ? ko
              ? '확인 필요'
              : 'Needs attention'
            : !status
              ? ko
                ? '불러오는 중'
                : 'Loading'
              : status.running
                ? ko
                  ? '연결 대기 중'
                  : 'Listening'
                : ko
                  ? '꺼짐'
                  : 'Off'}
        </span>
      </div>
      <p className="text-xs text-text-tertiary">
        {ko
          ? 'H 앱을 켜 둔 상태에서 같은 내부 네트워크나 WireGuard로 연결하세요. 연결 키 없이 이 주소에 접근 가능한 기기가 사용할 수 있습니다.'
          : 'Connect over your local network or WireGuard while H is open. Devices that can reach this address can use it without a connection key.'}
      </p>
      <div className="rounded-lg bg-bg-secondary p-4 space-y-3">
        <p className="text-xs text-text-secondary">
          {ko ? '휴대폰에 입력할 주소' : 'Address for your phone'}
        </p>
        <p className="break-all font-mono text-sm select-all">{url || '—'}</p>
        <div className="flex flex-wrap gap-3">
          <button
            disabled={!status || busy}
            onClick={() => void copyAddress()}
            className="rounded-lg border border-border px-3 py-2 text-sm disabled:opacity-50"
          >
            {copied ? (ko ? '복사됨' : 'Copied') : ko ? '주소 복사' : 'Copy address'}
          </button>
          <button
            disabled={busy}
            onClick={() => void refresh()}
            className="rounded-lg border border-border px-3 py-2 text-sm disabled:opacity-50"
          >
            {ko ? '상태 새로고침' : 'Refresh status'}
          </button>
        </div>
        <span role="status" className="sr-only">
          {copied ? (ko ? '연결 주소가 복사되었습니다.' : 'Address copied.') : ''}
        </span>
      </div>
      <details className="rounded-lg border border-border p-3">
        <summary className="cursor-pointer text-sm text-text-secondary">
          {ko ? '고급 설정 · IP와 포트' : 'Advanced · IP and port'}
        </summary>
        <div className="mt-3 flex flex-wrap gap-3">
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
        {address === '127.0.0.1' && (
          <p className="text-xs text-text-tertiary">
            {ko
              ? '127.0.0.1은 이 컴퓨터 전용입니다. 휴대폰에서 연결하려면 내부 IP를 선택하세요.'
              : '127.0.0.1 is local to this computer. Choose a private IP for your phone.'}
          </p>
        )}
        <button
          disabled={!status || busy}
          onClick={() => void save(status?.config.enabled ?? false)}
          className="mt-3 rounded-lg border border-border px-3 py-2 text-sm disabled:opacity-50"
        >
          {ko ? '설정 저장·적용' : 'Save and apply'}
        </button>
      </details>
      <div className="flex gap-2">
        <button
          disabled={!status || busy || status.running}
          onClick={() => {
            if (!status) return
            setBusy(true)
            setError('')
            invoke<Status>('set_mobile_config', { config: { ...status.config, enabled: true } })
              .then(setStatus)
              .catch((e) => setError(String(e)))
              .finally(() => setBusy(false))
          }}
          className="rounded-lg bg-accent px-3 py-2 text-sm text-white disabled:opacity-50"
        >
          {ko ? '연결 켜기' : 'Enable connection'}
        </button>
        {status?.config.enabled && (
          <button
            disabled={busy}
            onClick={() => {
              setBusy(true)
              setError('')
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

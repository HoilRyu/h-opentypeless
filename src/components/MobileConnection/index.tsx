import { Smartphone } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { MobileConnectionSetting } from '../ForkSettings/MobileConnectionSetting'
import { useAppStore } from '../../stores/appStore'

export function MobileConnection() {
  const { i18n } = useTranslation()
  const ko = i18n.language.startsWith('ko')
  const config = useAppStore((s) => s.config)
  const supported = [
    'custom-whisper',
    'glm-asr',
    'openai-whisper',
    'groq-whisper',
    'siliconflow',
  ].includes(config.stt_provider)
  const link = 'text-accent underline underline-offset-4'
  return (
    <div className="mx-auto max-w-3xl space-y-6 p-6 sm:p-8">
      <header className="space-y-2">
        <h2 className="flex items-center gap-3 text-xl font-semibold text-text-primary">
          <Smartphone size={22} className="text-accent" />
          {ko ? '모바일 연결' : 'Mobile connection'}
        </h2>
        <p className="text-sm text-text-secondary">
          {ko
            ? 'Android 음성 키보드를 이 컴퓨터의 음성 인식과 AI 다듬기에 연결합니다.'
            : 'Connect your Android voice keyboard to speech recognition and AI polish on this computer.'}
        </p>
      </header>
      <MobileConnectionSetting />
      <section className="space-y-3 rounded-xl border border-border p-5">
        <h3 className="text-sm font-medium">{ko ? '휴대폰에서 연결하기' : 'Connect your phone'}</h3>
        <ol className="list-decimal space-y-2 pl-5 text-sm text-text-secondary">
          <li>
            {ko
              ? '휴대폰을 같은 내부 네트워크 또는 WireGuard에 연결하세요.'
              : 'Connect your phone to the same local network or WireGuard.'}
          </li>
          <li>
            {ko
              ? 'Android 앱의 서버 주소에 위 연결 주소를 입력하세요.'
              : 'Enter the connection address above in the Android app’s server settings.'}
          </li>
          <li>
            {ko
              ? 'H 앱을 켜 둔 상태에서 음성 키보드를 사용하세요.'
              : 'Keep H running while using the voice keyboard.'}
          </li>
        </ol>
      </section>
      <section className="space-y-3 rounded-xl border border-border p-5">
        <h3 className="text-sm font-medium">{ko ? '사용 중인 모델 설정' : 'Model settings'}</h3>
        <p className="break-words text-sm text-text-secondary">
          STT · {config.stt_provider}
          <br />
          AI · {config.polish_enabled ? config.llm_provider : ko ? '다듬기 꺼짐' : 'Polish off'}
        </p>
        {!supported && (
          <p role="status" className="text-sm text-amber-500">
            {ko
              ? '현재 STT 제공자는 모바일 음성을 지원하지 않습니다. Local / Custom Whisper 또는 Whisper 호환 제공자로 변경하세요.'
              : 'This STT provider does not support mobile audio. Choose Local / Custom Whisper or a Whisper-compatible provider.'}
          </p>
        )}
        <div className="flex flex-wrap gap-4 text-sm">
          <a className={link} href="#/settings?pane=stt">
            {ko ? '음성 인식 설정' : 'Speech settings'}
          </a>
          <a className={link} href="#/settings?pane=llm">
            {ko ? 'AI 다듬기 설정' : 'AI polish settings'}
          </a>
        </div>
      </section>
    </div>
  )
}

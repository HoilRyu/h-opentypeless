import React from 'react'
import { createRoot } from 'react-dom/client'
import { Capsule } from '../../../src/components/Capsule'
import { useAppStore } from '../../../src/stores/appStore'
import '../../../src/i18n'
import '../../../src/styles/globals.css'
import '../../../src/components/VoiceFeedback/voiceFeedback.css'
window.__TAURI_INTERNALS__ = { invoke: async () => undefined } as any
useAppStore.setState({config:{...useAppStore.getState().config, stt_provider:'builtin-stt'},pipelineState:'recording',partialTranscript:'자료 조사는 민지가 하고 검토는 준호가 해요.'})
;(window as any).capsuleStore=useAppStore
createRoot(document.getElementById('root')!).render(<Capsule />)

import { isMacPlatform } from '../stores/appStore'
export function getCapsuleShellSize(
  state: string,
  mac = isMacPlatform(),
  preview = false,
  styleMenu = false,
) {
  if (state === 'recording' && styleMenu)
    return { width: 360, height: (mac ? 44 : 36) + (preview ? 56 : 0) + 64 }
  if (state === 'recording' && preview) return { width: 360, height: mac ? 100 : 92 }
  const height = mac && state !== 'idle' ? 44 : 36
  switch (state) {
    case 'idle':
      return { width: 36, height }
    case 'preparing':
      return { width: mac ? 208 : 180, height }
    case 'outputting':
      return { width: mac ? 160 : 144, height }
    case 'ask_recording':
    case 'ask_thinking':
      return { width: mac ? 248 : 168, height }
    case 'recording':
      return { width: 360, height }
    default:
      return { width: mac ? 232 : 200, height }
  }
}

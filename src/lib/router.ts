import { H_MANAGED_CLOUD_ENABLED } from './h-features'
import { useState, useEffect, useCallback } from 'react'

export type Route = 'home' | 'mobile' | 'settings' | 'history' | 'upgrade' | 'account' | 'tutorial'

export function parseHash(): Route {
  const hash = window.location.hash.replace('#/', '')
  if (!H_MANAGED_CLOUD_ENABLED && hash === 'tutorial') return 'tutorial'
  if (hash === 'history' || hash === 'mobile') return hash
  if (H_MANAGED_CLOUD_ENABLED && (hash === 'upgrade' || hash === 'account')) return hash
  if (hash === 'settings' || hash.startsWith('settings?')) return 'settings'
  return 'home'
}

export function useRoute() {
  const [route, setRoute] = useState<Route>(parseHash)

  useEffect(() => {
    const onHashChange = () => setRoute(parseHash())
    window.addEventListener('hashchange', onHashChange)
    return () => window.removeEventListener('hashchange', onHashChange)
  }, [])

  const navigate = useCallback((r: Route) => {
    window.location.hash = r === 'home' ? '#/' : `#/${r}`
  }, [])

  return { route, navigate }
}

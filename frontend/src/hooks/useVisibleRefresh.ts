import { useEffect, useRef } from 'react'

/** Pick up imports from other devices while this page is visible. */
export function useVisibleRefresh(refresh: () => Promise<unknown>, intervalMs = 15_000) {
  const current = useRef(refresh)
  useEffect(() => { current.current = refresh })
  useEffect(() => {
    let running = false
    const update = async () => {
      if (document.visibilityState !== 'visible' || running) return
      running = true
      try { await current.current() }
      finally { running = false }
    }
    // Pages report request failures without throwing away their previous data.
    const schedule = () => { void update().catch(() => {}) }
    const timer = window.setInterval(schedule, intervalMs)
    window.addEventListener('focus', schedule)
    document.addEventListener('visibilitychange', schedule)
    return () => {
      window.clearInterval(timer)
      window.removeEventListener('focus', schedule)
      document.removeEventListener('visibilitychange', schedule)
    }
  }, [intervalMs])
}

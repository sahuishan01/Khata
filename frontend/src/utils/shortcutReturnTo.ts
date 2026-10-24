const destinations = new Set(['/add', '/transactions', '/chat', '/upload'])

// A sign-in return target can never be an external URL or an authentication loop.
export function shortcutReturnTo(state: unknown): string {
  const value = (state as { returnTo?: unknown } | null)?.returnTo
  return typeof value === 'string' && destinations.has(value) ? value : '/'
}

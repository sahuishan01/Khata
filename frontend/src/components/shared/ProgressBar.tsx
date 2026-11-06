interface ProgressBarProps {
  pct: number
  color?: string
}

/** Budget meter. Fill is state (green → amber at 80% → red at 100%). */
export function ProgressBar({ pct, color }: ProgressBarProps) {
  const barColor = color || (pct >= 100 ? 'var(--expense)' : pct >= 80 ? 'var(--warn)' : 'var(--income)')
  return (
    <div
      role="progressbar" aria-valuenow={Math.round(pct)} aria-valuemin={0} aria-valuemax={100}
      style={{ height: 6, background: 'var(--surface-3)', borderRadius: 3, overflow: 'hidden' }}
    >
      <div style={{ width: `${Math.min(Math.max(pct, 0), 100)}%`, height: '100%', borderRadius: 3, background: barColor, transition: 'width .3s var(--ease)' }} />
    </div>
  )
}

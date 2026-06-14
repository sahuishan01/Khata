interface ProgressBarProps {
  pct: number
  color?: string
}

export function ProgressBar({ pct, color }: ProgressBarProps) {
  const barColor = color || (pct >= 100 ? 'var(--expense)' : pct >= 80 ? 'var(--warn)' : 'var(--income)')
  return (
    <div style={{ height: 6, background: 'var(--hairline)', borderRadius: 3, overflow: 'hidden' }}>
      <div style={{ width: `${Math.min(pct, 100)}%`, height: '100%', borderRadius: 3, background: barColor, transition: 'width 0.3s' }} />
    </div>
  )
}

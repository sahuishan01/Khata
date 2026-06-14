import { ReactNode } from 'react'

interface ChipProps {
  children: ReactNode
  color?: 'purple' | 'green' | 'red' | 'amber' | 'gray'
  onClick?: () => void
  title?: string
  style?: React.CSSProperties
}

const chipColors = {
  purple: { bg: 'var(--brand-soft)', color: 'var(--brand)' },
  green: { bg: 'var(--income-soft)', color: 'var(--income)' },
  red: { bg: 'var(--expense-soft)', color: 'var(--expense)' },
  amber: { bg: 'rgba(224,163,58,.14)', color: 'var(--warn)' },
  gray: { bg: 'var(--surface-2)', color: 'var(--text-2)' },
}

export function Chip({ children, color = 'gray', onClick, title, style }: ChipProps) {
  const c = chipColors[color]
  return (
    <span
      title={title}
      onClick={onClick}
      style={{
        display: 'inline-flex', alignItems: 'center', gap: 4,
        padding: '2px 8px', borderRadius: 999, fontSize: 11, fontWeight: 600,
        lineHeight: 1.4, background: c.bg, color: c.color,
        maxWidth: '100%', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
        cursor: onClick ? 'pointer' : undefined, transition: 'opacity 0.12s',
        ...style,
      }}
    >
      {children}
    </span>
  )
}

export function Pill({ children, active, onClick }: { children: ReactNode; active?: boolean; onClick?: () => void }) {
  return (
    <button
      onClick={onClick}
      style={{
        padding: '4px 10px', fontSize: 12, borderRadius: 999, fontWeight: 500,
        border: active ? 'none' : '1px solid var(--hairline)',
        background: active ? 'var(--brand)' : 'var(--surface-2)',
        color: active ? 'white' : 'var(--text)',
        cursor: 'pointer', transition: 'all 0.12s', minHeight: 32,
      }}
    >
      {children}
    </button>
  )
}

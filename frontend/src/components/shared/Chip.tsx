import type { ReactNode } from 'react'

interface ChipProps {
  children: ReactNode
  color?: 'purple' | 'brand' | 'green' | 'red' | 'amber' | 'gray'
  onClick?: () => void
  title?: string
  style?: React.CSSProperties
}

const chipColors = {
  purple: { bg: 'var(--brand-soft)', color: 'var(--brand)' },
  brand: { bg: 'var(--brand-soft)', color: 'var(--brand)' },
  green: { bg: 'var(--income-soft)', color: 'var(--income)' },
  red: { bg: 'var(--expense-soft)', color: 'var(--expense)' },
  amber: { bg: 'var(--warn-soft)', color: 'var(--warn)' },
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
        padding: '3px 10px', borderRadius: 'var(--r-pill)', fontSize: 'var(--fs-caption)', fontWeight: 600,
        lineHeight: 1.4, background: c.bg, color: c.color,
        maxWidth: '100%', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
        cursor: onClick ? 'pointer' : undefined,
        ...style,
      }}
    >
      {children}
    </span>
  )
}

/** Segmented filter pill. Selected = brand (an action state, DESIGN.md §1). */
export function Pill({ children, active, onClick }: { children: ReactNode; active?: boolean; onClick?: () => void }) {
  return (
    <button
      onClick={onClick}
      aria-pressed={active}
      style={{
        minHeight: 32, padding: '0 14px', fontSize: 'var(--fs-caption)', fontWeight: 600,
        borderRadius: 'var(--r-pill)',
        border: `1px solid ${active ? 'var(--brand)' : 'var(--hairline)'}`,
        background: active ? 'var(--brand-soft)' : 'transparent',
        color: active ? 'var(--text)' : 'var(--text-2)',
        transition: 'var(--transition-fast)',
        whiteSpace: 'nowrap',
      }}
    >
      {children}
    </button>
  )
}

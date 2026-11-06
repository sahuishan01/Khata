import { formatINR } from '../../utils/format'

interface AmountProps {
  paise: number
  /** Signed context (transaction amounts, deltas): prefix +/− and color by sign. */
  sign?: boolean
  /**
   * Color override for unsigned totals. Unsigned amounts are neutral by
   * default (DESIGN.md §3.5) — a "Total spent" figure is not income.
   */
  tone?: 'income' | 'expense' | 'neutral' | 'muted'
  size?: 'sm' | 'md' | 'lg' | 'xl'
  className?: string
}

const sizes = { sm: 13, md: 15, lg: 22, xl: 40 }

const toneColor = {
  income: 'var(--income)',
  expense: 'var(--expense)',
  neutral: 'var(--text)',
  muted: 'var(--text-2)',
}

export function Amount({ paise, sign, tone, size = 'sm', className }: AmountProps) {
  const color = tone
    ? toneColor[tone]
    : sign
      ? (paise < 0 ? 'var(--expense)' : paise > 0 ? 'var(--income)' : 'var(--text)')
      : 'var(--text)'
  return (
    <span
      className={`figure${className ? ` ${className}` : ''}`}
      style={{ fontSize: sizes[size], color, lineHeight: 1.15 }}
    >
      {formatINR(paise, { sign })}
    </span>
  )
}

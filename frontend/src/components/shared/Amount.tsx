import { formatINR } from '../../utils/format'

interface AmountProps {
  paise: number
  sign?: boolean
  size?: 'sm' | 'md' | 'lg'
  className?: string
}

const sizeStyles = {
  sm: { fontSize: 12 },
  md: { fontSize: 15 },
  lg: { fontSize: 22 },
}

export function Amount({ paise, sign, size = 'md', className }: AmountProps) {
  return (
    <span
      className={className}
      style={{
        fontVariantNumeric: 'tabular-nums',
        fontWeight: 600,
        whiteSpace: 'nowrap',
        ...sizeStyles[size],
        color: paise < 0 ? 'var(--expense)' : paise > 0 ? 'var(--income)' : 'var(--text)',
      }}
    >
      {formatINR(paise, { sign })}
    </span>
  )
}

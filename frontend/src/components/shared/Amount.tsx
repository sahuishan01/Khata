import { formatINR } from '../../utils/format'

interface AmountProps {
  paise: number
  sign?: boolean
  size?: 'sm' | 'md' | 'lg'
  className?: string
}

const sizeStyles = {
  sm: { fontSize: 11.5 },
  md: { fontSize: 14 },
  lg: { fontSize: 22 },
}

export function Amount({ paise, sign, size = 'sm', className }: AmountProps) {
  return (
    <span
      className={className}
      style={{
        fontVariantNumeric: 'tabular-nums',
        fontFeatureSettings: '"tnum" 1',
        fontFamily: 'var(--font-mono, "Roboto Mono", ui-monospace, SFMono-Regular, Menlo, monospace)',
        fontWeight: 700,
        whiteSpace: 'nowrap',
        ...sizeStyles[size],
        color: paise < 0 ? 'var(--expense)' : paise > 0 ? 'var(--income)' : 'var(--text)',
      }}
    >
      {formatINR(paise, { sign })}
    </span>
  )
}

import { ReactNode } from 'react'

interface StatCardProps {
  icon: ReactNode
  label: string
  value: ReactNode
  color?: 'brand' | 'income' | 'expense' | 'amber'
}

const colors = {
  brand: { bg: 'var(--brand-soft)', color: 'var(--brand)' },
  income: { bg: 'var(--income-soft)', color: 'var(--income)' },
  expense: { bg: 'var(--expense-soft)', color: 'var(--expense)' },
  amber: { bg: 'rgba(224,163,58,.14)', color: 'var(--warn)' },
}

export function StatCard({ icon, label, value, color = 'brand' }: StatCardProps) {
  const c = colors[color]
  return (
    <div style={{ background: 'var(--surface)', border: '1px solid var(--hairline)', borderRadius: 12, padding: 11, boxShadow: 'inset 0 1px 0 rgba(255,255,255,0.04)' }}>
      <div style={{ fontSize: 9.5, color: 'var(--text-2)' }}>{label}</div>
      <div style={{ fontSize: 16, fontWeight: 700, marginTop: 4, color: c.color, fontVariantNumeric: 'tabular-nums', fontFeatureSettings: '"tnum" 1' }}>
        {value}
      </div>
    </div>
  )
}

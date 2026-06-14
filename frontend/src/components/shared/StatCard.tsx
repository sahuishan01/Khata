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
    <div className="card" style={{ padding: 18, display: 'flex', flexDirection: 'column', gap: 12, cursor: 'default' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        <div style={{ width: 36, height: 36, borderRadius: 8, background: c.bg, color: c.color, display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 15, flexShrink: 0 }}>
          {icon}
        </div>
        <span style={{ fontSize: 11, fontWeight: 600, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>{label}</span>
      </div>
      <div style={{ fontSize: 22, fontWeight: 700, color: 'var(--text)', letterSpacing: '-0.5px', fontVariantNumeric: 'tabular-nums', lineHeight: 1 }}>
        {value}
      </div>
    </div>
  )
}

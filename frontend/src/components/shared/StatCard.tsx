import type { ReactNode } from 'react'

interface StatCardProps {
  icon?: ReactNode
  label: string
  value: ReactNode
  /** Tints the icon only; the value color comes from the value itself. */
  color?: 'brand' | 'income' | 'expense' | 'amber'
  hint?: string
}

const tints = {
  brand: 'var(--text-2)',
  income: 'var(--income)',
  expense: 'var(--expense)',
  amber: 'var(--warn)',
}

export function StatCard({ icon, label, value, color = 'brand', hint }: StatCardProps) {
  return (
    <div className="stat-card">
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, color: tints[color] }}>
        {icon}
        <span className="stat-label">{label}</span>
      </div>
      <div className="stat-value">{value}</div>
      {hint && <div style={{ fontSize: 'var(--fs-caption)', color: 'var(--text-2)' }}>{hint}</div>}
    </div>
  )
}

import type { ReactNode } from 'react'

export function ChartTooltip({ title, children }: { title?: ReactNode; children: ReactNode }) {
  return (
    <div style={{
      background: 'var(--surface-2)',
      border: '1px solid var(--rule)',
      borderRadius: 'var(--r-md)',
      padding: '10px 12px',
      boxShadow: 'var(--shadow-md)',
      minWidth: 160,
    }}>
      {title && <div className="mono" style={{ fontSize: 12, color: 'var(--text-2)', marginBottom: 6 }}>{title}</div>}
      {children}
    </div>
  )
}

export function TooltipRow({ swatch, label, value }: { swatch: string; label: string; value: string }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 13, marginTop: 4 }}>
      <span style={{ width: 8, height: 8, borderRadius: 2, background: swatch, flexShrink: 0 }} />
      <span style={{ color: 'var(--text-2)', flex: 1 }}>{label}</span>
      <span className="figure" style={{ color: 'var(--text)', fontSize: 13 }}>{value}</span>
    </div>
  )
}

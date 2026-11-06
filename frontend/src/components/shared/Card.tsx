import type { ReactNode } from 'react'

export function Card({ children, style, className, onClick }: { children: ReactNode; style?: React.CSSProperties; className?: string; onClick?: () => void }) {
  return (
    <div
      className={className}
      onClick={onClick}
      style={{
        background: 'var(--surface)',
        border: '1px solid var(--hairline)',
        borderRadius: 'var(--r-lg)',
        overflow: 'hidden',
        minWidth: 0,
        ...style,
      }}
    >
      {children}
    </div>
  )
}

export function CardHeader({ title, subtitle, action }: { title: string; subtitle?: string; action?: ReactNode }) {
  return (
    <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', gap: 12, padding: '20px 20px 0' }}>
      <div style={{ minWidth: 0 }}>
        <h2 style={{ fontSize: 'var(--fs-title)', fontWeight: 650, letterSpacing: '-0.01em' }}>{title}</h2>
        {subtitle && <p style={{ fontSize: 'var(--fs-small)', color: 'var(--text-2)', marginTop: 2 }}>{subtitle}</p>}
      </div>
      {action}
    </div>
  )
}

export function CardBody({ children, style }: { children: ReactNode; style?: React.CSSProperties }) {
  return <div style={{ padding: 20, ...style }}>{children}</div>
}

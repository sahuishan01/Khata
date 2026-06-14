import { ReactNode } from 'react'

const cardStyle: React.CSSProperties = {
  background: 'var(--surface)',
  border: '1px solid var(--hairline)',
  borderRadius: 16,
  position: 'relative',
  overflow: 'hidden',
}

export function Card({ children, style, className, onClick }: { children: ReactNode; style?: React.CSSProperties; className?: string; onClick?: () => void }) {
  return (
    <div className={className} style={{ ...cardStyle, ...style }} onClick={onClick}>
      <div style={{ position: 'absolute', top: 0, left: 1, right: 1, height: 1, background: 'rgba(255,255,255,0.04)', borderRadius: '16px 16px 0 0', pointerEvents: 'none' }} />
      <div style={{ position: 'relative' }}>{children}</div>
    </div>
  )
}

export function CardHeader({ title, action }: { title: string; action?: ReactNode }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '16px 16px 0' }}>
      <span style={{ fontSize: 11, fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.08em' }}>{title}</span>
      {action}
    </div>
  )
}

export function CardBody({ children, style }: { children: ReactNode; style?: React.CSSProperties }) {
  return <div style={{ padding: 16, ...style }}>{children}</div>
}

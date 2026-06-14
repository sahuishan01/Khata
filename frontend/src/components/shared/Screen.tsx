import { ReactNode } from 'react'

export function Screen({ title, subtitle, children, actions }: { title: string; subtitle?: string; children: ReactNode; actions?: ReactNode }) {
  return (
    <div>
      <div className="flex items-center justify-between" style={{ marginBottom: 20, flexWrap: 'wrap', gap: 10 }}>
        <div>
          <h1 className="page-title" style={{ marginBottom: subtitle ? 4 : 0 }}>{title}</h1>
          {subtitle && <p className="text-muted">{subtitle}</p>}
        </div>
        {actions && <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>{actions}</div>}
      </div>
      {children}
    </div>
  )
}

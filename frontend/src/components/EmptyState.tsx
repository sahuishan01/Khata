import { ReactNode } from 'react'

interface EmptyStateProps {
  icon: string
  title: string
  description: string
  action?: { label: string; onClick: () => void }
}

export function EmptyState({ icon, title, description, action }: EmptyStateProps) {
  return (
    <div style={{
      textAlign: 'center', padding: '48px 24px',
      background: 'var(--surface)', borderRadius: 'var(--r-xl)',
      border: '1px solid var(--hairline)',
    }}>
      <div style={{
        width: 56, height: 56, borderRadius: '50%',
        background: 'var(--brand-soft)', color: 'var(--brand)',
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        fontSize: 24, margin: '0 auto 14px',
      }}>
        {icon}
      </div>
      <h3 style={{ fontSize: 16, fontWeight: 600, marginBottom: 6 }}>{title}</h3>
      <p style={{ color: 'var(--text-2)', fontSize: 13.5, maxWidth: 340, margin: '0 auto 18px', lineHeight: 1.5 }}>
        {description}
      </p>
      {action && (
        <button className="btn btn-primary" onClick={action.onClick}>{action.label}</button>
      )}
    </div>
  )
}

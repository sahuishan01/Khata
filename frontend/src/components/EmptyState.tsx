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
      display: 'flex', flexDirection: 'column', alignItems: 'center', textAlign: 'center',
      gap: 8, padding: '34px 10px',
    }}>
      <div style={{
        width: 42, height: 42, borderRadius: 11,
        background: 'var(--brand-soft)', color: 'var(--brand)',
        display: 'grid', placeItems: 'center', fontSize: 20,
      }}>
        {icon}
      </div>
      <div style={{ fontSize: 13, fontWeight: 600 }}>{title}</div>
      <div style={{ fontSize: 11, color: 'var(--text-2)', maxWidth: '30ch' }}>{description}</div>
      {action && (
        <div style={{ marginTop: 6, background: 'var(--brand)', color: '#fff', fontWeight: 600, fontSize: 11.5, borderRadius: 999, padding: '9px 18px', cursor: 'pointer' }} onClick={action.onClick}>{action.label}</div>
      )}
    </div>
  )
}

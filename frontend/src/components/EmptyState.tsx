interface EmptyStateProps {
  icon: React.ReactNode
  title: string
  description: string
  action?: { label: string; onClick: () => void }
}

/** DESIGN.md §4.3 — icon, one sentence of direction, and the primary action. */
export function EmptyState({ icon, title, description, action }: EmptyStateProps) {
  return (
    <div className="empty-state">
      <div className="empty-state-icon" aria-hidden>{icon}</div>
      <div className="empty-state-title">{title}</div>
      <div className="empty-state-msg">{description}</div>
      {action && (
        <button className="btn btn-primary" style={{ marginTop: 4 }} onClick={action.onClick}>{action.label}</button>
      )}
    </div>
  )
}

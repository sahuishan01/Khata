import { AlertTriangle, RefreshCw } from 'lucide-react'

interface ErrorStateProps {
  title?: string
  message?: string
  onRetry?: () => void
}

export function ErrorState({ title = 'Something went wrong', message, onRetry }: ErrorStateProps) {
  return (
    <div className="error-state">
      <div className="error-state-icon">
        <AlertTriangle size={20} />
      </div>
      <div className="error-state-title">{title}</div>
      {message && <div className="error-state-msg">{message}</div>}
      {onRetry && (
        <button className="btn btn-secondary btn-sm" onClick={onRetry} style={{ marginTop: 8 }}>
          <RefreshCw size={14} />
          Retry
        </button>
      )}
    </div>
  )
}

interface EmptyStateProps {
  icon?: string
  title: string
  message?: string
  action?: { label: string; onClick: () => void }
}

export function EmptyState({ icon = '📄', title, message, action }: EmptyStateProps) {
  return (
    <div className="empty-state">
      <div className="empty-state-icon">{icon}</div>
      <div className="empty-state-title">{title}</div>
      {message && <div className="empty-state-msg">{message}</div>}
      {action && (
        <button className="btn btn-primary btn-sm" onClick={action.onClick} style={{ marginTop: 8 }}>
          {action.label}
        </button>
      )}
    </div>
  )
}

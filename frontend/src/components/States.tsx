import { AlertTriangle, RefreshCw } from 'lucide-react'
import { CopyButton } from './shared/CopyButton'

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
      <div style={{ display: 'flex', gap: 8, marginTop: 8, justifyContent: 'center', flexWrap: 'wrap' }}>
        {onRetry && (
          <button className="btn btn-secondary btn-sm" onClick={onRetry}>
            <RefreshCw size={14} />
            Retry
          </button>
        )}
        {message && <CopyButton text={`${title}: ${message}`} label="Copy error" />}
      </div>
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

interface SkeletonProps {
  lines?: number
  height?: 'sm' | 'md' | 'lg' | 'card'
  className?: string
}

export function Skeleton({ lines = 3, height = 'md', className = '' }: SkeletonProps) {
  const sizeClass = height === 'card' ? 'skeleton-rect' : height === 'lg' ? 'skeleton-line-lg' : height === 'sm' ? 'skeleton-line-sm' : 'skeleton-line'
  return (
    <div className={`skeleton-card ${className}`}>
      {Array.from({ length: lines }).map((_, i) => (
        <div key={i} className={`skeleton ${sizeClass}`} style={{ width: i === lines - 1 ? '60%' : '100%' }} />
      ))}
    </div>
  )
}

export function StatCardSkeleton() {
  return (
    <div className="skeleton-card" style={{ gap: 16 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
        <div className="skeleton skeleton-circle" />
        <div className="skeleton skeleton-line-sm" style={{ width: 80 }} />
      </div>
      <div className="skeleton skeleton-line-lg" style={{ width: '50%' }} />
    </div>
  )
}

export function TableSkeleton({ rows = 5 }: { rows?: number }) {
  return (
    <div className="panel" style={{ padding: 0 }}>
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} style={{ display: 'flex', gap: 12, padding: '12px 16px', borderBottom: i < rows - 1 ? '1px solid var(--hairline)' : 'none' }}>
          <div className="skeleton skeleton-circle" style={{ width: 32, height: 32 }} />
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 8 }}>
            <div className="skeleton skeleton-line" style={{ width: `${60 + Math.random() * 30}%` }} />
            <div className="skeleton skeleton-line-sm" style={{ width: '30%' }} />
          </div>
          <div className="skeleton skeleton-line" style={{ width: 72 }} />
        </div>
      ))}
    </div>
  )
}

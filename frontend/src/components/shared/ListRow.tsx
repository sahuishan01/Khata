import { ReactNode } from 'react'

interface ListRowProps {
  leading?: ReactNode
  children: ReactNode
  trailing?: ReactNode
  onClick?: () => void
}

export function ListRow({ leading, children, trailing, onClick }: ListRowProps) {
  return (
    <div
      onClick={onClick}
      style={{
        display: 'flex', alignItems: 'center', gap: 12,
        padding: '12px 16px', borderBottom: '1px solid var(--hairline)',
        cursor: onClick ? 'pointer' : undefined, transition: 'background 0.1s',
      }}
      onMouseEnter={e => onClick && (e.currentTarget.style.background = 'var(--surface-2)')}
      onMouseLeave={e => onClick && (e.currentTarget.style.background = '')}
    >
      {leading}
      <div style={{ flex: 1, minWidth: 0 }}>{children}</div>
      {trailing}
    </div>
  )
}

export function ListRowText({ primary, secondary }: { primary: string; secondary?: string }) {
  return (
    <>
      <div style={{ fontWeight: 600, fontSize: 13.5, color: 'var(--text)', display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden', wordBreak: 'break-word' }}>
        {primary}
      </div>
      {secondary && <div style={{ fontSize: 12, color: 'var(--text-2)', marginTop: 2 }}>{secondary}</div>}
    </>
  )
}

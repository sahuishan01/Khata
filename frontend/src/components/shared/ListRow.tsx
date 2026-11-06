import type { ReactNode } from 'react'

interface ListRowProps {
  leading?: ReactNode
  children: ReactNode
  trailing?: ReactNode
  onClick?: () => void
}

export function ListRow({ leading, children, trailing, onClick }: ListRowProps) {
  const Tag = onClick ? 'button' : 'div'
  return (
    <Tag
      onClick={onClick}
      className="list-row"
      style={{
        display: 'flex', alignItems: 'center', gap: 12, width: '100%',
        minHeight: 52, padding: '12px 20px',
        background: 'transparent', border: 'none', borderBottom: '1px solid var(--hairline)',
        color: 'inherit', textAlign: 'left', font: 'inherit',
        cursor: onClick ? 'pointer' : undefined,
      }}
    >
      {leading}
      <div style={{ flex: 1, minWidth: 0 }}>{children}</div>
      {trailing}
    </Tag>
  )
}

export function ListRowText({ primary, secondary }: { primary: string; secondary?: string }) {
  return (
    <>
      <div style={{ fontWeight: 550, fontSize: 'var(--fs-body)', color: 'var(--text)', display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden', wordBreak: 'break-word', lineHeight: 1.35 }}>
        {primary}
      </div>
      {secondary && <div className="mono" style={{ fontSize: 'var(--fs-caption)', color: 'var(--text-2)', marginTop: 2 }}>{secondary}</div>}
    </>
  )
}

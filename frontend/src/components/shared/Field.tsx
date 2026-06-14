import { InputHTMLAttributes } from 'react'

export function Field(props: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      {...props}
      className="form-input"
      style={{
        width: '100%', padding: '9px 10px', background: 'var(--surface-2)',
        border: '1px solid var(--hairline)', borderRadius: 9, fontSize: 11,
        color: 'var(--text)', outline: 'none', transition: 'border-color 0.14s',
        ...props.style,
      }}
    />
  )
}

export function Select(props: InputHTMLAttributes<HTMLSelectElement>) {
  return (
    <select
      {...props}
      className="form-input"
      style={{
        width: '100%', padding: '9px 12px', background: 'var(--surface-2)',
        border: '1px solid var(--hairline)', borderRadius: 8, fontSize: 14,
        color: 'var(--text)', outline: 'none', cursor: 'pointer',
        ...props.style,
      }}
    />
  )
}

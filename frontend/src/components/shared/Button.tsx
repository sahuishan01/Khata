import { ButtonHTMLAttributes, ReactNode } from 'react'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost'
  size?: 'sm' | 'md'
  children: ReactNode
}

const variants = {
  primary: { background: 'var(--brand)', color: 'white' },
  secondary: { background: 'var(--surface-2)', color: 'var(--text)', border: '1px solid var(--hairline)' },
  danger: { background: 'transparent', color: 'var(--expense)', border: '1px solid var(--expense)' },
  ghost: { background: 'transparent', color: 'var(--text-2)', border: 'none' },
}

const sizes = {
  sm: { padding: '5px 10px', fontSize: 11 },
  md: { padding: '10px 18px', fontSize: 11.5 },
}

export function Button({ variant = 'primary', size = 'md', children, style, ...props }: ButtonProps) {
  return (
    <button
      {...props}
      style={{
        display: 'inline-flex', alignItems: 'center', justifyContent: 'center', gap: 6,
        borderRadius: 999, fontWeight: 600, cursor: 'pointer', fontSize: 11.5,
        lineHeight: 1, whiteSpace: 'nowrap', textDecoration: 'none',
        transition: 'all 0.15s',
        ...variants[variant], ...sizes[size],
        ...(props.disabled ? { opacity: 0.4, cursor: 'not-allowed' } : {}),
        ...style,
      }}
    >
      {children}
    </button>
  )
}

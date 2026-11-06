import type { ButtonHTMLAttributes, ReactNode } from 'react'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost'
  size?: 'sm' | 'md' | 'lg'
  children: ReactNode
}

/** DESIGN.md §4.4 — styling lives in index.css (.btn-*). */
export function Button({ variant = 'primary', size = 'md', children, className, ...props }: ButtonProps) {
  const sizeClass = size === 'sm' ? ' btn-sm' : size === 'lg' ? ' btn-lg' : ''
  return (
    <button {...props} className={`btn btn-${variant}${sizeClass}${className ? ` ${className}` : ''}`}>
      {children}
    </button>
  )
}

import type { InputHTMLAttributes, SelectHTMLAttributes } from 'react'

export function Field(props: InputHTMLAttributes<HTMLInputElement>) {
  return <input {...props} className={`form-input${props.className ? ` ${props.className}` : ''}`} />
}

export function Select(props: SelectHTMLAttributes<HTMLSelectElement>) {
  return <select {...props} className={`form-input${props.className ? ` ${props.className}` : ''}`} />
}

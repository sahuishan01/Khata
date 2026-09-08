import { useState } from 'react'
import { Copy, Check } from 'lucide-react'

interface CopyButtonProps {
  /** Text placed on the clipboard. */
  text: string
  /** Visible label; omit for an icon-only button. */
  label?: string
  size?: number
  className?: string
  style?: React.CSSProperties
}

/**
 * Small button that copies `text` to the clipboard and flashes a check for 1.5s.
 * Used next to error messages so they can be pasted verbatim into a bug report.
 */
export function CopyButton({ text, label = 'Copy', size = 13, className, style }: CopyButtonProps) {
  const [copied, setCopied] = useState(false)

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text)
    } catch {
      // clipboard API blocked (insecure context / permissions) — fall back
      const ta = document.createElement('textarea')
      ta.value = text
      ta.style.position = 'fixed'
      ta.style.opacity = '0'
      document.body.appendChild(ta)
      ta.select()
      try { document.execCommand('copy') } catch { /* give up silently */ }
      document.body.removeChild(ta)
    }
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  return (
    <button
      type="button"
      onClick={copy}
      className={className}
      title="Copy to clipboard"
      style={{
        display: 'inline-flex', alignItems: 'center', gap: 4,
        background: 'transparent', border: '1px solid var(--hairline)',
        borderRadius: 6, padding: '3px 8px', cursor: 'pointer',
        fontSize: 11, fontWeight: 600,
        color: copied ? 'var(--income)' : 'var(--text-2)',
        ...style,
      }}
    >
      {copied ? <Check size={size} /> : <Copy size={size} />}
      {label && (copied ? 'Copied' : label)}
    </button>
  )
}

export function formatINR(rupees: number, opts: { sign?: boolean } = {}) {
  const abs = Math.abs(rupees)
  const body = abs.toLocaleString('en-IN', {
    minimumFractionDigits: Number.isInteger(abs) ? 0 : 2,
    maximumFractionDigits: 2,
  })
  const s = opts.sign ? (rupees < 0 ? '−' : '+') : (rupees < 0 ? '−' : '')
  return `${s}₹${body}`
}

export function formatDate(iso: string): string {
  if (!iso) return ''
  const d = new Date(iso + (iso.includes('T') ? '' : 'T00:00:00'))
  if (isNaN(d.getTime())) return iso
  return d.toLocaleDateString('en-IN', { day: '2-digit', month: 'short', year: 'numeric' })
}

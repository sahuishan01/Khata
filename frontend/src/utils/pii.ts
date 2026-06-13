export function maskIdentifier(id: string): string {
  if (id.length <= 4) return '••••'
  const last4 = id.slice(-4)
  return `••••${last4}`
}

export function maskDescription(desc: string, blur: boolean): string {
  if (!blur) return desc
  if (desc.length <= 8) return '••••••••'
  return desc.slice(0, 4) + '••••' + desc.slice(-4)
}

export function maskEmail(email: string, blur: boolean): string {
  if (!blur) return email
  const [local, domain] = email.split('@')
  if (!domain) return email
  return `${local.slice(0, 2)}•••@${domain}`
}

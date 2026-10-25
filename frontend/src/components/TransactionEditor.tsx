import { useEffect, useRef, useState, type FormEvent } from 'react'
import { isAxiosError } from 'axios'
import { api } from '../api/client'

export interface EditableTransaction {
  id: string; rev: number; txn_date: string; value_date: string; description: string
  amount: number; direction: string; bank: string; category: string
  is_transfer: boolean; possible_duplicate?: boolean; notes: string
  bank_ref?: string | null; balance?: number | null
}

interface Props {
  transaction: EditableTransaction
  categories: string[]
  onSaved: (transaction: EditableTransaction) => void
  onClose: () => void
}

const money = (value: string, positive: boolean) => /^-?\d+(?:\.\d{1,2})?$/.test(value) &&
  Number.isFinite(Number(value)) && Math.abs(Number(value)) <= 999999999999.99 && (!positive || Number(value) > 0)

export function TransactionEditor({ transaction, categories, onSaved, onClose }: Props) {
  const dialog = useRef<HTMLDialogElement>(null)
  const [draft, setDraft] = useState(() => ({ ...transaction, amount: String(transaction.amount),
    balance: transaction.balance == null ? '' : String(transaction.balance), bank_ref: transaction.bank_ref ?? '' }))
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const valid = draft.description.trim().length > 0 && draft.description.length <= 16000 &&
    money(draft.amount, true) && (!draft.balance || money(draft.balance, false)) &&
    !!draft.txn_date && !!draft.value_date && !!draft.category.trim() && draft.category.length <= 100 &&
    !!draft.bank.trim() && draft.bank.length <= 100 && draft.bank_ref.length <= 500 && draft.notes.length <= 16000

  useEffect(() => { dialog.current?.showModal() }, [])

  async function save(event: FormEvent) {
    event.preventDefault()
    if (!valid || saving) return
    setSaving(true); setError(null)
    try {
      const { data } = await api.put<EditableTransaction>(`/txns/${transaction.id}`, {
        base_rev: draft.rev, description: draft.description, amount: Number(draft.amount), direction: draft.direction,
        txn_date: draft.txn_date, value_date: draft.value_date, category: draft.category.trim(), bank: draft.bank.trim(),
        bank_ref: draft.bank_ref, balance: draft.balance ? Number(draft.balance) : null,
        is_transfer: draft.is_transfer, notes: draft.notes,
      })
      onSaved(data); onClose()
    } catch (e) {
      setError(isAxiosError(e) && e.response?.status === 409
        ? 'This transaction changed. Reopen it to review the latest values before saving.'
        : 'Could not save the transaction. Check your connection and retry.')
    } finally { setSaving(false) }
  }

  return (
    <dialog ref={dialog} aria-labelledby="edit-transaction-heading"
      onCancel={event => { event.preventDefault(); if (!saving) onClose() }}
      style={{ width: 'min(560px, calc(100% - 32px))', maxHeight: 'calc(100dvh - 32px)', overflow: 'hidden', boxSizing: 'border-box', background: 'var(--surface)', color: 'var(--text)', border: '1px solid var(--hairline)', borderRadius: 16, padding: 24 }}>
      <form onSubmit={save} style={{ display: 'flex', flexDirection: 'column', maxHeight: 'calc(100dvh - 80px)' }}>
        <h2 id="edit-transaction-heading" style={{ fontSize: 20, margin: '0 0 16px', flexShrink: 0 }}>Edit transaction</h2>
        <div style={{ minHeight: 0, overflowY: 'auto' }}>
        <fieldset disabled={saving} style={{ border: 0, padding: 0, margin: 0, display: 'grid', gap: 12, minWidth: 0 }}>
          <label className="form-label">Description
            <textarea className="form-input" aria-label="Description" autoFocus rows={4} value={draft.description} maxLength={16000} required
              onChange={e => setDraft({ ...draft, description: e.target.value })} />
          </label>
          <label className="form-label">Amount (INR)
            <input className="form-input" inputMode="decimal" value={draft.amount} required aria-invalid={!money(draft.amount, true)}
              onChange={e => setDraft({ ...draft, amount: e.target.value })} />
          </label>
          <label className="form-label">Type
            <select className="form-input" aria-label="Type" value={draft.direction} onChange={e => setDraft({ ...draft, direction: e.target.value })}>
              <option value="debit">Expense</option><option value="credit">Income</option>
            </select>
          </label>
          <label className="form-label">Transaction date
            <input className="form-input" type="date" required value={draft.txn_date} onChange={e => setDraft({ ...draft, txn_date: e.target.value })} />
          </label>
          <label className="form-label">Value date
            <input className="form-input" type="date" required value={draft.value_date} onChange={e => setDraft({ ...draft, value_date: e.target.value })} />
          </label>
          <label className="form-label">Category
            <input className="form-input" list="edit-categories" value={draft.category} required maxLength={100} onChange={e => setDraft({ ...draft, category: e.target.value })} />
            <datalist id="edit-categories">{categories.map(c => <option key={c} value={c} />)}</datalist>
          </label>
          <label className="form-label">Bank
            <input className="form-input" value={draft.bank} required maxLength={100} onChange={e => setDraft({ ...draft, bank: e.target.value })} />
          </label>
          <label className="form-label">Bank reference
            <input className="form-input" value={draft.bank_ref} maxLength={500} onChange={e => setDraft({ ...draft, bank_ref: e.target.value })} />
          </label>
          <label className="form-label">Balance (optional)
            <input className="form-input" inputMode="decimal" value={draft.balance} aria-invalid={!!draft.balance && !money(draft.balance, false)} onChange={e => setDraft({ ...draft, balance: e.target.value })} />
          </label>
          <label style={{ display: 'flex', alignItems: 'center', gap: 8, minHeight: 44 }}>
            <input type="checkbox" checked={draft.is_transfer} onChange={e => setDraft({ ...draft, is_transfer: e.target.checked })} />Transfer
          </label>
          <label className="form-label">Notes
            <textarea className="form-input" aria-label="Notes" rows={2} maxLength={16000} value={draft.notes} onChange={e => setDraft({ ...draft, notes: e.target.value })} />
          </label>
          <p style={{ fontSize: 12, color: 'var(--text-2)', margin: 0 }}>Use a positive amount with up to two decimal places.</p>
        </fieldset>
        </div>
        {error && <p role="alert" style={{ color: 'var(--expense)', flexShrink: 0 }}>{error}</p>}
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8, marginTop: 16, flexShrink: 0 }}>
          <button type="button" className="btn btn-secondary" style={{ minHeight: 44 }} disabled={saving} onClick={onClose}>Cancel</button>
          <button type="submit" className="btn btn-primary" style={{ minHeight: 44 }} disabled={saving || !valid}>{saving ? 'Saving…' : 'Save'}</button>
        </div>
      </form>
    </dialog>
  )
}

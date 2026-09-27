/** Pure helpers for emailing invoices: per-invoice state, readiness and template editing. */
import type { EmailAccount, EmailCredentialsStatus, EmailProvider, EmailProviderPreset, InvoiceEmail } from '@/types'

export type MailMarkKind = 'none' | 'sent' | 'failed'

/** Where an invoice stands with its patient's inbox. */
export interface InvoiceEmailState {
  kind: MailMarkKind
  /** The latest successful send, if any. */
  lastSent: InvoiceEmail | null
  /** The latest attempt, successful or not. */
  last: InvoiceEmail | null
}

export function groupEmailsByInvoice(records: readonly InvoiceEmail[]): Map<number, InvoiceEmail[]> {
  const groups = new Map<number, InvoiceEmail[]>()
  for (const record of records) {
    const group = groups.get(record.invoice_id)
    if (group) group.push(record)
    else groups.set(record.invoice_id, [record])
  }
  return groups
}

/** Newest first by time, then by id. */
function newestFirst(a: InvoiceEmail, b: InvoiceEmail): number {
  return b.sent_at.localeCompare(a.sent_at) || b.id - a.id
}

/**
 * Once an email has gone, the invoice counts as sent even if a later resend failed:
 * the patient already has it. Only failures with no success behind them read as failed.
 */
export function invoiceEmailState(records: readonly InvoiceEmail[]): InvoiceEmailState {
  const sorted = [...records].sort(newestFirst)
  const last = sorted[0] ?? null
  const lastSent = sorted.find((record) => record.status === 'sent') ?? null
  const kind: MailMarkKind = lastSent ? 'sent' : last ? 'failed' : 'none'
  return { kind, lastSent, last }
}

export function describeMailMark(kind: MailMarkKind): string {
  switch (kind) {
    case 'sent': return 'Inviata al paziente'
    case 'failed': return 'Invio al paziente non riuscito'
    case 'none': return 'Non ancora inviata al paziente'
  }
}

/** A saved mailbox with its password: nothing on this side stops a send. */
export function isEmailReady(account: EmailAccount | null, credentials: EmailCredentialsStatus | null): boolean {
  return account?.saved === true && credentials?.password_configured === true
}

/** The preset whose domains include the address's, or the custom one. */
export function detectProvider(address: string, presets: readonly EmailProviderPreset[]): EmailProvider {
  const domain = address.trim().toLowerCase().split('@')[1] ?? ''
  return presets.find((preset) => preset.domains.includes(domain))?.provider ?? 'custom'
}

/** Puts `{key}` in place of the selection and says where the caret goes next. */
export function insertPlaceholder(text: string, start: number, end: number, key: string): { text: string; caret: number } {
  const token = `{${key}}`
  return { text: text.slice(0, start) + token + text.slice(end), caret: start + token.length }
}

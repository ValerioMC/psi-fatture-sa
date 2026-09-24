/**
 * Display labels and tones for every enumerated domain value, in one place so
 * a status reads the same in a badge, a filter, a dialog and the printed PDF.
 */
import type { AppointmentStatus, InvoiceStatus, PaymentMethod, TaxRegime } from '@/types'

/** A semantic colour role. Maps 1:1 to the -soft / -line token pairs in style.css. */
export type Tone = 'neutral' | 'accent' | 'safe' | 'warn' | 'danger'

export interface StatusPresentation {
  label: string
  tone: Tone
}

export const INVOICE_STATUS: Readonly<Record<InvoiceStatus, StatusPresentation>> = {
  draft: { label: 'Bozza', tone: 'neutral' },
  issued: { label: 'Emessa', tone: 'warn' },
  paid: { label: 'Pagata', tone: 'safe' },
  overdue: { label: 'Scaduta', tone: 'danger' },
  cancelled: { label: 'Annullata', tone: 'neutral' },
}

/** Order in which invoice statuses are offered to the user: the life of an invoice. */
export const INVOICE_STATUS_ORDER: readonly InvoiceStatus[] = ['draft', 'issued', 'paid', 'overdue', 'cancelled']

export const APPOINTMENT_STATUS: Readonly<Record<AppointmentStatus, StatusPresentation>> = {
  scheduled: { label: 'In programma', tone: 'accent' },
  completed: { label: 'Svolto', tone: 'safe' },
  cancelled: { label: 'Annullato', tone: 'neutral' },
}

export const APPOINTMENT_STATUS_ORDER: readonly AppointmentStatus[] = ['scheduled', 'completed', 'cancelled']

export const PAYMENT_METHOD_LABEL: Readonly<Record<PaymentMethod, string>> = {
  bonifico: 'Bonifico bancario',
  contanti: 'Contanti',
  pos: 'POS / Carta',
  altro: 'Altro',
}

export const PAYMENT_METHOD_ORDER: readonly PaymentMethod[] = ['bonifico', 'pos', 'contanti', 'altro']

export const TAX_REGIME_LABEL: Readonly<Record<TaxRegime, string>> = {
  forfettario: 'Regime forfettario',
  ordinario: 'Regime ordinario',
}

export function isInvoiceStatus(value: string): value is InvoiceStatus {
  return value in INVOICE_STATUS
}

/**
 * Italian noun agreement for a count: `plural(3, 'fattura', 'fatture')` → "3 fatture".
 */
export function plural(count: number, singular: string, pluralForm: string): string {
  return `${count} ${count === 1 ? singular : pluralForm}`
}

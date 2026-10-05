/**
 * The invoice form's data: its shape, the checks run on save and the payloads it
 * sends. Each line carries a `key` so Vue keeps its inputs while lines come and go.
 */
import type { SegmentOption } from '@/components/ui/types'
import type { CreateInvoiceInput, Invoice, InvoiceLineInput, InvoiceStatus, PaymentMethod, UpdateInvoiceInput } from '@/types'
import { INVOICE_STATUS, INVOICE_STATUS_ORDER } from '@/utils/labels'
import { enpapIncludesBollo, hasAmountOverride, type TaxProfile } from '@/utils/tax'

export interface FormLine extends InvoiceLineInput {
  key: number
}

export interface InvoiceForm {
  client_id: number | null
  invoice_number: string
  issue_date: string
  due_date: string
  status: InvoiceStatus
  payment_method: PaymentMethod
  notes: string
  apply_enpap: boolean
  hide_quantity: boolean
  paid_date: string
  lines: FormLine[]
}

export type InvoiceFieldKey = 'client' | 'invoice_number' | 'issue_date' | 'due_date' | `line-${number}`
export type InvoiceFormErrors = Partial<Record<InvoiceFieldKey, string>>

export const INVOICE_STATUS_OPTIONS: SegmentOption<InvoiceStatus>[] = INVOICE_STATUS_ORDER.map((value) => ({
  value,
  label: INVOICE_STATUS[value].label,
  tone: INVOICE_STATUS[value].tone,
}))

export function blankLine(key: number): FormLine {
  return { key, service_id: undefined, description: '', quantity: 1, unit_price: 0, vat_rate: 0, amount_override: null }
}

export function blankInvoiceForm(firstLine: FormLine, issueDate: string): InvoiceForm {
  return {
    client_id: null,
    invoice_number: '',
    issue_date: issueDate,
    due_date: '',
    status: 'draft',
    payment_method: 'bonifico',
    notes: '',
    apply_enpap: true,
    hide_quantity: false,
    paid_date: '',
    lines: [firstLine],
  }
}

/** A saved invoice as the form edits it; optional dates become empty fields. */
export function formFromInvoice(invoice: Invoice, nextKey: () => number): InvoiceForm {
  return {
    client_id: invoice.client_id,
    invoice_number: invoice.invoice_number,
    issue_date: invoice.issue_date,
    due_date: invoice.due_date ?? '',
    status: invoice.status,
    payment_method: invoice.payment_method,
    notes: invoice.notes,
    apply_enpap: invoice.apply_enpap,
    hide_quantity: invoice.hide_quantity,
    paid_date: invoice.paid_date ?? '',
    lines: invoice.lines.map((line) => ({
      key: nextKey(),
      service_id: line.service_id,
      description: line.description,
      quantity: line.quantity,
      unit_price: line.unit_price,
      vat_rate: line.vat_rate,
      amount_override: line.amount_override ?? null,
    })),
  }
}

/** One message per field to fix. The number is checked only when editing: a new invoice gets the next one. */
export function invoiceFormErrors(form: InvoiceForm, isEdit: boolean): InvoiceFormErrors {
  const errors: InvoiceFormErrors = {}
  if (form.client_id === null) errors.client = 'Scegli il paziente a cui intestare la fattura.'
  if (isEdit && form.invoice_number.trim() !== '') {
    const number = Number(form.invoice_number.trim())
    if (!Number.isInteger(number) || number < 1) errors.invoice_number = 'Un numero intero maggiore di zero.'
  }
  if (!form.issue_date) errors.issue_date = 'Indica la data di emissione.'
  if (form.due_date && form.due_date < form.issue_date) errors.due_date = 'La scadenza non può precedere l’emissione.'
  form.lines.forEach((line, index) => {
    const message = lineError(line)
    if (message !== undefined) errors[`line-${index}`] = message
  })
  return errors
}

/** The first problem of a line, checked in the order the fields appear. */
export function lineError(line: InvoiceLineInput): string | undefined {
  if (!line.description.trim()) return 'Serve una descrizione.'
  if (!Number.isInteger(line.quantity) || line.quantity < 1) return 'La quantità è un intero da 1 in su.'
  if (!Number.isFinite(line.unit_price) || line.unit_price < 0) return 'Prezzo non valido.'
  if (!Number.isFinite(line.vat_rate) || line.vat_rate < 0 || line.vat_rate > 100) return 'IVA tra 0 e 100.'
  if (hasAmountOverride(line) && (!Number.isFinite(line.amount_override) || line.amount_override < 0)) return 'Importo non valido.'
  return undefined
}

export function createInvoiceInput(form: InvoiceForm, clientId: number): CreateInvoiceInput {
  const lines: InvoiceLineInput[] = form.lines.map(({ key: _key, ...line }) => ({ ...line, service_id: line.service_id || undefined }))
  return {
    client_id: clientId,
    issue_date: form.issue_date,
    due_date: form.due_date || undefined,
    status: form.status,
    payment_method: form.payment_method,
    notes: form.notes,
    apply_enpap: form.apply_enpap,
    hide_quantity: form.hide_quantity,
    lines,
  }
}

/** An emptied number keeps the saved one; a paid date is sent only for a paid invoice. */
export function updateInvoiceInput(form: InvoiceForm, clientId: number, id: number): UpdateInvoiceInput {
  return {
    ...createInvoiceInput(form, clientId),
    id,
    invoice_number: form.invoice_number.trim() || undefined,
    paid_date: form.status === 'paid' ? form.paid_date || undefined : undefined,
  }
}

export function enpapDescription(profile: TaxProfile): string {
  return enpapIncludesBollo(profile)
    ? 'Addebitato al paziente su imponibile e marca da bollo, che nel forfettario è compenso.'
    : 'Addebitato al paziente sul solo imponibile, senza la marca da bollo.'
}

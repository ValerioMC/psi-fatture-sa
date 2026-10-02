/**
 * Figures that make the patient register actionable: who is currently in care
 * (billed recently) and whose record still lacks what an invoice or its email needs.
 */
import type { Client, InvoiceStatus } from '@/types'

export type MissingField = 'codice fiscale' | 'indirizzo' | 'email'

export type ClientForCompleteness = Pick<
  Client,
  'client_type' | 'fiscal_code' | 'vat_number' | 'address' | 'city' | 'zip_code' | 'email'
>

export interface InvoiceForCare {
  client_id: number
  status: InvoiceStatus
  issue_date: string
}

const blank = (value: string | undefined): boolean => (value ?? '').trim() === ''

/** Fields an invoice or its email would be missing, in the order the form asks for them. */
export function missingFields(client: ClientForCompleteness): MissingField[] {
  const missing: MissingField[] = []
  const hasTaxId = client.client_type === 'azienda' ? !blank(client.vat_number) || !blank(client.fiscal_code) : !blank(client.fiscal_code)
  if (!hasTaxId) missing.push('codice fiscale')
  if (blank(client.address) || blank(client.city) || blank(client.zip_code)) missing.push('indirizzo')
  if (blank(client.email)) missing.push('email')
  return missing
}

function isoDay(date: Date): string {
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${date.getFullYear()}-${month}-${day}`
}

/** Patients with a non-cancelled invoice issued within the last `days` days, today included. */
export function patientsInCare(invoices: readonly InvoiceForCare[], days: number, today: Date = new Date()): Set<number> {
  const since = new Date(today.getFullYear(), today.getMonth(), today.getDate() - days)
  const from = isoDay(since)
  const to = isoDay(today)
  const ids = new Set<number>()
  for (const invoice of invoices) {
    if (invoice.status === 'cancelled') continue
    const issued = invoice.issue_date.slice(0, 10)
    if (issued > from && issued <= to) ids.add(invoice.client_id)
  }
  return ids
}

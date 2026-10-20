/**
 * What invoices say about patients: one patient's history, newest first with its total,
 * and who across the register is in care because they were billed recently.
 */
import type { Invoice } from '@/features/invoice/model/invoice'

export type InvoiceForCare = Pick<Invoice, 'client_id' | 'status' | 'issue_date'>

export function newestFirst(invoices: readonly Invoice[]): Invoice[] {
  return [...invoices].sort((a, b) => b.issue_date.localeCompare(a.issue_date))
}

/** What the patient has been invoiced, cancelled invoices aside. */
export function historyTotal(invoices: readonly Pick<Invoice, 'status' | 'total_due'>[]): number {
  return invoices.filter((invoice) => invoice.status !== 'cancelled').reduce((sum, invoice) => sum + invoice.total_due, 0)
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

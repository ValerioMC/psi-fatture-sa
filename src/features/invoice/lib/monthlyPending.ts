/**
 * Money not yet in hand, grouped by month of issue: the hatched part of each
 * monthly column (issued or overdue, the same rule as the "Da incassare"
 * total) and the dotted draft part on top of it (invoices not yet issued).
 */
import type { InvoiceStatus } from '@/features/invoice/model/invoice'

export interface InvoiceForPending {
  status: InvoiceStatus
  issue_date: string
  total_due: number
}

export interface MonthPending {
  amount: number
  count: number
}

const PENDING_STATUSES: readonly InvoiceStatus[] = ['issued', 'overdue']
const DRAFT_STATUSES: readonly InvoiceStatus[] = ['draft']

/** Twelve entries, index 0 is January. */
function sumByMonth(invoices: readonly InvoiceForPending[], statuses: readonly InvoiceStatus[]): MonthPending[] {
  const months: MonthPending[] = Array.from({ length: 12 }, () => ({ amount: 0, count: 0 }))
  for (const invoice of invoices) {
    if (!statuses.includes(invoice.status)) continue
    const month = months[Number(invoice.issue_date.slice(5, 7)) - 1]
    if (month === undefined) continue
    month.count += 1
    month.amount = Math.round((month.amount + invoice.total_due) * 100) / 100
  }
  return months
}

export function pendingByMonth(invoices: readonly InvoiceForPending[]): MonthPending[] {
  return sumByMonth(invoices, PENDING_STATUSES)
}

export function draftByMonth(invoices: readonly InvoiceForPending[]): MonthPending[] {
  return sumByMonth(invoices, DRAFT_STATUSES)
}

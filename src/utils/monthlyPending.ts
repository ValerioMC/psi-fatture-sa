/**
 * Money issued but not yet paid, grouped by month of issue: the hatched part
 * of each monthly column. Same rule as the "Da incassare" total, issued or
 * overdue, so the chart and the hero figure always add up.
 */
import type { InvoiceStatus } from '@/types'

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

/** Twelve entries, index 0 is January. */
export function pendingByMonth(invoices: readonly InvoiceForPending[]): MonthPending[] {
  const months: MonthPending[] = Array.from({ length: 12 }, () => ({ amount: 0, count: 0 }))
  for (const invoice of invoices) {
    if (!PENDING_STATUSES.includes(invoice.status)) continue
    const month = months[Number(invoice.issue_date.slice(5, 7)) - 1]
    if (month === undefined) continue
    month.count += 1
    month.amount = Math.round((month.amount + invoice.total_due) * 100) / 100
  }
  return months
}

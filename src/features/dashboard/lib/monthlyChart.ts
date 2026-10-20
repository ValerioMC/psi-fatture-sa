/**
 * The monthly chart's figures: one column per month stacking paid, pending and draft,
 * the average and best month over the months already lived, and the year's split.
 */
import type { MonthlyRevenue } from '@/features/dashboard/model/dashboard'
import type { MonthPending } from '@/features/invoice/lib/monthlyPending'

export interface MonthColumn {
  month: number
  name: string
  paid: number
  paidCount: number
  pending: number
  pendingCount: number
  draft: number
  draftCount: number
  /** Paid, pending and draft together: the height of the column. */
  total: number
  /** Paid and pending only: what is really invoiced, the basis for the average and the best month. */
  issued: number
}

export interface YearSplit {
  paid: number
  pending: number
  draft: number
}

const NOTHING: MonthPending = { amount: 0, count: 0 }

/** `pending` and `drafts` are indexed from January. */
export function monthColumns(
  months: readonly MonthlyRevenue[],
  pending: readonly MonthPending[],
  drafts: readonly MonthPending[],
): MonthColumn[] {
  return months.map((month) => {
    const open = pending[month.month - 1] ?? NOTHING
    const draft = drafts[month.month - 1] ?? NOTHING
    return {
      month: month.month,
      name: month.month_name,
      paid: month.revenue,
      paidCount: month.invoice_count,
      pending: open.amount,
      pendingCount: open.count,
      draft: draft.amount,
      draftCount: draft.count,
      total: month.revenue + open.amount + draft.amount,
      issued: month.revenue + open.amount,
    }
  })
}

/** `currentMonth` is null when the year shown is not the current one: then no month is ahead. */
export function isFutureMonth(month: number, currentMonth: number | null): boolean {
  return currentMonth !== null && month > currentMonth
}

/** A month ahead with nothing booked is an empty slot; with invoices in it, it is a column like any other. */
export function isEmptySlot(column: MonthColumn, currentMonth: number | null): boolean {
  return isFutureMonth(column.month, currentMonth) && column.total === 0
}

function livedColumns(columns: readonly MonthColumn[], currentMonth: number | null): MonthColumn[] {
  return columns.filter((column) => !isFutureMonth(column.month, currentMonth) && column.issued > 0)
}

/** Average invoiced over the months already lived that had any; none until there are two. */
export function issuedAverage(columns: readonly MonthColumn[], currentMonth: number | null): number | null {
  const lived = livedColumns(columns, currentMonth)
  if (lived.length < 2) return null
  return lived.reduce((sum, column) => sum + column.issued, 0) / lived.length
}

/** The lived month with the most invoiced; the earliest wins a tie. */
export function bestMonth(columns: readonly MonthColumn[], currentMonth: number | null): number | null {
  const best = livedColumns(columns, currentMonth).reduce<MonthColumn | null>(
    (top, column) => (top === null || column.issued > top.issued ? column : top),
    null,
  )
  return best?.month ?? null
}

export function yearSplit(columns: readonly MonthColumn[]): YearSplit {
  return columns.reduce<YearSplit>(
    (split, column) => ({ paid: split.paid + column.paid, pending: split.pending + column.pending, draft: split.draft + column.draft }),
    { paid: 0, pending: 0, draft: 0 },
  )
}

/** The column's height as a share of the axis; a non-zero column never collapses below a sliver. */
export function heightShare(column: MonthColumn, axisMax: number): number {
  return column.total > 0 ? Math.max((column.total / axisMax) * 100, 1.5) : 0
}

/** A segment's part of its column, as a CSS percentage. */
export function segmentShare(column: MonthColumn, amount: number): string {
  return column.total > 0 ? `${(amount / column.total) * 100}%` : '0%'
}

/**
 * The invoice seal's state machine, kept free of Vue so it can be tested.
 *
 * An invoice's life is draft → issued → paid, with two exits: overdue (issued
 * and past its due date, or marked so) and cancelled. The seal draws that life
 * as one shape; this module decides which shape and how far the clock has run.
 */
import type { InvoiceStatus } from '@/types'

export type SealKind = 'draft' | 'issued' | 'overdue' | 'paid' | 'cancelled'

export interface SealState {
  kind: SealKind
  /**
   * For an issued invoice with a due date: the share of the payment window
   * already elapsed, 0 at issue and 1 on the due date. Null when there is no
   * window to measure (no due date, or any other state).
   */
  elapsed: number | null
  /** Whole days until the due date; negative once it has passed. Null without a due date. */
  daysToDue: number | null
}

export interface SealInput {
  status: InvoiceStatus
  issue_date: string
  due_date?: string | null
}

const DAY_MS = 86_400_000

/** Parses a `YYYY-MM-DD` (or ISO datetime) string as a local calendar day. */
function toDay(isoDate: string): number {
  const [year, month, day] = isoDate.slice(0, 10).split('-').map(Number)
  return Date.UTC(year, month - 1, day) / DAY_MS
}

function todayAsDay(today: Date): number {
  return Date.UTC(today.getFullYear(), today.getMonth(), today.getDate()) / DAY_MS
}

/**
 * Derives the seal for an invoice on a given day.
 *
 * An issued invoice whose due date has passed reads as overdue even when
 * nobody has changed its status yet: the seal shows the fact, the badge shows
 * the record.
 */
export function deriveSeal(invoice: SealInput, today: Date = new Date()): SealState {
  const due = invoice.due_date ? toDay(invoice.due_date) : null
  const now = todayAsDay(today)
  const daysToDue = due === null ? null : due - now

  if (invoice.status !== 'issued') {
    return { kind: invoice.status, elapsed: null, daysToDue: invoice.status === 'overdue' ? daysToDue : null }
  }
  if (due === null || daysToDue === null) {
    return { kind: 'issued', elapsed: null, daysToDue: null }
  }
  if (daysToDue < 0) {
    return { kind: 'overdue', elapsed: 1, daysToDue }
  }
  const issued = toDay(invoice.issue_date)
  const window = due - issued
  const elapsed = window <= 0 ? 1 : Math.min(1, Math.max(0, (now - issued) / window))
  return { kind: 'issued', elapsed, daysToDue }
}

/** A short human sentence for the seal's tooltip and screen-reader label. */
export function describeSeal(state: SealState): string {
  switch (state.kind) {
    case 'draft':
      return 'Bozza, non ancora emessa'
    case 'paid':
      return 'Pagata'
    case 'cancelled':
      return 'Annullata'
    case 'overdue':
      if (state.daysToDue !== null && state.daysToDue < 0) {
        const days = -state.daysToDue
        return `Scaduta da ${days} ${days === 1 ? 'giorno' : 'giorni'}`
      }
      return 'Scaduta'
    case 'issued':
      if (state.daysToDue === null) return 'Emessa, in attesa di pagamento'
      if (state.daysToDue === 0) return 'Emessa, scade oggi'
      return `Emessa, scade tra ${state.daysToDue} ${state.daysToDue === 1 ? 'giorno' : 'giorni'}`
  }
}

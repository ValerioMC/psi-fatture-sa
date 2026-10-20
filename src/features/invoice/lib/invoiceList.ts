/**
 * The invoice list's filters, figures and confirmation texts. Cancelled invoices
 * count in the list but never in the amounts.
 */
import type { SegmentOption } from '@/components/ui/types'
import type { Invoice, InvoiceStatus } from '@/features/invoice/model/invoice'
import { formatCurrency } from '@/common/lib/format'
import { searchKey } from '@/common/lib/search'
import { INVOICE_STATUS, INVOICE_STATUS_ORDER, isInvoiceStatus, plural } from '@/common/lib/labels'

export type StatusFilter = InvoiceStatus | 'all'

export interface ListSummary {
  count: number
  total: number
  paid: number
  pending: number
  overdue: number
}

export interface StatusSegment {
  value: InvoiceStatus
  label: string
  count: number
  share: number
}

export interface SendProgress {
  done: number
  total: number
}

type InvoiceAmount = Pick<Invoice, 'status' | 'total_due'>

/** The year in the URL; 0 means every year, anything unreadable means this year. */
export function yearFromQuery(raw: unknown, currentYear: number): number {
  const year = Number(raw)
  return Number.isInteger(year) && year >= 0 ? year : currentYear
}

export function statusFromQuery(raw: unknown): StatusFilter {
  const status = String(raw ?? '')
  return isInvoiceStatus(status) ? status : 'all'
}

/** Only what differs from the default view goes into the URL. */
export function listQuery(year: number, status: StatusFilter, currentYear: number): Record<string, string | number> {
  return {
    ...(year !== currentYear ? { year } : {}),
    ...(status !== 'all' ? { status } : {}),
  }
}

export function yearOptions(currentYear: number): SegmentOption<number>[] {
  return [
    { value: currentYear, label: String(currentYear) },
    { value: currentYear - 1, label: String(currentYear - 1) },
    { value: currentYear - 2, label: String(currentYear - 2) },
    { value: 0, label: 'Tutti gli anni' },
  ]
}

export const BULK_STATUS_OPTIONS: SegmentOption<InvoiceStatus>[] = (['issued', 'paid', 'overdue', 'cancelled'] as const).map((value) => ({
  value,
  label: INVOICE_STATUS[value].label,
  tone: INVOICE_STATUS[value].tone,
}))

/** Matches number or patient, ignoring case and accents ("nicolo" finds "Nicolò"). */
export function searchInvoices<T extends Pick<Invoice, 'invoice_number' | 'client_name'>>(invoices: readonly T[], query: string): readonly T[] {
  const needle = searchKey(query.trim())
  if (needle === '') return invoices
  return invoices.filter((invoice) => searchKey(`${invoice.invoice_number} ${invoice.client_name}`).includes(needle))
}

export function invoicesInStatus<T extends Pick<Invoice, 'status'>>(invoices: readonly T[], status: StatusFilter): readonly T[] {
  return status === 'all' ? invoices : invoices.filter((invoice) => invoice.status === status)
}

export function statusFilterOptions(invoices: readonly Pick<Invoice, 'status'>[]): SegmentOption<StatusFilter>[] {
  return [
    { value: 'all', label: 'Tutte', count: invoices.length },
    ...INVOICE_STATUS_ORDER.map((value) => ({
      value,
      label: INVOICE_STATUS[value].label,
      count: invoices.filter((invoice) => invoice.status === value).length,
    })),
  ]
}

export function summarise(invoices: readonly InvoiceAmount[]): ListSummary {
  const live = invoices.filter((invoice) => invoice.status !== 'cancelled')
  return {
    count: invoices.length,
    total: sumDue(live),
    paid: sumDue(live.filter((invoice) => invoice.status === 'paid')),
    pending: sumDue(live.filter((invoice) => invoice.status === 'issued' || invoice.status === 'overdue')),
    overdue: live.filter((invoice) => invoice.status === 'overdue').length,
  }
}

/** Percentage of the total already paid, rounded; 0 when nothing is due. */
export function paidShare(summary: ListSummary): number {
  return summary.total > 0 ? Math.round((summary.paid / summary.total) * 100) : 0
}

/** Amount due per month of issue, January first. */
export function monthlyTotals(invoices: readonly (InvoiceAmount & Pick<Invoice, 'issue_date'>)[]): number[] {
  const totals = Array.from({ length: 12 }, () => 0)
  for (const invoice of invoices) {
    if (invoice.status === 'cancelled') continue
    totals[Number(invoice.issue_date.slice(5, 7)) - 1] += invoice.total_due
  }
  return totals
}

/** The statuses present in the list, in display order, with their share of it in percent. */
export function statusDistribution(invoices: readonly Pick<Invoice, 'status'>[]): StatusSegment[] {
  const total = invoices.length
  if (total === 0) return []
  return INVOICE_STATUS_ORDER.map((value) => ({
    value,
    label: INVOICE_STATUS[value].label,
    count: invoices.filter((invoice) => invoice.status === value).length,
  }))
    .filter((segment) => segment.count > 0)
    .map((segment) => ({ ...segment, share: (segment.count / total) * 100 }))
}

/** What a bulk status change is about to do, said before it happens. */
export function describeBulkStatus(invoices: readonly InvoiceAmount[], target: InvoiceStatus): string {
  const count = invoices.length
  const label = INVOICE_STATUS[target].label.toLocaleLowerCase('it-IT')
  const paidNote = target === 'paid' ? ', con data di pagamento oggi' : ''
  return `${plural(count, 'fattura', 'fatture')} per ${formatCurrency(sumDue(invoices))} ${count === 1 ? 'passerà' : 'passeranno'} a “${label}”${paidNote}.`
}

/** Drafts and cancelled invoices are never emailed. */
export function isEmailable(invoice: Pick<Invoice, 'status'>): boolean {
  return invoice.status !== 'draft' && invoice.status !== 'cancelled'
}

/** What a bulk email is about to do: how many leave, how many go again, how many stay out. */
export function describeBulkEmail(sending: number, alreadySent: number, skipped: number): string {
  const parts = [`${plural(sending, 'fattura partirà', 'fatture partiranno')} via email ai pazienti, con il PDF allegato e il modello salvato.`]
  if (alreadySent > 0) parts.push(`${plural(alreadySent, 'era già stata inviata', 'erano già state inviate')} e ${alreadySent === 1 ? 'arriverà' : 'arriveranno'} di nuovo.`)
  if (skipped > 0) parts.push(`${plural(skipped, 'bozza o annullata resta', 'bozze o annullate restano')} fuori.`)
  return parts.join(' ')
}

export function describeSendProgress(progress: SendProgress | null): string {
  return progress
    ? `Invio ${Math.min(progress.done + 1, progress.total)} di ${progress.total}…`
    : 'Chi non ha un indirizzo email nella scheda viene saltato.'
}

/** The toast after a bulk email: a count when all went, the first failure otherwise. */
export function describeBulkEmailOutcome(sent: number, failures: readonly string[]): string {
  if (failures.length === 0) return `${plural(sent, 'fattura inviata', 'fatture inviate')} ai pazienti`
  return `${plural(sent, 'inviata', 'inviate')}, ${plural(failures.length, 'non riuscita', 'non riuscite')}. ${failures[0]}`
}

function sumDue(invoices: readonly InvoiceAmount[]): number {
  return invoices.reduce((total, invoice) => total + invoice.total_due, 0)
}


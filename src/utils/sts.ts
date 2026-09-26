/**
 * The Sistema Tessera Sanitaria as the user sees it: one state per invoice
 * derived from its transmissions, the actions that state allows, and the
 * deadlines of the payment year (Decreto 29/10/2025).
 */
import type {
  InvoiceStatus,
  TsDispatchSummary,
  TsEnvironment,
  TsOperation,
  TsSubmission,
  TsSubmissionStatus,
} from '@/types'
import { formatDateLong } from './format'
import { plural, type StatusPresentation } from './labels'

export const TS_SUBMISSION_STATUS: Readonly<Record<TsSubmissionStatus, StatusPresentation>> = {
  non_inviata: { label: 'In coda', tone: 'neutral' },
  inviata: { label: 'In invio', tone: 'accent' },
  accettata: { label: 'Accettata', tone: 'safe' },
  scartata: { label: 'Scartata', tone: 'danger' },
  annullata: { label: 'Annullata', tone: 'neutral' },
  sostituita: { label: 'Sostituita', tone: 'neutral' },
}

export const TS_OPERATION_LABEL: Readonly<Record<TsOperation, string>> = {
  invio: 'Invio',
  sostituzione: 'Sostituzione',
  annullamento: 'Annullamento',
}

export const TS_ENVIRONMENT_LABEL: Readonly<Record<TsEnvironment, string>> = {
  test: 'Ambiente di test',
  produzione: 'Produzione',
}

/**
 * What the Sistema TS holds for one invoice.
 *
 *   none       never transmitted (or only in another environment)
 *   queued     waiting in the local queue, maybe retrying
 *   sending    a call is in flight right now
 *   accepted   the expense is on the Sistema TS
 *   rejected   the last attempt was refused and nothing is on the Sistema TS
 *   cancelled  it was there and has been cancelled
 */
export type TsMarkKind = 'none' | 'queued' | 'sending' | 'accepted' | 'rejected' | 'cancelled'

export interface InvoiceStsState {
  kind: TsMarkKind
  /** The transmission that defines the state, if any. */
  current: TsSubmission | null
  /** The accepted transmission a replacement or cancellation must target. */
  live: TsSubmission | null
  /** A follow-up that failed while the data stayed on the Sistema TS. */
  failedFollowUp: TsSubmission | null
}

const IN_FLIGHT: readonly TsSubmissionStatus[] = ['non_inviata', 'inviata']

/** Folds an invoice's transmissions, newest first by id, into one state. */
export function invoiceStsState(submissions: readonly TsSubmission[], environment: TsEnvironment): InvoiceStsState {
  const mine = submissions.filter((s) => s.environment === environment).sort((a, b) => b.id - a.id)
  const inFlight = mine.find((s) => IN_FLIGHT.includes(s.status)) ?? null
  const live = mine.find((s) => s.status === 'accettata' && s.operation !== 'annullamento') ?? null
  const latest = mine[0] ?? null
  const failedFollowUp = live !== null && latest !== null && latest.status === 'scartata' && latest.id > live.id ? latest : null

  if (inFlight !== null) {
    return { kind: inFlight.status === 'inviata' ? 'sending' : 'queued', current: inFlight, live, failedFollowUp: null }
  }
  if (live !== null) return { kind: 'accepted', current: live, live, failedFollowUp }
  if (latest === null) return { kind: 'none', current: null, live: null, failedFollowUp: null }
  if (latest.operation === 'annullamento' && latest.status === 'accettata') {
    return { kind: 'cancelled', current: latest, live: null, failedFollowUp: null }
  }
  if (latest.status === 'scartata') return { kind: 'rejected', current: latest, live: null, failedFollowUp: null }
  return { kind: 'cancelled', current: latest, live: null, failedFollowUp: null }
}

/** Groups transmissions by invoice, for list views. */
export function groupByInvoice(submissions: readonly TsSubmission[]): Map<number, TsSubmission[]> {
  const groups = new Map<number, TsSubmission[]>()
  for (const submission of submissions) {
    const group = groups.get(submission.invoice_id)
    if (group) group.push(submission)
    else groups.set(submission.invoice_id, [submission])
  }
  return groups
}

export interface StsActions {
  send: boolean
  replace: boolean
  cancel: boolean
  withdraw: boolean
}

/** What the user may do next. Only paid invoices carry a health expense. */
export function stsActions(state: InvoiceStsState, invoiceStatus: InvoiceStatus): StsActions {
  const paid = invoiceStatus === 'paid'
  return {
    send: paid && (state.kind === 'none' || state.kind === 'rejected' || state.kind === 'cancelled'),
    replace: paid && state.kind === 'accepted',
    cancel: state.kind === 'accepted',
    withdraw: state.kind === 'queued' && state.current?.status === 'non_inviata',
  }
}

const TS_MARK_DESCRIPTION: Readonly<Record<TsMarkKind, string>> = {
  none: 'Non trasmessa al Sistema TS',
  queued: 'In coda per il Sistema TS',
  sending: 'Trasmissione in corso',
  accepted: 'Registrata dal Sistema TS',
  rejected: 'Scartata dal Sistema TS',
  cancelled: 'Annullata sul Sistema TS',
}

export function describeTsMark(kind: TsMarkKind): string {
  return TS_MARK_DESCRIPTION[kind]
}

// ─── Deadlines ──────────────────────────────────────────────────────────────

/** Sistema TS dates are calendar days: build them at local noon to dodge DST edges. */
function localDate(year: number, monthIndex: number, day: number): Date {
  return new Date(year, monthIndex, day, 12)
}

function nextWorkingDay(date: Date): Date {
  const shifted = new Date(date)
  while (shifted.getDay() === 0 || shifted.getDay() === 6) shifted.setDate(shifted.getDate() + 1)
  return shifted
}

function toIso(date: Date): string {
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${date.getFullYear()}-${month}-${day}`
}

export interface StsDeadlines {
  /** Last day to transmit expenses paid in `year`. */
  transmission: string
  /** Last day to correct them without sanctions: five days after the deadline. */
  correction: string
}

/**
 * From 2025 expenses on, transmission is due by 31 January of the next year
 * and corrections are free for five more days; a weekend moves each to Monday.
 */
export function stsDeadlines(paymentYear: number): StsDeadlines {
  const transmission = nextWorkingDay(localDate(paymentYear + 1, 0, 31))
  const correctionDay = new Date(transmission)
  correctionDay.setDate(correctionDay.getDate() + 5)
  return { transmission: toIso(transmission), correction: toIso(nextWorkingDay(correctionDay)) }
}

/**
 *   open        before the transmission deadline
 *   correction  past it, still inside the free correction window
 *   late        past both: sending or correcting now risks a sanction
 */
export type StsDeadlinePhase = 'open' | 'correction' | 'late'

export function stsDeadlinePhase(paymentDate: string, todayIso: string): StsDeadlinePhase {
  const deadlines = stsDeadlines(Number(paymentDate.slice(0, 4)))
  if (todayIso <= deadlines.transmission) return 'open'
  if (todayIso <= deadlines.correction) return 'correction'
  return 'late'
}

export interface SummaryLine {
  message: string
  tone: 'success' | 'error' | 'info'
}

/** One sentence for what a pass over the queue did, or why it could not. */
export function describeDispatch(summary: TsDispatchSummary): SummaryLine {
  const parts: string[] = []
  if (summary.accepted > 0) parts.push(plural(summary.accepted, 'accettata', 'accettate'))
  if (summary.rejected > 0) parts.push(plural(summary.rejected, 'scartata', 'scartate'))
  if (summary.retrying > 0) parts.push(`${summary.retrying} da ritentare`)

  if (summary.blocked !== null) {
    return { message: parts.length > 0 ? `${parts.join(', ')}. ${summary.blocked}` : summary.blocked, tone: 'error' }
  }
  if (parts.length === 0) {
    return {
      message: summary.waiting_other_environment > 0
        ? `Niente da inviare in questo ambiente (${summary.waiting_other_environment} in coda nell'altro)`
        : 'Niente da inviare: la coda è vuota',
      tone: 'info',
    }
  }
  return { message: `Sistema TS: ${parts.join(', ')}`, tone: summary.rejected > 0 || summary.retrying > 0 ? 'error' : 'success' }
}

/** Backend timestamps are UTC `YYYY-MM-DD HH:MM:SS`; shows them in local time. */
export function formatStsTimestamp(timestamp: string): string {
  const moment = new Date(`${timestamp.replace(' ', 'T')}Z`)
  if (Number.isNaN(moment.getTime())) return timestamp
  return moment.toLocaleString('it-IT', { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' })
}

/** How a report or point-query row names the last operation on a document. */
export function describeSendKind(kind: string | null): string {
  switch (kind) {
    case 'I': return 'Inserito'
    case 'V': return 'Variato'
    case 'R': return 'Rimborso'
    case 'C': return 'Cancellato'
    default: return kind ?? '—'
  }
}

/**
 * A date after an Italian preposition, with the article it takes: "entro il
 * 2 febbraio", "fino all’8 febbraio", "entro il 1° febbraio".
 */
export function datePhrase(preposition: 'entro' | 'fino' | 'dal', iso: string): string {
  const day = Number(iso.slice(8, 10))
  const [, month = '', year = ''] = formatDateLong(iso).split(' ')
  const elided = day === 8 || day === 11
  const article = {
    entro: elided ? 'entro l’' : 'entro il ',
    fino: elided ? 'fino all’' : 'fino al ',
    dal: elided ? 'dall’' : 'dal ',
  }[preposition]
  return `${article}${day === 1 ? '1°' : day} ${month} ${year}`
}

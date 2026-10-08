/**
 * The Sistema Tessera Sanitaria as the user sees it: one state per invoice
 * derived from its transmissions, the actions that state allows, and the
 * deadlines of the payment year (Decreto 29/10/2025).
 */
import type {
  Invoice,
  InvoiceStatus,
  TsDispatchProgress,
  TsDispatchSummary,
  TsEnvironment,
  TsOperation,
  TsSubmission,
  TsSubmissionStatus,
} from '@/types'
import { formatDateLong, formatUtcTimestamp } from './format'
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

/**
 * The accepted sends the user may cancel: one per invoice, and only while nothing
 * else is queued for it, since the backend refuses a second follow-up in flight.
 */
export function cancellableSubmissionIds(groups: ReadonlyMap<number, readonly TsSubmission[]>, environment: TsEnvironment): Set<number> {
  const ids = new Set<number>()
  for (const submissions of groups.values()) {
    const state = invoiceStsState(submissions, environment)
    if (canCancel(state) && state.live !== null) ids.add(state.live.id)
  }
  return ids
}

/**
 * Where a transmission belongs in the history filters.
 *
 *   queue      waiting or in flight
 *   accepted   its data is on the Sistema TS now
 *   cancelled  a document removed from the Sistema TS, or the accepted request that removed it
 *   rejected   refused
 *   retired    replaced by a newer send: listed under "all" only
 */
export type TsHistoryGroup = 'queue' | 'accepted' | 'cancelled' | 'rejected' | 'retired'

export function historyGroup(submission: Pick<TsSubmission, 'operation' | 'status'>): TsHistoryGroup {
  switch (submission.status) {
    case 'non_inviata':
    case 'inviata':
      return 'queue'
    case 'accettata':
      return submission.operation === 'annullamento' ? 'cancelled' : 'accepted'
    case 'annullata':
      return 'cancelled'
    case 'scartata':
      return 'rejected'
    case 'sostituita':
      return 'retired'
  }
}

/** An accepted cancellation reads as what it did, not as one more "Accettata". */
export function historyBadge(submission: Pick<TsSubmission, 'operation' | 'status'>): StatusPresentation {
  if (submission.operation === 'annullamento' && submission.status === 'accettata') {
    return { label: 'Cancellata dal TS', tone: 'neutral' }
  }
  return TS_SUBMISSION_STATUS[submission.status]
}

/** Share of the pass already answered, 0–100. A pass that stopped early stays where it stopped. */
export function dispatchPercent(progress: Pick<TsDispatchProgress, 'total' | 'processed'>): number {
  if (progress.total <= 0) return 0
  return Math.min(100, Math.round((progress.processed / progress.total) * 100))
}

/** How a transmission of a pass ended; each name is also its counter in `TsDispatchSummary`. */
export type DispatchOutcome = 'accepted' | 'rejected' | 'retrying'

export const DISPATCH_OUTCOMES: readonly DispatchOutcome[] = ['accepted', 'rejected', 'retrying']

/** "3 accettate", "1 scartata", "2 da ritentare": how a count of one outcome is said. */
export function outcomeLabel(outcome: DispatchOutcome, count: number): string {
  switch (outcome) {
    case 'accepted': return plural(count, 'accettata', 'accettate')
    case 'rejected': return plural(count, 'scartata', 'scartate')
    case 'retrying': return `${count} da ritentare`
  }
}

export interface DispatchSegment {
  outcome: DispatchOutcome
  /** Width on the bar, 0–100. */
  share: number
}

/** The bar's filled part split by outcome, in reading order, so it shows how the pass is going as well as how far. */
export function dispatchSegments(progress: TsDispatchProgress): DispatchSegment[] {
  if (progress.total <= 0) return []
  return DISPATCH_OUTCOMES.map((outcome) => ({ outcome, share: Math.min(100, (progress.summary[outcome] / progress.total) * 100) }))
}

/** The outcome one report added over the previous one, or null when nothing was counted (a skipped call). */
export function answeredOutcome(previous: TsDispatchSummary, next: TsDispatchSummary): DispatchOutcome | null {
  return DISPATCH_OUTCOMES.find((outcome) => next[outcome] > previous[outcome]) ?? null
}

/** Outcomes already counted when a view first sees a pass, grouped since their order is unknown. */
export function seedOutcomes(summary: TsDispatchSummary): DispatchOutcome[] {
  return DISPATCH_OUTCOMES.flatMap((outcome) => Array<DispatchOutcome>(summary[outcome]).fill(outcome))
}

/**
 *   sending      calls still going out
 *   done         every submission got its answer
 *   attention    finished, but some were refused or wait for a retry
 *   interrupted  stopped early: the rest would fail the same way
 */
export type DispatchPhase = 'sending' | 'done' | 'attention' | 'interrupted'

export function dispatchPhase(progress: TsDispatchProgress): DispatchPhase {
  if (!progress.finished) return 'sending'
  if (progress.summary.blocked !== null) return 'interrupted'
  return progress.summary.rejected > 0 || progress.summary.retrying > 0 ? 'attention' : 'done'
}

export interface DispatchHeadline {
  title: string
  sentence: string
}

/** What the progress card says about a pass: a title for the phase and one sentence of where it stands. */
export function dispatchHeadline(progress: TsDispatchProgress): DispatchHeadline {
  const { total, processed, summary } = progress
  const sentSoFar = `${processed} di ${total} ${total === 1 ? 'trasmissione inviata' : 'trasmissioni inviate'}`
  switch (dispatchPhase(progress)) {
    case 'sending':
      return { title: 'Invio al Sistema TS', sentence: `${sentSoFar}.` }
    case 'done':
      return {
        title: 'Invio completato',
        sentence: summary.accepted === total
          ? total === 1 ? 'La trasmissione è stata accettata.' : `Tutte le ${total} trasmissioni sono state accettate.`
          : `${sentSoFar}.`,
      }
    case 'attention':
      return { title: 'Invio completato, alcune da rivedere', sentence: `${sentSoFar}: le scartate sono nella scheda Trasmissioni.` }
    case 'interrupted':
      return { title: 'Invio interrotto', sentence: `${summary.blocked ?? ''} ${sentSoFar}, le altre restano in coda.`.trim() }
  }
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

/** Only accepted data can be cancelled, and not while another transmission for the invoice is queued. */
export function canCancel(state: InvoiceStsState): boolean {
  return state.kind === 'accepted'
}

/** What the user may do next. Only paid invoices carry a health expense. */
export function stsActions(state: InvoiceStsState, invoiceStatus: InvoiceStatus): StsActions {
  const paid = invoiceStatus === 'paid'
  return {
    send: paid && (state.kind === 'none' || state.kind === 'rejected' || state.kind === 'cancelled'),
    replace: paid && state.kind === 'accepted',
    cancel: canCancel(state),
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
  const parts = DISPATCH_OUTCOMES.filter((outcome) => summary[outcome] > 0).map((outcome) => outcomeLabel(outcome, summary[outcome]))

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
  return formatUtcTimestamp(timestamp)
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

/** The sentence under an invoice's Sistema TS mark: where it stands and what happens next. */
export function stsStateSentence(state: InvoiceStsState, invoice: Pick<Invoice, 'status'>): string {
  const current = state.current
  switch (state.kind) {
    case 'none':
      return invoice.status === 'paid'
        ? 'Spesa sanitaria da trasmettere.'
        : 'Si trasmette quando la fattura risulta pagata.'
    case 'queued':
      return current?.last_error
        ? `Nuovo tentativo: ${formatStsTimestamp(current.next_attempt_at)}.`
        : `${TS_OPERATION_LABEL[current?.operation ?? 'invio']} in attesa di invio.`
    case 'sending':
      return 'Invio in corso…'
    case 'accepted':
      return current?.sent_at ? `Registrata il ${formatStsTimestamp(current.sent_at)}.` : 'Registrata.'
    case 'rejected':
      return 'Il Sistema TS non l’ha accettata: correggi i dati e ritrasmetti.'
    case 'cancelled':
      return 'Il documento è stato cancellato dal Sistema TS.'
  }
  return ''
}

export interface DeadlineNotice {
  tone: 'accent' | 'neutral' | 'warn' | 'danger'
  text: string
}

/**
 * The deadline that matters for a paid invoice, and how close it is. Nothing for an unpaid
 * invoice, nor for a late one already registered: there is no deadline left to act on.
 */
export function invoiceDeadlineNotice(
  invoice: Pick<Invoice, 'status' | 'paid_date'>,
  registered: boolean,
  today: string,
): DeadlineNotice | null {
  const paid = invoice.paid_date
  if (invoice.status !== 'paid' || !paid) return null
  const dates = stsDeadlines(Number(paid.slice(0, 4)))
  const phase = stsDeadlinePhase(paid, today)
  if (phase === 'open') {
    return registered
      ? { tone: 'neutral', text: `Correzioni senza sanzioni ${datePhrase('fino', dates.correction)}` }
      : { tone: 'accent', text: `Da inviare ${datePhrase('entro', dates.transmission)}` }
  }
  if (phase === 'correction') {
    return { tone: 'warn', text: `Ultimi giorni senza sanzioni: ${datePhrase('fino', dates.correction)}` }
  }
  return registered
    ? null
    : { tone: 'danger', text: `Termine scaduto ${datePhrase('dal', dates.transmission)}: sarebbe un invio tardivo` }
}

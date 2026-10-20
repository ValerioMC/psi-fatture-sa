import { describe, expect, it } from 'vitest'
import type { TsDispatchProgress, TsSubmission } from '@/features/sts/model/ts'
import {
  cancellableSubmissionIds,
  datePhrase,
  dispatchPercent,
  dispatchVerdict,
  finishedDispatchReport,
  describeDispatch,
  describeSendKind,
  formatStsTimestamp,
  groupByInvoice,
  historyBadge,
  historyGroup,
  invoiceDeadlineNotice,
  invoiceStsState,
  outcomeLabel,
  settleProgress,
  stsActions,
  stsDeadlinePhase,
  stsDeadlines,
  stsStateSentence,
} from './sts'

function submission(overrides: Partial<TsSubmission>): TsSubmission {
  return {
    id: 1,
    invoice_id: 10,
    invoice_number: '12',
    invoice_year: 2026,
    client_name: 'Anna Bianchi',
    operation: 'invio',
    status: 'non_inviata',
    target_submission_id: null,
    environment: 'produzione',
    document: null,
    protocol: null,
    outcome_code: null,
    outcome_message: null,
    attempt_count: 0,
    last_error: null,
    next_attempt_at: '2026-09-26 10:00:00',
    last_attempt_at: null,
    sent_at: null,
    resolved_at: null,
    created_at: '2026-09-26 10:00:00',
    updated_at: '2026-09-26 10:00:00',
    ...overrides,
  }
}

describe('invoiceStsState', () => {
  it('reads an invoice with no transmissions as never sent', () => {
    expect(invoiceStsState([], 'produzione').kind).toBe('none')
  })

  it('tells queued from in-flight', () => {
    expect(invoiceStsState([submission({ status: 'non_inviata' })], 'produzione').kind).toBe('queued')
    expect(invoiceStsState([submission({ status: 'inviata' })], 'produzione').kind).toBe('sending')
  })

  it('keeps the accepted transmission as the live one while a follow-up is queued', () => {
    const original = submission({ id: 1, status: 'accettata' })
    const cancellation = submission({ id: 2, operation: 'annullamento', status: 'non_inviata', target_submission_id: 1 })
    const state = invoiceStsState([original, cancellation], 'produzione')
    expect(state.kind).toBe('queued')
    expect(state.live?.id).toBe(1)
  })

  it('follows a replacement: the newest accepted send is live', () => {
    const state = invoiceStsState(
      [
        submission({ id: 1, status: 'sostituita' }),
        submission({ id: 2, operation: 'sostituzione', status: 'accettata', target_submission_id: 1 }),
      ],
      'produzione',
    )
    expect(state.kind).toBe('accepted')
    expect(state.live?.id).toBe(2)
  })

  it('reads an accepted cancellation as cancelled', () => {
    const state = invoiceStsState(
      [
        submission({ id: 1, status: 'annullata' }),
        submission({ id: 2, operation: 'annullamento', status: 'accettata', target_submission_id: 1 }),
      ],
      'produzione',
    )
    expect(state.kind).toBe('cancelled')
    expect(state.live).toBeNull()
  })

  it('keeps the data accepted when a later replacement was rejected, and says so', () => {
    const state = invoiceStsState(
      [
        submission({ id: 1, status: 'accettata' }),
        submission({ id: 2, operation: 'sostituzione', status: 'scartata', target_submission_id: 1 }),
      ],
      'produzione',
    )
    expect(state.kind).toBe('accepted')
    expect(state.failedFollowUp?.id).toBe(2)
  })

  it('reads a rejected first send as rejected', () => {
    expect(invoiceStsState([submission({ status: 'scartata' })], 'produzione').kind).toBe('rejected')
  })

  it('ignores transmissions made in the other environment', () => {
    expect(invoiceStsState([submission({ status: 'accettata', environment: 'test' })], 'produzione').kind).toBe('none')
  })
})

describe('stsActions', () => {
  it('offers the send only for paid invoices not on the Sistema TS', () => {
    expect(stsActions(invoiceStsState([], 'produzione'), 'paid').send).toBe(true)
    expect(stsActions(invoiceStsState([], 'produzione'), 'issued').send).toBe(false)
    expect(stsActions(invoiceStsState([submission({ status: 'scartata' })], 'produzione'), 'paid').send).toBe(true)
  })

  it('offers replace and cancel for accepted data, cancel even if the invoice is no longer paid', () => {
    const accepted = invoiceStsState([submission({ status: 'accettata' })], 'produzione')
    expect(stsActions(accepted, 'paid')).toEqual({ send: false, replace: true, cancel: true, withdraw: false })
    expect(stsActions(accepted, 'cancelled')).toEqual({ send: false, replace: false, cancel: true, withdraw: false })
  })

  it('offers withdrawal only before the call starts', () => {
    expect(stsActions(invoiceStsState([submission({ status: 'non_inviata' })], 'produzione'), 'paid').withdraw).toBe(true)
    expect(stsActions(invoiceStsState([submission({ status: 'inviata' })], 'produzione'), 'paid').withdraw).toBe(false)
  })
})

describe('groupByInvoice', () => {
  it('collects transmissions per invoice', () => {
    const groups = groupByInvoice([
      submission({ id: 1, invoice_id: 10 }),
      submission({ id: 2, invoice_id: 11 }),
      submission({ id: 3, invoice_id: 10 }),
    ])
    expect(groups.get(10)?.map((s) => s.id)).toEqual([1, 3])
    expect(groups.get(11)?.length).toBe(1)
  })
})

describe('stsDeadlines', () => {
  it('moves the 2025 deadline from Saturday 31 January to Monday 2 February', () => {
    expect(stsDeadlines(2025)).toEqual({ transmission: '2026-02-02', correction: '2026-02-09' })
  })

  it('keeps a weekday deadline and adds five days for corrections', () => {
    expect(stsDeadlines(2026)).toEqual({ transmission: '2027-02-01', correction: '2027-02-08' })
    expect(stsDeadlines(2027)).toEqual({ transmission: '2028-01-31', correction: '2028-02-07' })
  })

  it('places today in the right phase', () => {
    expect(stsDeadlinePhase('2025-06-10', '2026-02-02')).toBe('open')
    expect(stsDeadlinePhase('2025-06-10', '2026-02-05')).toBe('correction')
    expect(stsDeadlinePhase('2025-06-10', '2026-02-10')).toBe('late')
  })
})

describe('describeDispatch', () => {
  const empty = { accepted: 0, rejected: 0, retrying: 0, waiting_other_environment: 0, blocked: null }

  it('celebrates a clean pass', () => {
    expect(describeDispatch({ ...empty, accepted: 3 })).toEqual({ message: 'Sistema TS: 3 accettate', tone: 'success' })
  })

  it('flags rejections and retries', () => {
    expect(describeDispatch({ ...empty, accepted: 1, rejected: 1 })).toEqual({
      message: 'Sistema TS: 1 accettata, 1 scartata',
      tone: 'error',
    })
  })

  it('puts the blocking reason first when nothing moved', () => {
    expect(describeDispatch({ ...empty, blocked: 'Manca la password' })).toEqual({ message: 'Manca la password', tone: 'error' })
  })

  it('explains an empty pass', () => {
    expect(describeDispatch(empty).tone).toBe('info')
    expect(describeDispatch({ ...empty, waiting_other_environment: 2 }).message).toContain("nell'altro")
  })
})

describe('formatStsTimestamp', () => {
  it('reads backend UTC timestamps and keeps unparsable ones as they are', () => {
    expect(formatStsTimestamp('2026-09-26 10:05:00')).toMatch(/26 set/)
    expect(formatStsTimestamp('non una data')).toBe('non una data')
  })
})

describe('describeSendKind', () => {
  it('names the Sistema TS operation codes', () => {
    expect(describeSendKind('I')).toBe('Inserito')
    expect(describeSendKind('V')).toBe('Variato')
    expect(describeSendKind(null)).toBe('—')
  })
})

describe('datePhrase', () => {
  it('elides the article before eight and eleven', () => {
    expect(datePhrase('fino', '2027-02-08')).toBe('fino all’8 febbraio 2027')
    expect(datePhrase('entro', '2027-02-11')).toBe('entro l’11 febbraio 2027')
  })

  it('writes the first of the month as an ordinal', () => {
    expect(datePhrase('entro', '2027-02-01')).toBe('entro il 1° febbraio 2027')
  })

  it('keeps the plain article otherwise', () => {
    expect(datePhrase('fino', '2026-02-09')).toBe('fino al 9 febbraio 2026')
    expect(datePhrase('dal', '2026-02-02')).toBe('dal 2 febbraio 2026')
  })
})

describe('stsStateSentence', () => {
  const NONE = { kind: 'none', current: null, live: null, failedFollowUp: null } as const

  it('asks to transmit a paid invoice and to wait for payment otherwise', () => {
    expect(stsStateSentence(NONE, { status: 'paid' })).toBe('Spesa sanitaria da trasmettere.')
    expect(stsStateSentence(NONE, { status: 'issued' })).toBe('Si trasmette quando la fattura risulta pagata.')
  })

  it('names the retry time of a queued submission that failed, else its operation', () => {
    const waiting = submission({ operation: 'sostituzione' })
    expect(stsStateSentence({ ...NONE, kind: 'queued', current: waiting }, { status: 'paid' })).toBe(
      'Sostituzione in attesa di invio.',
    )
    const retrying = submission({ last_error: 'rete', next_attempt_at: '2026-10-06 10:00:00' })
    expect(stsStateSentence({ ...NONE, kind: 'queued', current: retrying }, { status: 'paid' })).toMatch(/^Nuovo tentativo: /)
  })

  it('says a rejected invoice must be corrected and sent again', () => {
    expect(stsStateSentence({ ...NONE, kind: 'rejected' }, { status: 'paid' })).toContain('correggi i dati e ritrasmetti')
  })
})

describe('invoiceDeadlineNotice', () => {
  const paid = { status: 'paid', paid_date: '2026-03-10' } as const

  it('has nothing to say about an unpaid invoice', () => {
    expect(invoiceDeadlineNotice({ status: 'issued', paid_date: undefined }, false, '2026-10-06')).toBeNull()
  })

  it('asks to send before the deadline, then only offers corrections once registered', () => {
    expect(invoiceDeadlineNotice(paid, false, '2026-10-06')?.tone).toBe('accent')
    expect(invoiceDeadlineNotice(paid, true, '2026-10-06')?.tone).toBe('neutral')
  })

  it('warns inside the correction window and flags a late transmission only when still pending', () => {
    const { transmission, correction } = stsDeadlines(2026)
    const inWindow = transmission < correction ? correction : transmission
    expect(invoiceDeadlineNotice(paid, true, inWindow)?.tone).not.toBe('danger')
    expect(invoiceDeadlineNotice(paid, false, '2028-01-01')?.tone).toBe('danger')
    expect(invoiceDeadlineNotice(paid, true, '2028-01-01')).toBeNull()
  })
})

function progress(overrides: Partial<TsDispatchProgress>, summary: Partial<TsDispatchProgress['summary']> = {}): TsDispatchProgress {
  return {
    total: 4,
    processed: 0,
    finished: false,
    ...overrides,
    summary: { accepted: 0, rejected: 0, retrying: 0, waiting_other_environment: 0, blocked: null, ...summary },
  }
}

describe('cancellableSubmissionIds', () => {
  it('offers the live send of an accepted invoice', () => {
    const sent = submission({ id: 1, status: 'accettata' })
    expect([...cancellableSubmissionIds(groupByInvoice([sent]), 'produzione')]).toEqual([1])
  })

  it('withholds it while a cancellation waits in the queue', () => {
    const sent = submission({ id: 1, status: 'accettata' })
    const cancelling = submission({ id: 2, operation: 'annullamento', status: 'non_inviata', target_submission_id: 1 })
    expect(cancellableSubmissionIds(groupByInvoice([sent, cancelling]), 'produzione').size).toBe(0)
  })

  it('withholds it once the cancellation is accepted', () => {
    const sent = submission({ id: 1, status: 'annullata' })
    const cancelled = submission({ id: 2, operation: 'annullamento', status: 'accettata', target_submission_id: 1 })
    expect(cancellableSubmissionIds(groupByInvoice([sent, cancelled]), 'produzione').size).toBe(0)
  })

  it('ignores the other environment', () => {
    const sent = submission({ id: 1, status: 'accettata', environment: 'test' })
    expect(cancellableSubmissionIds(groupByInvoice([sent]), 'produzione').size).toBe(0)
  })
})

describe('historyGroup and historyBadge', () => {
  it('files an accepted cancellation with the cancelled, not with the accepted', () => {
    const cancellation = { operation: 'annullamento', status: 'accettata' } as const
    expect(historyGroup(cancellation)).toBe('cancelled')
    expect(historyBadge(cancellation)).toEqual({ label: 'Cancellata dal TS', tone: 'neutral' })
  })

  it('keeps accepted sends and replacements under accepted', () => {
    expect(historyGroup({ operation: 'invio', status: 'accettata' })).toBe('accepted')
    expect(historyGroup({ operation: 'sostituzione', status: 'accettata' })).toBe('accepted')
    expect(historyBadge({ operation: 'invio', status: 'accettata' }).label).toBe('Accettata')
  })

  it('sorts the remaining statuses', () => {
    expect(historyGroup({ operation: 'invio', status: 'annullata' })).toBe('cancelled')
    expect(historyGroup({ operation: 'invio', status: 'inviata' })).toBe('queue')
    expect(historyGroup({ operation: 'invio', status: 'scartata' })).toBe('rejected')
    expect(historyGroup({ operation: 'invio', status: 'sostituita' })).toBe('retired')
  })
})

describe('dispatch progress', () => {
  it('measures the share answered so far', () => {
    expect(dispatchPercent(progress({ processed: 1, total: 3 }))).toBe(33)
    expect(dispatchPercent(progress({ processed: 3, total: 3 }))).toBe(100)
    expect(dispatchPercent(progress({ processed: 0, total: 0 }))).toBe(0)
  })

  it('leaves a stopped pass where it stopped', () => {
    expect(dispatchPercent(progress({ processed: 1, total: 4, finished: true }, { blocked: 'Servizio non raggiungibile' }))).toBe(25)
  })
})

describe('outcomeLabel', () => {
  it('agrees the noun with the count', () => {
    expect(outcomeLabel('accepted', 1)).toBe('1 accettata')
    expect(outcomeLabel('rejected', 3)).toBe('3 scartate')
    expect(outcomeLabel('retrying', 2)).toBe('2 da ritentare')
  })
})

describe('finished pass', () => {
  it('judges the pass from its summary', () => {
    expect(dispatchVerdict(progress({}, { accepted: 4 }).summary)).toBe('done')
    expect(dispatchVerdict(progress({}, { accepted: 3, rejected: 1 }).summary)).toBe('attention')
    expect(dispatchVerdict(progress({}, { retrying: 1, blocked: 'Credenziali rifiutate.' }).summary)).toBe('interrupted')
  })

  it('reports a clean pass', () => {
    expect(finishedDispatchReport(progress({ processed: 4, finished: true }, { accepted: 4 }))).toEqual({
      title: 'Invio completato',
      sentence: 'Tutte le 4 fatture sono state accettate dal Sistema TS.',
    })
  })

  it('points to the rejected ones', () => {
    expect(finishedDispatchReport(progress({ processed: 4, finished: true }, { accepted: 3, rejected: 1 })).sentence).toContain('scheda Trasmissioni')
  })

  it('gives the reason of an interrupted pass', () => {
    expect(finishedDispatchReport(progress({ processed: 1, finished: true }, { retrying: 1, blocked: 'Sistema TS non raggiungibile.' }))).toEqual({
      title: 'Invio interrotto',
      sentence: 'Sistema TS non raggiungibile. Le fatture non inviate restano in coda.',
    })
  })

  it('settles a pass whose last event is late from the returned summary', () => {
    const settled = settleProgress(progress({ processed: 2 }), { ...progress({}).summary, accepted: 3, rejected: 1 })
    expect(settled).toMatchObject({ processed: 4, finished: true })
    expect(settled.summary.rejected).toBe(1)
  })

  it('never moves a settled pass backwards', () => {
    expect(settleProgress(progress({ processed: 3 }), { ...progress({}).summary, accepted: 1 }).processed).toBe(3)
  })
})

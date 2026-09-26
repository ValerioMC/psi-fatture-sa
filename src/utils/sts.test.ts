import { describe, expect, it } from 'vitest'
import type { TsSubmission } from '@/types'
import {
  datePhrase,
  describeDispatch,
  describeSendKind,
  formatStsTimestamp,
  groupByInvoice,
  invoiceStsState,
  stsActions,
  stsDeadlinePhase,
  stsDeadlines,
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

import { describe, expect, it } from 'vitest'
import { deriveSeal, describeSeal } from './invoiceSeal'

const TODAY = new Date(2026, 8, 24)

describe('deriveSeal', () => {
  it('keeps draft, paid and cancelled as they are, without a clock', () => {
    for (const status of ['draft', 'paid', 'cancelled'] as const) {
      expect(deriveSeal({ status, issue_date: '2026-09-01', due_date: '2026-10-01' }, TODAY)).toEqual({
        kind: status,
        elapsed: null,
        daysToDue: null,
      })
    }
  })

  it('measures how much of the payment window has passed', () => {
    const seal = deriveSeal({ status: 'issued', issue_date: '2026-09-14', due_date: '2026-10-04' }, TODAY)
    expect(seal.kind).toBe('issued')
    expect(seal.elapsed).toBeCloseTo(0.5)
    expect(seal.daysToDue).toBe(10)
  })

  it('reads an issued invoice past its due date as overdue', () => {
    const seal = deriveSeal({ status: 'issued', issue_date: '2026-08-01', due_date: '2026-08-31' }, TODAY)
    expect(seal).toEqual({ kind: 'overdue', elapsed: 1, daysToDue: -24 })
  })

  it('shows an issued invoice without due date as a plain ring', () => {
    expect(deriveSeal({ status: 'issued', issue_date: '2026-09-01' }, TODAY)).toEqual({ kind: 'issued', elapsed: null, daysToDue: null })
  })

  it('counts the due day itself as still in time', () => {
    const seal = deriveSeal({ status: 'issued', issue_date: '2026-09-01', due_date: '2026-09-24' }, TODAY)
    expect(seal.kind).toBe('issued')
    expect(seal.elapsed).toBe(1)
    expect(describeSeal(seal)).toBe('Emessa, scade oggi')
  })
})

describe('describeSeal', () => {
  it('says how long an invoice has been overdue, with the right plural', () => {
    expect(describeSeal({ kind: 'overdue', elapsed: 1, daysToDue: -1 })).toBe('Scaduta da 1 giorno')
    expect(describeSeal({ kind: 'overdue', elapsed: 1, daysToDue: -12 })).toBe('Scaduta da 12 giorni')
  })

  it('says how long is left on an issued invoice', () => {
    expect(describeSeal({ kind: 'issued', elapsed: 0.2, daysToDue: 1 })).toBe('Emessa, scade tra 1 giorno')
  })
})

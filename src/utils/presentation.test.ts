import { describe, expect, it } from 'vitest'
import { initialsOf, tintIndexOf, MONOGRAM_TINT_COUNT } from './monogram'
import { ageOn, clientDisplayName } from './client'
import { plural } from './labels'
import { hasPrimaryModifier, shortcutLabel } from './platform'
import { summariseAttention } from './attention'
import { weeklyDates } from './recurrence'

describe('monogram', () => {
  it('takes the first letter of the first two words', () => {
    expect(initialsOf('Rossi Maria')).toBe('RM')
    expect(initialsOf("  D'Angelo   Serena ")).toBe('DS')
  })

  it('uses two letters of a single word and a placeholder for nothing', () => {
    expect(initialsOf('Hera')).toBe('HE')
    expect(initialsOf('')).toBe('?')
  })

  it('gives the same name the same tint, whatever the case', () => {
    expect(tintIndexOf('Rossi Maria')).toBe(tintIndexOf('rossi maria'))
    const tint = tintIndexOf('Bianchi Luca')
    expect(tint).toBeGreaterThanOrEqual(1)
    expect(tint).toBeLessThanOrEqual(MONOGRAM_TINT_COUNT)
  })
})

describe('client presentation', () => {
  it('puts the surname first for people and the business name for companies', () => {
    expect(clientDisplayName({ client_type: 'persona_fisica', first_name: 'Maria', last_name: 'Rossi' })).toBe('Rossi Maria')
    expect(clientDisplayName({ client_type: 'azienda', first_name: '', last_name: 'Studio Hera' })).toBe('Studio Hera')
  })

  it('computes age on a given day, before and after the birthday', () => {
    expect(ageOn('1990-09-25', new Date(2026, 8, 24))).toBe(35)
    expect(ageOn('1990-09-24', new Date(2026, 8, 24))).toBe(36)
    expect(ageOn(undefined)).toBeNull()
  })
})

describe('plural', () => {
  it('agrees the noun with the count', () => {
    expect(plural(1, 'fattura', 'fatture')).toBe('1 fattura')
    expect(plural(0, 'fattura', 'fatture')).toBe('0 fatture')
  })
})

describe('platform shortcuts', () => {
  const mac = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0)'
  const windows = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'

  it('labels the primary modifier per platform', () => {
    expect(shortcutLabel('K', mac)).toBe('⌘K')
    expect(shortcutLabel('K', windows)).toBe('Ctrl K')
  })

  it('accepts only that modifier', () => {
    expect(hasPrimaryModifier({ metaKey: true, ctrlKey: false }, mac)).toBe(true)
    expect(hasPrimaryModifier({ metaKey: false, ctrlKey: true }, mac)).toBe(false)
    expect(hasPrimaryModifier({ metaKey: false, ctrlKey: true }, windows)).toBe(true)
  })
})

describe('summariseAttention', () => {
  it('buckets invoices by what they need, counting late issued ones as overdue', () => {
    const today = new Date(2026, 8, 24)
    const attention = summariseAttention(
      [
        { status: 'issued', issue_date: '2026-08-01', due_date: '2026-08-31', total_due: 100 },
        { status: 'overdue', issue_date: '2026-07-01', due_date: '2026-07-31', total_due: 50.1 },
        { status: 'issued', issue_date: '2026-09-01', due_date: '2026-10-01', total_due: 80 },
        { status: 'draft', issue_date: '2026-09-20', total_due: 70 },
        { status: 'paid', issue_date: '2026-06-01', total_due: 999 },
      ],
      today,
    )
    expect(attention.overdue).toEqual({ count: 2, amount: 150.1 })
    expect(attention.awaiting).toEqual({ count: 1, amount: 80 })
    expect(attention.drafts).toEqual({ count: 1, amount: 70 })
  })
})

describe('weeklyDates', () => {
  it('lists every chosen weekday for the given weeks, starting on the start day', () => {
    // 24 Sept 2026 is a Thursday (4); Monday is 1.
    expect(weeklyDates('2026-09-24', [4, 1], 2)).toEqual(['2026-09-24', '2026-09-28', '2026-10-01', '2026-10-05'])
  })

  it('does not slip a day across the daylight-saving change', () => {
    // Italy leaves summer time on 25 Oct 2026.
    expect(weeklyDates('2026-10-22', [4], 2)).toEqual(['2026-10-22', '2026-10-29'])
  })

  it('returns nothing without weekdays or weeks', () => {
    expect(weeklyDates('2026-09-24', [], 4)).toEqual([])
    expect(weeklyDates('2026-09-24', [1], 0)).toEqual([])
  })
})

import { describe, expect, it } from 'vitest'
import { weeklyDates } from './recurrence'

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

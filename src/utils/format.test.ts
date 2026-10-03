import { describe, expect, it } from 'vitest'
import {
  formatBytes,
  formatCurrency,
  formatCurrencyCompact,
  formatDateShort,
  formatLocalTimestamp,
  formatMonthYear,
  minutesBetween,
  parseIsoDate,
  splitCurrency,
  toIsoDate,
} from './format'

describe('currency', () => {
  it('groups thousands even for four-digit amounts', () => {
    expect(formatCurrency(2583.1).replace(/\s/g, ' ')).toBe('2.583,10 €')
  })

  it('splits euros from cents for hero figures', () => {
    expect(splitCurrency(62340.7)).toEqual({ whole: '62.340', fraction: ',70', symbol: '€' })
  })

  it('compacts axis labels to thousands', () => {
    expect(formatCurrencyCompact(12500)).toBe('12,5k €')
    expect(formatCurrencyCompact(20000)).toBe('20k €')
    expect(formatCurrencyCompact(800)).toBe('800 €')
  })
})

describe('dates', () => {
  it('formats the local calendar day, never the UTC one', () => {
    expect(toIsoDate(new Date(2026, 2, 29, 0, 30))).toBe('2026-03-29')
    expect(toIsoDate(parseIsoDate('2026-12-31'))).toBe('2026-12-31')
  })

  it('writes short and month labels in Italian', () => {
    expect(formatDateShort('2026-09-04')).toBe('4 set')
    expect(formatMonthYear(2026, 9)).toBe('Settembre 2026')
  })

  it('measures a session in minutes, zero when inverted', () => {
    expect(minutesBetween('09:30', '10:20')).toBe(50)
    expect(minutesBetween('10:00', '09:00')).toBe(0)
  })
})

describe('file sizes', () => {
  it('keeps bytes whole and gives larger sizes one decimal', () => {
    expect(formatBytes(512)).toBe('512 B')
    expect(formatBytes(1_534_000)).toBe('1,5 MB')
    expect(formatBytes(48_000)).toBe('48 KB')
    expect(formatBytes(2_100_000_000)).toBe('2,1 GB')
  })
})

describe('local timestamps', () => {
  it('reads the backend stamp as local time, with the year', () => {
    expect(formatLocalTimestamp('2026-10-03 09:15:00')).toBe('3 ott 2026, 09:15')
  })

  it('leaves an unreadable stamp as it is', () => {
    expect(formatLocalTimestamp('ieri')).toBe('ieri')
  })
})

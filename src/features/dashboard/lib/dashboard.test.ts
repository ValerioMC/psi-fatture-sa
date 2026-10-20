import { describe, expect, it } from 'vitest'
import type { DashboardData } from '@/features/dashboard/model/dashboard'
import type { MonthlyInvoicePreview } from '@/features/invoice/model/invoice'
import { collectionRate, greeting, niceCeiling, previousMonth, taxEstimate, unbilledMonth } from './dashboard'

describe('previousMonth', () => {
  it('steps back one month, crossing into last year from January', () => {
    expect(previousMonth(2026, 10)).toEqual({ year: 2026, month: 9 })
    expect(previousMonth(2026, 1)).toEqual({ year: 2025, month: 12 })
  })
})

describe('unbilledMonth', () => {
  const preview = (appointment_count: number, estimated_due: number) => ({ appointment_count, estimated_due }) as MonthlyInvoicePreview

  it('sums sessions and amounts, or says nothing is left', () => {
    expect(unbilledMonth([preview(3, 246.8), preview(1, 81.6)], 2026, 9)).toEqual({ month: 9, year: 2026, sessions: 4, amount: 328.4 })
    expect(unbilledMonth([], 2026, 9)).toBeNull()
  })
})

describe('collectionRate', () => {
  it('is the paid share of the invoiced amount', () => {
    expect(collectionRate({ total_revenue: 300, paid_revenue: 100 } as DashboardData)).toBe(33)
    expect(collectionRate({ total_revenue: 0, paid_revenue: 0 } as DashboardData)).toBe(0)
    expect(collectionRate(null)).toBe(0)
  })
})

describe('taxEstimate', () => {
  it('names the substitute tax rate in the forfettario regime', () => {
    expect(taxEstimate(10000, 78, true, true)?.taxLabel).toBe('Imposta sostitutiva 5%')
    expect(taxEstimate(10000, 78, true, false)?.taxLabel).toBe('Imposta sostitutiva 15%')
  })

  it('adds IRPEF and the surcharges in the ordinario regime', () => {
    const estimate = taxEstimate(30000, 78, false, false)
    expect(estimate?.taxLabel).toBe('IRPEF e addizionali')
    expect(estimate?.revenue).toBe(30000)
  })

  it('has nothing to estimate before any revenue', () => {
    expect(taxEstimate(0, 78, true, false)).toBeNull()
  })
})

describe('greeting', () => {
  it('follows the time of day and adds the name when known', () => {
    expect(greeting(9, 'Maria')).toBe('Buongiorno, Maria')
    expect(greeting(15, undefined)).toBe('Buon pomeriggio')
    expect(greeting(20, '')).toBe('Buonasera')
  })
})

describe('niceCeiling', () => {
  it('gives an empty chart a default axis', () => {
    expect(niceCeiling(0)).toBe(1000)
  })

  it('rounds up to a clean step without doubling the headroom', () => {
    expect(niceCeiling(7300)).toBe(8000)
    expect(niceCeiling(8000)).toBe(8000)
    expect(niceCeiling(8001)).toBe(10000)
    expect(niceCeiling(95)).toBe(100)
  })
})

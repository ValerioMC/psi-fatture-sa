import { describe, expect, it } from 'vitest'
import type { MonthlyRevenue } from '@/features/dashboard/model/dashboard'
import {
  bestMonth,
  heightShare,
  isEmptySlot,
  issuedAverage,
  monthColumns,
  segmentShare,
  yearSplit,
} from './monthlyChart'

const month = (number: number, revenue: number): MonthlyRevenue => ({ month: number, month_name: `m${number}`, revenue, invoice_count: revenue > 0 ? 1 : 0 })

describe('monthColumns', () => {
  it('stacks paid, pending and draft by month, counting only paid and pending as issued', () => {
    const [march] = monthColumns([month(3, 100)], [{ amount: 0, count: 0 }, { amount: 0, count: 0 }, { amount: 40, count: 2 }], [{ amount: 0, count: 0 }, { amount: 0, count: 0 }, { amount: 10, count: 1 }])
    expect(march).toMatchObject({ paid: 100, pending: 40, pendingCount: 2, draft: 10, draftCount: 1, total: 150, issued: 140 })
  })

  it('reads a month missing from pending or drafts as nothing', () => {
    const [june] = monthColumns([month(6, 50)], [], [])
    expect(june).toMatchObject({ pending: 0, draft: 0, total: 50, issued: 50 })
  })
})

describe('average and best month', () => {
  const columns = monthColumns([month(1, 100), month(2, 300), month(3, 0), month(4, 900)], [], [])

  it('averages the lived months that had any, leaving out months ahead', () => {
    expect(issuedAverage(columns, 3)).toBe(200)
    expect(bestMonth(columns, 3)).toBe(2)
  })

  it('counts every month of a past year', () => {
    expect(issuedAverage(columns, null)).toBeCloseTo(433.33, 2)
    expect(bestMonth(columns, null)).toBe(4)
  })

  it('has no average with fewer than two lived months', () => {
    expect(issuedAverage(columns, 1)).toBeNull()
    expect(bestMonth(monthColumns([month(1, 0)], [], []), 1)).toBeNull()
  })
})

describe('shape', () => {
  const [small, empty] = monthColumns([month(5, 1), month(11, 0)], [], [])

  it('keeps a small column visible and an empty month ahead as a slot', () => {
    expect(heightShare(small, 1000)).toBe(1.5)
    expect(heightShare(empty, 1000)).toBe(0)
    expect(isEmptySlot(empty, 10)).toBe(true)
    expect(isEmptySlot(empty, null)).toBe(false)
  })

  it('splits a column into its segments and the year into its totals', () => {
    const [column] = monthColumns([month(1, 75)], [{ amount: 25, count: 1 }], [])
    expect(segmentShare(column, column.pending)).toBe('25%')
    expect(segmentShare(empty, 0)).toBe('0%')
    expect(yearSplit([column, small])).toEqual({ paid: 76, pending: 25, draft: 0 })
  })
})

import { describe, expect, it } from 'vitest'
import { pendingByMonth, type InvoiceForPending } from './monthlyPending'

const invoice = (status: InvoiceForPending['status'], issue_date: string, total_due: number): InvoiceForPending => ({ status, issue_date, total_due })

describe('pendingByMonth', () => {
  it('returns twelve empty months for no invoices', () => {
    const months = pendingByMonth([])
    expect(months).toHaveLength(12)
    expect(months.every((month) => month.amount === 0 && month.count === 0)).toBe(true)
  })

  it('sums issued and overdue invoices by month of issue', () => {
    const months = pendingByMonth([
      invoice('issued', '2026-08-31', 83.6),
      invoice('overdue', '2026-08-02', 100.1),
      invoice('issued', '2026-09-30', 50),
    ])
    expect(months[7]).toEqual({ amount: 183.7, count: 2 })
    expect(months[8]).toEqual({ amount: 50, count: 1 })
  })

  it('ignores paid, draft and cancelled invoices', () => {
    const months = pendingByMonth([
      invoice('paid', '2026-03-10', 80),
      invoice('draft', '2026-03-11', 80),
      invoice('cancelled', '2026-03-12', 80),
    ])
    expect(months[2]).toEqual({ amount: 0, count: 0 })
  })

  it('skips an invoice with a malformed date instead of throwing', () => {
    expect(() => pendingByMonth([invoice('issued', 'bad', 10)])).not.toThrow()
  })
})

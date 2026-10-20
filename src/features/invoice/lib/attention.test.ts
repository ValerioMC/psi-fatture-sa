import { describe, expect, it } from 'vitest'
import { summariseAttention } from './attention'

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

/**
 * What needs the psychologist's attention, derived from the year's invoices:
 * money that is late, money that is on its way, and drafts not yet issued.
 * Overdue follows the seal: an issued invoice past its due date counts as
 * overdue even before anyone changes its status.
 */
import type { InvoiceStatus } from '@/types'
import { deriveSeal } from './invoiceSeal'

export interface InvoiceForAttention {
  status: InvoiceStatus
  issue_date: string
  due_date?: string | null
  total_due: number
}

export interface Bucket {
  count: number
  amount: number
}

export interface Attention {
  overdue: Bucket
  awaiting: Bucket
  drafts: Bucket
}

const empty = (): Bucket => ({ count: 0, amount: 0 })

export function summariseAttention(invoices: readonly InvoiceForAttention[], today: Date = new Date()): Attention {
  const attention: Attention = { overdue: empty(), awaiting: empty(), drafts: empty() }
  for (const invoice of invoices) {
    const kind = deriveSeal(invoice, today).kind
    const bucket =
      kind === 'overdue' ? attention.overdue
        : kind === 'issued' ? attention.awaiting
          : kind === 'draft' ? attention.drafts
            : null
    if (bucket === null) continue
    bucket.count += 1
    bucket.amount = Math.round((bucket.amount + invoice.total_due) * 100) / 100
  }
  return attention
}

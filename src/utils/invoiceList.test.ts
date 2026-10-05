import { describe, expect, it } from 'vitest'
import type { InvoiceStatus } from '@/types'
import { formatCurrency } from './format'
import {
  describeBulkEmail,
  describeBulkEmailOutcome,
  describeBulkStatus,
  describeSendProgress,
  invoicesInStatus,
  isEmailable,
  listQuery,
  monthlyTotals,
  paidShare,
  searchInvoices,
  statusDistribution,
  statusFilterOptions,
  statusFromQuery,
  summarise,
  yearFromQuery,
} from './invoiceList'

const invoice = (status: InvoiceStatus, total_due: number, issue_date = '2026-03-10', client_name = 'Anna Bianchi', invoice_number = '001') => ({
  status,
  total_due,
  issue_date,
  client_name,
  invoice_number,
})

describe('URL filters', () => {
  it('reads the year, falling back to this year when unreadable', () => {
    expect(yearFromQuery('2024', 2026)).toBe(2024)
    expect(yearFromQuery('0', 2026)).toBe(0)
    expect(yearFromQuery(undefined, 2026)).toBe(2026)
    expect(yearFromQuery('-1', 2026)).toBe(2026)
  })

  it('reads only known statuses', () => {
    expect(statusFromQuery('overdue')).toBe('overdue')
    expect(statusFromQuery('nonsense')).toBe('all')
    expect(statusFromQuery(undefined)).toBe('all')
  })

  it('writes only what differs from the default view', () => {
    expect(listQuery(2026, 'all', 2026)).toEqual({})
    expect(listQuery(2025, 'paid', 2026)).toEqual({ year: 2025, status: 'paid' })
  })
})

describe('searchInvoices', () => {
  const list = [invoice('paid', 10, '2026-01-01', 'Nicolò Rossi', '004'), invoice('paid', 10, '2026-01-01', 'Anna Bianchi', '012')]

  it('ignores case and accents', () => {
    expect(searchInvoices(list, ' NICOLO ').map((i) => i.invoice_number)).toEqual(['004'])
  })

  it('matches the number too and returns everything for an empty query', () => {
    expect(searchInvoices(list, '012').map((i) => i.client_name)).toEqual(['Anna Bianchi'])
    expect(searchInvoices(list, '  ')).toBe(list)
  })
})

describe('status filter', () => {
  const list = [invoice('paid', 10), invoice('overdue', 20), invoice('paid', 30)]

  it('keeps one status or all', () => {
    expect(invoicesInStatus(list, 'paid')).toHaveLength(2)
    expect(invoicesInStatus(list, 'all')).toBe(list)
  })

  it('counts every status for the tabs', () => {
    const options = statusFilterOptions(list)
    expect(options[0]).toEqual({ value: 'all', label: 'Tutte', count: 3 })
    expect(options.find((o) => o.value === 'paid')?.count).toBe(2)
    expect(options.find((o) => o.value === 'draft')?.count).toBe(0)
  })
})

describe('summarise', () => {
  it('leaves cancelled invoices out of the amounts but counts them in the list', () => {
    const summary = summarise([invoice('paid', 100), invoice('issued', 50), invoice('overdue', 25), invoice('cancelled', 999), invoice('draft', 10)])
    expect(summary).toEqual({ count: 5, total: 185, paid: 100, pending: 75, overdue: 1 })
  })

  it('gives the paid share as a whole percentage', () => {
    expect(paidShare({ count: 2, total: 300, paid: 100, pending: 200, overdue: 0 })).toBe(33)
    expect(paidShare({ count: 0, total: 0, paid: 0, pending: 0, overdue: 0 })).toBe(0)
  })
})

describe('monthlyTotals', () => {
  it('adds up the amount due by month of issue, skipping cancelled invoices', () => {
    const totals = monthlyTotals([invoice('paid', 100, '2026-01-31'), invoice('issued', 50, '2026-01-02'), invoice('cancelled', 70, '2026-12-01')])
    expect(totals).toHaveLength(12)
    expect(totals[0]).toBe(150)
    expect(totals[11]).toBe(0)
  })
})

describe('statusDistribution', () => {
  it('lists only present statuses in display order with their share', () => {
    expect(statusDistribution([invoice('paid', 1), invoice('draft', 1), invoice('paid', 1), invoice('paid', 1)])).toEqual([
      { value: 'draft', label: 'Bozza', count: 1, share: 25 },
      { value: 'paid', label: 'Pagata', count: 3, share: 75 },
    ])
    expect(statusDistribution([])).toEqual([])
  })
})

describe('bulk confirmations', () => {
  it('says how many invoices and how much a status change moves', () => {
    expect(describeBulkStatus([invoice('issued', 80), invoice('issued', 20)], 'paid')).toBe(
      `2 fatture per ${formatCurrency(100)} passeranno a “pagata”, con data di pagamento oggi.`,
    )
    expect(describeBulkStatus([invoice('issued', 80)], 'overdue')).toBe(`1 fattura per ${formatCurrency(80)} passerà a “scaduta”.`)
  })

  it('never emails drafts or cancelled invoices', () => {
    expect(['draft', 'cancelled', 'issued', 'paid', 'overdue'].map((s) => isEmailable({ status: s as InvoiceStatus }))).toEqual([false, false, true, true, true])
  })

  it('names resends and skipped invoices only when there are some', () => {
    expect(describeBulkEmail(2, 0, 0)).toBe('2 fatture partiranno via email ai pazienti, con il PDF allegato e il modello salvato.')
    expect(describeBulkEmail(1, 1, 2)).toBe(
      '1 fattura partirà via email ai pazienti, con il PDF allegato e il modello salvato. 1 era già stata inviata e arriverà di nuovo. 2 bozze o annullate restano fuori.',
    )
  })

  it('counts the sending progress from one', () => {
    expect(describeSendProgress({ done: 0, total: 3 })).toBe('Invio 1 di 3…')
    expect(describeSendProgress({ done: 3, total: 3 })).toBe('Invio 3 di 3…')
    expect(describeSendProgress(null)).toBe('Chi non ha un indirizzo email nella scheda viene saltato.')
  })

  it('reports the first failure after a partial send', () => {
    expect(describeBulkEmailOutcome(3, [])).toBe('3 fatture inviate ai pazienti')
    expect(describeBulkEmailOutcome(1, ['N. 004: rifiutata', 'N. 005: rifiutata'])).toBe('1 inviata, 2 non riuscite. N. 004: rifiutata')
  })
})

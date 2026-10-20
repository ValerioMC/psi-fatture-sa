import { describe, expect, it } from 'vitest'
import type { Invoice, InvoiceStatus } from '@/features/invoice/model/invoice'
import { historyTotal, newestFirst, patientsInCare, type InvoiceForCare } from './patientInvoices'

describe('history', () => {
  const invoice = (status: InvoiceStatus, total_due: number, issue_date: string) => ({ status, total_due, issue_date }) as Invoice

  it('sorts newest first and totals all but cancelled invoices', () => {
    const list = [invoice('paid', 100, '2026-01-10'), invoice('cancelled', 50, '2026-03-01'), invoice('issued', 30, '2026-02-01')]
    expect(newestFirst(list).map((i) => i.issue_date)).toEqual(['2026-03-01', '2026-02-01', '2026-01-10'])
    expect(historyTotal(list)).toBe(130)
  })
})

describe('patientsInCare', () => {
  const today = new Date(2026, 9, 2)
  const invoice = (client_id: number, issue_date: string, status: InvoiceForCare['status'] = 'issued'): InvoiceForCare => ({ client_id, status, issue_date })

  it('counts each patient billed in the window once', () => {
    const ids = patientsInCare([invoice(1, '2026-10-02'), invoice(1, '2026-09-01'), invoice(2, '2026-07-05')], 90, today)
    expect([...ids].sort()).toEqual([1, 2])
  })

  it('excludes invoices outside the window or cancelled', () => {
    const ids = patientsInCare([invoice(3, '2026-07-04'), invoice(4, '2026-10-03'), invoice(5, '2026-09-10', 'cancelled')], 90, today)
    expect(ids.size).toBe(0)
  })
})

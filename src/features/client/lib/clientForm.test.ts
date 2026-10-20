import { describe, expect, it } from 'vitest'
import type { InvoiceStatus } from '@/features/invoice/model/invoice'
import { blankClientForm, cardDescription, cardPlace, checkClientField, historyTotal, newestFirst } from './clientForm'
import type { Invoice } from '@/features/invoice/model/invoice'

describe('checkClientField', () => {
  it('asks a person for first name and surname, a company only for its name', () => {
    const person = blankClientForm()
    expect(checkClientField('first_name', person)).toEqual({ valid: false, message: 'Serve il nome.' })
    expect(checkClientField('last_name', person)).toEqual({ valid: false, message: 'Serve il cognome.' })
    const company = { ...blankClientForm(), client_type: 'azienda' as const }
    expect(checkClientField('first_name', company).valid).toBe(true)
    expect(checkClientField('last_name', company)).toEqual({ valid: false, message: 'Serve la ragione sociale.' })
  })

  it('requires the fiscal code before checking its shape', () => {
    expect(checkClientField('fiscal_code', blankClientForm())).toEqual({ valid: false, message: 'Serve il codice fiscale.' })
    expect(checkClientField('fiscal_code', { ...blankClientForm(), fiscal_code: 'RSSMRA85T10A562S' }).valid).toBe(true)
    expect(checkClientField('fiscal_code', { ...blankClientForm(), fiscal_code: 'XXX' }).valid).toBe(false)
  })

  it('checks the P.IVA of a company only', () => {
    expect(checkClientField('vat_number', { ...blankClientForm(), vat_number: '123' }).valid).toBe(true)
    expect(checkClientField('vat_number', { ...blankClientForm(), client_type: 'azienda', vat_number: '123' }).valid).toBe(false)
  })
})

describe('record card', () => {
  const today = new Date(2026, 9, 5)

  it('describes a person by age when the birth date is known', () => {
    expect(cardDescription({ client_type: 'persona_fisica', birth_date: '1990-10-06' }, today)).toBe('35 anni')
    expect(cardDescription({ client_type: 'persona_fisica', birth_date: undefined }, today)).toBe('Persona fisica')
    expect(cardDescription({ client_type: 'azienda', birth_date: '1990-01-01' }, today)).toBe('Azienda o ente')
  })

  it('writes the place on one line and leaves out blank parts', () => {
    expect(cardPlace({ address: 'Via Roma 1', zip_code: '20100', city: 'Milano', province: 'MI' })).toEqual(['Via Roma 1', '20100 Milano (MI)'])
    expect(cardPlace({ address: ' ', zip_code: '', city: 'Milano', province: '' })).toEqual(['Milano'])
  })
})

describe('history', () => {
  const invoice = (status: InvoiceStatus, total_due: number, issue_date: string) => ({ status, total_due, issue_date }) as Invoice

  it('sorts newest first and totals all but cancelled invoices', () => {
    const list = [invoice('paid', 100, '2026-01-10'), invoice('cancelled', 50, '2026-03-01'), invoice('issued', 30, '2026-02-01')]
    expect(newestFirst(list).map((i) => i.issue_date)).toEqual(['2026-03-01', '2026-02-01', '2026-01-10'])
    expect(historyTotal(list)).toBe(130)
  })
})

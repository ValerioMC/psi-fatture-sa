import { describe, expect, it } from 'vitest'
import { missingFields, patientsInCare, type ClientForCompleteness, type InvoiceForCare } from './patientRegister'

const complete: ClientForCompleteness = {
  client_type: 'persona_fisica',
  fiscal_code: 'RSSMRA80A01H501U',
  vat_number: undefined,
  address: 'Via Roma 1',
  city: 'Roma',
  zip_code: '00100',
  email: 'mario@example.it',
}

describe('missingFields', () => {
  it('returns nothing for a complete record', () => {
    expect(missingFields(complete)).toEqual([])
  })

  it('lists every gap in form order', () => {
    expect(missingFields({ ...complete, fiscal_code: ' ', zip_code: '', email: undefined })).toEqual(['codice fiscale', 'indirizzo', 'email'])
  })

  it('accepts a VAT number in place of the fiscal code for companies', () => {
    expect(missingFields({ ...complete, client_type: 'azienda', fiscal_code: '', vat_number: '01234567890' })).toEqual([])
    expect(missingFields({ ...complete, client_type: 'persona_fisica', fiscal_code: '', vat_number: '01234567890' })).toEqual(['codice fiscale'])
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

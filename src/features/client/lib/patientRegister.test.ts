import { describe, expect, it } from 'vitest'
import { missingFields, type ClientForCompleteness } from './patientRegister'

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

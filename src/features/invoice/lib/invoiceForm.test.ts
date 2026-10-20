import { describe, expect, it } from 'vitest'
import type { Invoice } from '@/features/invoice/model/invoice'
import {
  blankInvoiceForm,
  blankLine,
  createInvoiceInput,
  enpapDescription,
  formFromInvoice,
  invoiceFormErrors,
  lineError,
  updateInvoiceInput,
  type InvoiceForm,
} from './invoiceForm'

function filledForm(overrides: Partial<InvoiceForm> = {}): InvoiceForm {
  const form = blankInvoiceForm({ ...blankLine(0), description: 'Colloquio', unit_price: 80, service_id: 3 }, '2026-03-10')
  return { ...form, client_id: 7, ...overrides }
}

describe('invoiceFormErrors', () => {
  it('accepts a complete form', () => {
    expect(invoiceFormErrors(filledForm(), false)).toEqual({})
  })

  it('asks for the patient, the issue date and a due date not before it', () => {
    const errors = invoiceFormErrors(filledForm({ client_id: null, issue_date: '', due_date: '2026-01-01' }), false)
    expect(Object.keys(errors)).toEqual(['client', 'issue_date'])
    expect(invoiceFormErrors(filledForm({ due_date: '2026-03-09' }), false).due_date).toBe('La scadenza non può precedere l’emissione.')
  })

  it('checks the number only when editing', () => {
    expect(invoiceFormErrors(filledForm({ invoice_number: 'abc' }), false)).toEqual({})
    expect(invoiceFormErrors(filledForm({ invoice_number: '0' }), true).invoice_number).toBe('Un numero intero maggiore di zero.')
    expect(invoiceFormErrors(filledForm({ invoice_number: ' ' }), true)).toEqual({})
  })

  it('keys line errors by position', () => {
    const form = filledForm()
    form.lines.push(blankLine(1))
    expect(invoiceFormErrors(form, false)).toEqual({ 'line-1': 'Serve una descrizione.' })
  })
})

describe('lineError', () => {
  const line = { ...blankLine(0), description: 'Colloquio', unit_price: 80 }

  it('reports the first problem in field order', () => {
    expect(lineError({ ...line, quantity: 0, unit_price: -1 })).toBe('La quantità è un intero da 1 in su.')
    expect(lineError({ ...line, unit_price: Number.NaN })).toBe('Prezzo non valido.')
    expect(lineError({ ...line, vat_rate: 101 })).toBe('IVA tra 0 e 100.')
    expect(lineError({ ...line, amount_override: -5 })).toBe('Importo non valido.')
    expect(lineError(line)).toBeUndefined()
  })
})

describe('payloads', () => {
  it('drops the line keys and empty optional fields', () => {
    const input = createInvoiceInput(filledForm({ lines: [{ ...blankLine(4), description: 'Seduta', service_id: 0 }] }), 7)
    expect(input.lines).toEqual([{ service_id: undefined, description: 'Seduta', quantity: 1, unit_price: 0, vat_rate: 0, amount_override: null }])
    expect(input.due_date).toBeUndefined()
    expect(input.client_id).toBe(7)
  })

  it('sends the paid date only for a paid invoice and keeps the saved number when emptied', () => {
    const paid = updateInvoiceInput(filledForm({ status: 'paid', paid_date: '2026-03-12', invoice_number: ' ' }), 7, 3)
    expect(paid).toMatchObject({ id: 3, invoice_number: undefined, paid_date: '2026-03-12' })
    expect(updateInvoiceInput(filledForm({ status: 'issued', paid_date: '2026-03-12' }), 7, 3).paid_date).toBeUndefined()
  })
})

describe('formFromInvoice', () => {
  it('gives every line a fresh key and turns missing dates into empty fields', () => {
    let key = 10
    const invoice = {
      client_id: 2, invoice_number: '004', issue_date: '2026-02-01', due_date: undefined, status: 'issued', payment_method: 'pos',
      notes: '', apply_enpap: false, hide_quantity: true, paid_date: undefined,
      lines: [
        { service_id: 1, description: 'A', quantity: 1, unit_price: 80, vat_rate: 0, amount_override: undefined },
        { service_id: undefined, description: 'B', quantity: 2, unit_price: 50, vat_rate: 22, amount_override: 90 },
      ],
    } as unknown as Invoice
    const form = formFromInvoice(invoice, () => key++)
    expect(form.due_date).toBe('')
    expect(form.paid_date).toBe('')
    expect(form.lines.map((line) => [line.key, line.amount_override])).toEqual([[10, null], [11, 90]])
  })
})

describe('enpapDescription', () => {
  it('says whether the bollo enters the ENPAP base', () => {
    expect(enpapDescription({ tax_regime: 'forfettario', enpap_excludes_bollo: false })).toContain('imponibile e marca da bollo')
    expect(enpapDescription({ tax_regime: 'ordinario', enpap_excludes_bollo: false })).toContain('senza la marca da bollo')
  })
})

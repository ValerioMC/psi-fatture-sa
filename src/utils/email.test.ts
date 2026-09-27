import { describe, expect, it } from 'vitest'
import type { EmailAccount, EmailProviderPreset, InvoiceEmail } from '@/types'
import { describeMailMark, detectProvider, groupEmailsByInvoice, insertPlaceholder, invoiceEmailState, isEmailReady } from './email'

function email(overrides: Partial<InvoiceEmail>): InvoiceEmail {
  return {
    id: 1,
    invoice_id: 10,
    recipient: 'anna@example.it',
    subject: 'Fattura n. 12/2026',
    attachment_name: 'Fattura_12_2026.pdf',
    status: 'sent',
    error: null,
    sent_at: '2026-03-20 10:00:00',
    ...overrides,
  }
}

describe('invoiceEmailState', () => {
  it('is none without attempts', () => {
    expect(invoiceEmailState([])).toEqual({ kind: 'none', lastSent: null, last: null })
  })

  it('stays sent when a later resend fails', () => {
    const sent = email({ id: 1, sent_at: '2026-03-20 10:00:00' })
    const failed = email({ id: 2, status: 'failed', error: 'rete', sent_at: '2026-03-21 10:00:00' })
    const state = invoiceEmailState([sent, failed])
    expect(state.kind).toBe('sent')
    expect(state.lastSent).toBe(sent)
    expect(state.last).toBe(failed)
  })

  it('is failed when nothing ever went', () => {
    expect(invoiceEmailState([email({ status: 'failed' })]).kind).toBe('failed')
  })

  it('breaks ties on the same second by id', () => {
    const first = email({ id: 1 })
    const second = email({ id: 2 })
    expect(invoiceEmailState([first, second]).last).toBe(second)
  })
})

describe('groupEmailsByInvoice', () => {
  it('groups attempts per invoice', () => {
    const groups = groupEmailsByInvoice([email({ id: 1, invoice_id: 1 }), email({ id: 2, invoice_id: 2 }), email({ id: 3, invoice_id: 1 })])
    expect(groups.get(1)?.map((record) => record.id)).toEqual([1, 3])
    expect(groups.get(2)?.length).toBe(1)
  })
})

describe('isEmailReady', () => {
  const account = { saved: true } as EmailAccount
  it('needs a saved account and its password', () => {
    expect(isEmailReady(account, { password_configured: true })).toBe(true)
    expect(isEmailReady({ ...account, saved: false }, { password_configured: true })).toBe(false)
    expect(isEmailReady(account, { password_configured: false })).toBe(false)
    expect(isEmailReady(null, null)).toBe(false)
  })
})

describe('detectProvider', () => {
  const presets = [
    { provider: 'psypec', domains: ['psypec.it'] },
    { provider: 'gmail', domains: ['gmail.com'] },
    { provider: 'custom', domains: [] },
  ] as unknown as EmailProviderPreset[]
  it('matches the domain, case-insensitively', () => {
    expect(detectProvider(' Maria@PSYPEC.it ', presets)).toBe('psypec')
    expect(detectProvider('maria@gmail.com', presets)).toBe('gmail')
    expect(detectProvider('maria@studio.it', presets)).toBe('custom')
    expect(detectProvider('', presets)).toBe('custom')
  })
})

describe('insertPlaceholder', () => {
  it('replaces the selection and moves the caret after the token', () => {
    expect(insertPlaceholder('Gentile X,', 8, 9, 'paziente')).toEqual({ text: 'Gentile {paziente},', caret: 18 })
    expect(insertPlaceholder('', 0, 0, 'importo')).toEqual({ text: '{importo}', caret: 9 })
  })
})

describe('describeMailMark', () => {
  it('names every state', () => {
    expect(describeMailMark('sent')).toMatch(/Inviata/)
    expect(describeMailMark('failed')).toMatch(/non riuscito/)
    expect(describeMailMark('none')).toMatch(/Non ancora/)
  })
})

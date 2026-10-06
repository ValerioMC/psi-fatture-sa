import { describe, expect, it } from 'vitest'
import {
  hasVat,
  legalNotes,
  paymentCondition,
  printedClientName,
  printedPaymentLabel,
  professionalFullName,
} from './invoicePrint'

const PLAIN = { apply_enpap: false, contributo_enpap: 0, ritenuta_acconto: 0, total_tax: 0 }

describe('legalNotes', () => {
  it('says the VAT exemption on an ordinary-regime invoice without VAT', () => {
    expect(legalNotes(PLAIN, { tax_regime: 'ordinario' })).toEqual([
      "Operazione esente da IVA ai sensi dell'art. 10, n. 18, DPR 633/72.",
    ])
  })

  it('says nothing about exemption when the invoice carries VAT', () => {
    expect(legalNotes({ ...PLAIN, total_tax: 12 }, { tax_regime: 'ordinario' })).toEqual([])
  })

  it('cites the forfettario regime instead of the exemption', () => {
    const notes = legalNotes(PLAIN, { tax_regime: 'forfettario' })
    expect(notes).toHaveLength(1)
    expect(notes[0]).toContain('Regime Forfettario. Imposta non dovuta.')
  })

  it('puts forfettario and ENPAP in one paragraph', () => {
    const notes = legalNotes({ ...PLAIN, apply_enpap: true, contributo_enpap: 2 }, { tax_regime: 'forfettario' })
    expect(notes).toHaveLength(1)
    expect(notes[0]).toMatch(/Imposta non dovuta\. Contributo integrativo ENPAP 2% \(.*2,00.*\) addebitato/)
  })

  it('keeps ENPAP as its own paragraph before the exemption on the ordinary regime', () => {
    const notes = legalNotes({ ...PLAIN, apply_enpap: true, contributo_enpap: 2 }, { tax_regime: 'ordinario' })
    expect(notes).toHaveLength(2)
    expect(notes[0]).toContain('Contributo integrativo ENPAP')
    expect(notes[1]).toContain('esente da IVA')
  })

  it('omits ENPAP when it was not applied or amounts to nothing', () => {
    expect(legalNotes({ ...PLAIN, apply_enpap: true, contributo_enpap: 0, total_tax: 5 }, { tax_regime: 'ordinario' })).toEqual([])
    expect(legalNotes({ ...PLAIN, apply_enpap: false, contributo_enpap: 2, total_tax: 5 }, { tax_regime: 'ordinario' })).toEqual([])
  })

  it('asks for the withholding with its amount only when there is one', () => {
    const notes = legalNotes({ ...PLAIN, total_tax: 5, ritenuta_acconto: 14 }, { tax_regime: 'ordinario' })
    expect(notes).toHaveLength(1)
    expect(notes[0]).toMatch(/^Si richiede di operare una ritenuta d'acconto del 20% pari a .*14,00/)
  })
})

describe('payment text', () => {
  it('prints the long label for cards and states a condition for every settled method', () => {
    expect(printedPaymentLabel('pos')).toBe('POS / Carta di credito')
    expect(paymentCondition('bonifico')).toBe('Pagamento a vista')
    expect(paymentCondition('contanti')).toBe('Contestuale alla prestazione')
    expect(paymentCondition('altro')).toBeUndefined()
  })
})

describe('names', () => {
  it('joins the professional title and names, skipping the empty ones', () => {
    expect(professionalFullName({ title: 'Dott.ssa', first_name: 'Maria', last_name: 'Rossi' })).toBe('Dott.ssa Maria Rossi')
    expect(professionalFullName({ title: '', first_name: 'Maria', last_name: 'Rossi' })).toBe('Maria Rossi')
  })

  it('addresses a person first name first and a company by its name', () => {
    expect(printedClientName({ client_type: 'persona_fisica', first_name: 'Anna', last_name: 'Bianchi' })).toBe('Anna Bianchi')
    expect(printedClientName({ client_type: 'azienda', first_name: '', last_name: 'Studio Alfa' })).toBe('Studio Alfa')
  })

  it('sees VAT only when some tax was charged', () => {
    expect(hasVat({ total_tax: 0 })).toBe(false)
    expect(hasVat({ total_tax: 0.5 })).toBe(true)
  })
})

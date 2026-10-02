import { describe, expect, it } from 'vitest'
import { enpapIncludesBollo, estimateForfettarioTax, lineNetAmount, toLineInput } from './tax'

const FORFETTARIO = { tax_regime: 'forfettario', enpap_excludes_bollo: false }
const ORDINARIO = { tax_regime: 'ordinario', enpap_excludes_bollo: false }

describe('invoice line helpers', () => {
  it('puts the bollo in the ENPAP base only for an opted-in forfettario profile', () => {
    expect(enpapIncludesBollo(FORFETTARIO)).toBe(true)
    expect(enpapIncludesBollo({ tax_regime: 'forfettario', enpap_excludes_bollo: true })).toBe(false)
    expect(enpapIncludesBollo(ORDINARIO)).toBe(false)
  })

  it('keeps a hand-typed zero and ignores a null override', () => {
    expect(lineNetAmount({ quantity: 1, unit_price: 70, vat_rate: 0, amount_override: 0 })).toBe(0)
    expect(lineNetAmount({ quantity: 2, unit_price: 70, vat_rate: 0, amount_override: null })).toBe(140)
  })

  it('sends emptied fields as zero, never as NaN', () => {
    expect(toLineInput({ quantity: NaN, unit_price: 70, vat_rate: Number.NaN, amount_override: Number.NaN })).toEqual({
      description: '',
      quantity: 0,
      unit_price: 70,
      vat_rate: 0,
      amount_override: null,
    })
  })
})

describe('estimateForfettarioTax', () => {
  it('estimates taxes from annual revenue', () => {
    const estimate = estimateForfettarioTax(50_000, 78)
    expect(estimate.taxableIncome).toBe(39_000)
    expect(estimate.inpsContribution).toBe(10_167.3)
    expect(estimate.substituteTaxRate).toBe(15)
    expect(estimate.totalTax).toBe(estimate.inpsContribution + estimate.substituteTax)
  })

  it('uses the reduced 5% rate for the first five years', () => {
    expect(estimateForfettarioTax(50_000, 78, true).substituteTaxRate).toBe(5)
  })
})

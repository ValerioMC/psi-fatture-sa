import { describe, expect, it } from 'vitest'
import { calculateInvoiceTotals, enpapIncludesBollo, estimateForfettarioTax, lineNetAmount } from './tax'

const FORFETTARIO = { tax_regime: 'forfettario', enpap_excludes_bollo: false }
const ORDINARIO = { tax_regime: 'ordinario', enpap_excludes_bollo: false }

describe('calculateInvoiceTotals', () => {
  it('computes a forfettario invoice with ENPAP and marca da bollo', () => {
    const totals = calculateInvoiceTotals(
      [{ quantity: 4, unit_price: 70, vat_rate: 0 }],
      FORFETTARIO,
      true,
    )
    expect(totals.total_net).toBe(280)
    expect(totals.total_tax).toBe(0)
    expect(totals.contributo_enpap).toBe(5.64)
    expect(totals.ritenuta_acconto).toBe(0)
    expect(totals.marca_da_bollo).toBe(2)
    expect(totals.total_gross).toBe(285.64)
    expect(totals.total_due).toBe(287.64)
  })

  it('computes ENPAP on the compenso alone and ritenuta on net plus ENPAP in ordinario', () => {
    const totals = calculateInvoiceTotals(
      [{ quantity: 1, unit_price: 100, vat_rate: 0 }],
      ORDINARIO,
      true,
    )
    expect(totals.contributo_enpap).toBe(2)
    expect(totals.ritenuta_acconto).toBe(20.4)
    expect(totals.marca_da_bollo).toBe(2)
    expect(totals.total_due).toBe(83.6)
  })

  it('leaves the bollo out of ENPAP when a forfettario profile opts out', () => {
    const totals = calculateInvoiceTotals(
      [{ quantity: 4, unit_price: 70, vat_rate: 0 }],
      { tax_regime: 'forfettario', enpap_excludes_bollo: true },
      true,
    )
    expect(totals.contributo_enpap).toBe(5.6)
    expect(totals.total_due).toBe(287.6)
  })

  it('puts the bollo in the ENPAP base only for an opted-in forfettario profile', () => {
    expect(enpapIncludesBollo(FORFETTARIO)).toBe(true)
    expect(enpapIncludesBollo({ tax_regime: 'forfettario', enpap_excludes_bollo: true })).toBe(false)
    expect(enpapIncludesBollo(ORDINARIO)).toBe(false)
  })

  it('taxes a hand-typed line amount instead of quantity × price', () => {
    const totals = calculateInvoiceTotals(
      [{ quantity: 4, unit_price: 70, vat_rate: 0, amount_override: 250 }],
      FORFETTARIO,
      true,
    )
    expect(totals.total_net).toBe(250)
    expect(totals.contributo_enpap).toBe(5.04)
    expect(totals.total_due).toBe(257.04)
  })

  it('keeps a hand-typed zero and ignores a null override', () => {
    expect(lineNetAmount({ quantity: 1, unit_price: 70, vat_rate: 0, amount_override: 0 })).toBe(0)
    expect(lineNetAmount({ quantity: 2, unit_price: 70, vat_rate: 0, amount_override: null })).toBe(140)
  })

  it('skips marca da bollo below threshold or with VAT', () => {
    const below = calculateInvoiceTotals(
      [{ quantity: 1, unit_price: 77.47, vat_rate: 0 }],
      FORFETTARIO,
      false,
    )
    expect(below.marca_da_bollo).toBe(0)

    const withVat = calculateInvoiceTotals(
      [{ quantity: 1, unit_price: 100, vat_rate: 22 }],
      FORFETTARIO,
      false,
    )
    expect(withVat.marca_da_bollo).toBe(0)
  })

  it('rounds per line like the backend', () => {
    const totals = calculateInvoiceTotals(
      [{ quantity: 3, unit_price: 33.335, vat_rate: 0 }],
      FORFETTARIO,
      false,
    )
    expect(totals.total_net).toBe(100.01)
  })

  it('treats NaN inputs from emptied fields as zero', () => {
    const totals = calculateInvoiceTotals(
      [{ quantity: NaN, unit_price: 70, vat_rate: 0 }],
      FORFETTARIO,
      true,
    )
    expect(totals.total_due).toBe(0)
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

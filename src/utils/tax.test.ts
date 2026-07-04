import { describe, expect, it } from 'vitest'
import { calculateInvoiceTotals, estimateForfettarioTax } from './tax'

describe('calculateInvoiceTotals', () => {
  it('computes a forfettario invoice with ENPAP and marca da bollo', () => {
    const totals = calculateInvoiceTotals(
      [{ quantity: 4, unit_price: 70, vat_rate: 0 }],
      'forfettario',
      true,
    )
    expect(totals.total_net).toBe(280)
    expect(totals.total_tax).toBe(0)
    expect(totals.contributo_enpap).toBe(5.6)
    expect(totals.ritenuta_acconto).toBe(0)
    expect(totals.marca_da_bollo).toBe(2)
    expect(totals.total_gross).toBe(285.6)
    expect(totals.total_due).toBe(287.6)
  })

  it('applies ritenuta on net plus ENPAP in ordinario regime', () => {
    const totals = calculateInvoiceTotals(
      [{ quantity: 1, unit_price: 100, vat_rate: 0 }],
      'ordinario',
      true,
    )
    expect(totals.contributo_enpap).toBe(2)
    expect(totals.ritenuta_acconto).toBe(20.4)
    expect(totals.marca_da_bollo).toBe(2)
    expect(totals.total_due).toBe(100 + 2 - 20.4 + 2)
  })

  it('skips marca da bollo below threshold or with VAT', () => {
    const below = calculateInvoiceTotals(
      [{ quantity: 1, unit_price: 77.47, vat_rate: 0 }],
      'forfettario',
      false,
    )
    expect(below.marca_da_bollo).toBe(0)

    const withVat = calculateInvoiceTotals(
      [{ quantity: 1, unit_price: 100, vat_rate: 22 }],
      'forfettario',
      false,
    )
    expect(withVat.marca_da_bollo).toBe(0)
  })

  it('rounds per line like the backend', () => {
    const totals = calculateInvoiceTotals(
      [{ quantity: 3, unit_price: 33.335, vat_rate: 0 }],
      'forfettario',
      false,
    )
    expect(totals.total_net).toBe(100.01)
  })

  it('treats NaN inputs from emptied fields as zero', () => {
    const totals = calculateInvoiceTotals(
      [{ quantity: NaN, unit_price: 70, vat_rate: 0 }],
      'forfettario',
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

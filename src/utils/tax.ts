/**
 * Tax calculation utilities for Italian psychologist invoices.
 * Ported from the Python domain logic.
 */

export interface LineData {
  quantity: number
  unit_price: number
  vat_rate: number
}

export interface InvoiceTotals {
  total_net: number
  total_tax: number
  contributo_enpap: number
  ritenuta_acconto: number
  marca_da_bollo: number
  total_gross: number
  total_due: number
}

const ENPAP_RATE = 0.02
const RITENUTA_RATE = 0.2
const MARCA_DA_BOLLO_AMOUNT = 2.0
const MARCA_DA_BOLLO_THRESHOLD = 77.47

/**
 * Calculates all invoice totals including ENPAP, ritenuta d'acconto, and marca da bollo.
 *
 * For "forfettario" regime: no ritenuta d'acconto applies.
 * Marca da bollo (€2) applies when the invoice is VAT-exempt and total_net > €77.47.
 */
export function calculateInvoiceTotals(
  lines: LineData[],
  taxRegime: string,
  applyEnpap: boolean,
): InvoiceTotals {
  const total_net = lines.reduce(
    (sum, line) => sum + line.quantity * line.unit_price,
    0,
  )

  const total_tax = lines.reduce(
    (sum, line) => sum + line.quantity * line.unit_price * (line.vat_rate / 100),
    0,
  )

  const total_gross = total_net + total_tax

  const contributo_enpap = applyEnpap ? round2(total_net * ENPAP_RATE) : 0

  const ritenuta_base = total_net + contributo_enpap
  const ritenuta_acconto =
    taxRegime === 'ordinario' ? round2(ritenuta_base * RITENUTA_RATE) : 0

  const hasVat = total_tax > 0
  const appliesMarcaDaBollo =
    !hasVat && total_net > MARCA_DA_BOLLO_THRESHOLD
  const marca_da_bollo = appliesMarcaDaBollo ? MARCA_DA_BOLLO_AMOUNT : 0

  const total_due =
    total_gross + contributo_enpap - ritenuta_acconto + marca_da_bollo

  return {
    total_net: round2(total_net),
    total_tax: round2(total_tax),
    contributo_enpap,
    ritenuta_acconto,
    marca_da_bollo,
    total_gross: round2(total_gross),
    total_due: round2(total_due),
  }
}

function round2(value: number): number {
  return Math.round(value * 100) / 100
}

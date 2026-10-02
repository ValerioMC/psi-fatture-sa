/**
 * Tax helpers for Italian psychologist invoices. Invoice totals are computed
 * only by the backend (`preview_invoice_totals`); here live the per-line
 * display helpers and the annual tax estimates.
 */
import type { InvoiceLineInput } from '@/types'

export interface LineData {
  quantity: number
  unit_price: number
  vat_rate: number
  amount_override?: number | null
}

/** The profile settings that decide how an invoice is taxed. */
export interface TaxProfile {
  tax_regime: string
  enpap_excludes_bollo: boolean
}

/** The line's taxable amount: the hand-typed one when set, otherwise quantity × unit price. */
export function lineNetAmount(line: LineData): number {
  const amount = hasAmountOverride(line)
    ? line.amount_override
    : sanitize(line.quantity) * sanitize(line.unit_price)
  return round2(sanitize(amount))
}

export function hasAmountOverride(line: LineData): line is LineData & { amount_override: number } {
  return line.amount_override !== undefined && line.amount_override !== null
}

/**
 * Whether the bollo concurs to the ENPAP base: in the forfettario regime it is
 * compenso (Interpello AdE) unless the profile opts out; in ordinario never.
 */
export function enpapIncludesBollo(profile: TaxProfile): boolean {
  return profile.tax_regime === 'forfettario' && !profile.enpap_excludes_bollo
}

/**
 * A form line as the backend reads it: emptied number fields become zero, and an
 * emptied hand-typed amount falls back to quantity × price.
 */
export function toLineInput(line: LineData & { service_id?: number; description?: string }): InvoiceLineInput {
  return {
    service_id: line.service_id,
    description: line.description ?? '',
    quantity: Math.trunc(sanitize(line.quantity)),
    unit_price: sanitize(line.unit_price),
    vat_rate: sanitize(line.vat_rate),
    amount_override: hasAmountOverride(line) && Number.isFinite(line.amount_override) ? line.amount_override : null,
  }
}

/** Treats NaN (e.g. an emptied number input) as zero to keep previews stable. */
function sanitize(value: number): number {
  return Number.isFinite(value) ? value : 0
}

function round2(value: number): number {
  return Math.round(value * 100) / 100
}

// ── Annual tax estimation ──────────────────────────────────────────────────────

const INPS_GESTIONE_SEPARATA_RATE = 0.2607

export interface ForfettarioTaxEstimate {
  annualRevenue: number
  taxableIncome: number
  substituteTaxRate: number
  inpsContribution: number
  taxableBaseAfterInps: number
  substituteTax: number
  totalTax: number
  netIncome: number
}

/**
 * Estimates annual forfettario taxes from total net compensation.
 */
export function estimateForfettarioTax(
  annualRevenue: number,
  coefficient: number,
  firstFiveYears: boolean = false,
): ForfettarioTaxEstimate {
  const taxableIncome = round2(annualRevenue * coefficient / 100)
  const substituteTaxRate = firstFiveYears ? 5 : 15
  const inpsContribution = round2(taxableIncome * INPS_GESTIONE_SEPARATA_RATE)
  const taxableBaseAfterInps = round2(taxableIncome - inpsContribution)
  const substituteTax = round2(taxableBaseAfterInps * substituteTaxRate / 100)
  const totalTax = round2(inpsContribution + substituteTax)
  const netIncome = round2(annualRevenue - totalTax)

  return {
    annualRevenue,
    taxableIncome,
    substituteTaxRate,
    inpsContribution,
    taxableBaseAfterInps,
    substituteTax,
    totalTax,
    netIncome,
  }
}

export interface OrdinarioTaxEstimate {
  annualRevenue: number
  estimatedCosts: number
  taxableIncome: number
  inpsContribution: number
  taxableBaseAfterInps: number
  irpef: number
  addizionaleRegionale: number
  addizionaleComunale: number
  totalTax: number
  netIncome: number
}

/**
 * Estimates annual ordinario taxes from total net compensation.
 */
export function estimateOrdinarioTax(
  annualRevenue: number,
  coefficient: number,
): OrdinarioTaxEstimate {
  const estimatedCosts = round2(annualRevenue * (100 - coefficient) / 100)
  const taxableIncome = round2(annualRevenue * coefficient / 100)
  const inpsContribution = round2(taxableIncome * INPS_GESTIONE_SEPARATA_RATE)
  const taxableBaseAfterInps = round2(taxableIncome - inpsContribution)
  const irpef = calculateIrpef(taxableBaseAfterInps)
  const addizionaleRegionale = round2(taxableBaseAfterInps * 0.0173)
  const addizionaleComunale = round2(taxableBaseAfterInps * 0.008)
  const totalTax = round2(inpsContribution + irpef + addizionaleRegionale + addizionaleComunale)
  const netIncome = round2(annualRevenue - totalTax)

  return {
    annualRevenue,
    estimatedCosts,
    taxableIncome,
    inpsContribution,
    taxableBaseAfterInps,
    irpef,
    addizionaleRegionale,
    addizionaleComunale,
    totalTax,
    netIncome,
  }
}

const IRPEF_BRACKETS: readonly [number, number, number][] = [
  [0, 28_000, 0.23],
  [28_000, 50_000, 0.35],
  [50_000, Infinity, 0.43],
]

/**
 * Calculates progressive IRPEF tax using 2024 brackets.
 */
function calculateIrpef(taxableIncome: number): number {
  let tax = 0
  let remaining = taxableIncome

  for (const [lower, upper, rate] of IRPEF_BRACKETS) {
    if (remaining <= 0) break
    const bracketSize = Math.min(upper - lower, remaining)
    tax += round2(bracketSize * rate)
    remaining -= bracketSize
  }

  return round2(tax)
}

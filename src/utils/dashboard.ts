/** The dashboard's own figures, computed from what the backend returns. */
import type { DashboardData, MonthlyInvoicePreview } from '@/types'
import { estimateForfettarioTax, estimateOrdinarioTax } from '@/utils/tax'

/** Sessions of one month that were held but never invoiced, and what they would bill. */
export interface UnbilledMonth {
  month: number
  year: number
  sessions: number
  amount: number
}

export interface TaxEstimateSummary {
  revenue: number
  net: number
  contributions: number
  tax: number
  taxLabel: string
}

/** The month before, `month` counted from 1; January steps back into December of last year. */
export function previousMonth(year: number, month: number): { year: number; month: number } {
  return month === 1 ? { year: year - 1, month: 12 } : { year, month: month - 1 }
}

/** Null when every session of the month is invoiced. */
export function unbilledMonth(previews: readonly MonthlyInvoicePreview[], year: number, month: number): UnbilledMonth | null {
  const sessions = previews.reduce((sum, preview) => sum + preview.appointment_count, 0)
  if (sessions === 0) return null
  const amount = previews.reduce((sum, preview) => sum + preview.estimated_due, 0)
  return { month, year, sessions, amount }
}

/** Percentage of the invoiced amount already paid, rounded. */
export function collectionRate(data: DashboardData | null): number {
  if (!data || data.total_revenue === 0) return 0
  return Math.round((data.paid_revenue / data.total_revenue) * 100)
}

/** The year's estimate in the regime's terms; null before anything is invoiced. */
export function taxEstimate(revenue: number, coefficient: number, forfettario: boolean, firstFiveYears: boolean): TaxEstimateSummary | null {
  if (revenue <= 0) return null
  if (forfettario) {
    const estimate = estimateForfettarioTax(revenue, coefficient, firstFiveYears)
    return { revenue, net: estimate.netIncome, contributions: estimate.inpsContribution, tax: estimate.substituteTax, taxLabel: `Imposta sostitutiva ${estimate.substituteTaxRate}%` }
  }
  const estimate = estimateOrdinarioTax(revenue, coefficient)
  return {
    revenue,
    net: estimate.netIncome,
    contributions: estimate.inpsContribution,
    tax: estimate.irpef + estimate.addizionaleRegionale + estimate.addizionaleComunale,
    taxLabel: 'IRPEF e addizionali',
  }
}

export function greeting(hour: number, firstName: string | undefined): string {
  const salutation = hour < 13 ? 'Buongiorno' : hour < 18 ? 'Buon pomeriggio' : 'Buonasera'
  return firstName ? `${salutation}, ${firstName}` : salutation
}

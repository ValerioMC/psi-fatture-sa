/** Formatting utilities for currency, dates, and other display values. */

const ITALIAN_MONTHS = [
  'gennaio', 'febbraio', 'marzo', 'aprile', 'maggio', 'giugno',
  'luglio', 'agosto', 'settembre', 'ottobre', 'novembre', 'dicembre',
]

/**
 * Formats a number as Italian currency string.
 * @example formatCurrency(1234.56) → "€ 1.234,56"
 */
export function formatCurrency(value: number): string {
  return new Intl.NumberFormat('it-IT', {
    style: 'currency',
    currency: 'EUR',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value)
}

/**
 * Formats an ISO date string as dd/MM/yyyy.
 * @example formatDate("2026-03-20") → "20/03/2026"
 */
export function formatDate(dateStr: string): string {
  if (!dateStr) return ''
  const [year, month, day] = dateStr.split('T')[0].split('-')
  return `${day}/${month}/${year}`
}

/**
 * Formats an ISO date string as "dd mese yyyy" in Italian.
 * @example formatDateLong("2026-03-20") → "20 marzo 2026"
 */
export function formatDateLong(dateStr: string): string {
  if (!dateStr) return ''
  const [year, month, day] = dateStr.split('T')[0].split('-')
  const monthName = ITALIAN_MONTHS[parseInt(month, 10) - 1] ?? ''
  return `${parseInt(day, 10)} ${monthName} ${year}`
}

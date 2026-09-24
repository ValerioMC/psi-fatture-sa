/** Formatting utilities for currency, dates, and other display values. */

export const ITALIAN_MONTHS = [
  'gennaio', 'febbraio', 'marzo', 'aprile', 'maggio', 'giugno',
  'luglio', 'agosto', 'settembre', 'ottobre', 'novembre', 'dicembre',
] as const

const currencyFormatter = new Intl.NumberFormat('it-IT', {
  style: 'currency',
  currency: 'EUR',
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
  // it-IT leaves four-digit amounts ungrouped ("2583,10"); a column of money must group consistently.
  useGrouping: 'always',
})

const compactFormatter = new Intl.NumberFormat('it-IT', {
  maximumFractionDigits: 1,
})

/**
 * Formats a number as Italian currency string.
 * @example formatCurrency(1234.56) → "1.234,56 €"
 */
export function formatCurrency(value: number): string {
  return currencyFormatter.format(value)
}

/**
 * Formats a whole-euro amount for chart axes and meters, where cents are noise.
 * @example formatCurrencyCompact(12500) → "12,5k €"; formatCurrencyCompact(800) → "800 €"
 */
export function formatCurrencyCompact(value: number): string {
  if (Math.abs(value) >= 1000) return `${compactFormatter.format(value / 1000)}k €`
  return `${Math.round(value)} €`
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

/**
 * Formats an ISO date as a short day and month, for dense lists.
 * @example formatDateShort("2026-03-20") → "20 mar"
 */
export function formatDateShort(dateStr: string): string {
  if (!dateStr) return ''
  const [, month, day] = dateStr.split('T')[0].split('-')
  const monthName = ITALIAN_MONTHS[parseInt(month, 10) - 1] ?? ''
  return `${parseInt(day, 10)} ${monthName.slice(0, 3)}`
}

/** "Settembre 2026" */
export function formatMonthYear(year: number, month1to12: number): string {
  const name = ITALIAN_MONTHS[month1to12 - 1] ?? ''
  return `${name.charAt(0).toUpperCase()}${name.slice(1)} ${year}`
}

/**
 * The calendar day of a Date in local time as `YYYY-MM-DD`.
 *
 * `Date#toISOString` converts to UTC first, which in Italy turns local
 * midnight into the previous day; every "today" and every generated date
 * must go through this instead.
 */
export function toIsoDate(date: Date): string {
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${date.getFullYear()}-${month}-${day}`
}

export function todayIso(): string {
  return toIsoDate(new Date())
}

/** Parses `YYYY-MM-DD` as local midnight (not UTC, see toIsoDate). */
export function parseIsoDate(isoDate: string): Date {
  const [year, month, day] = isoDate.slice(0, 10).split('-').map(Number)
  return new Date(year, month - 1, day)
}

/** Minutes between two `HH:MM` times on the same day; 0 when the range is empty or inverted. */
export function minutesBetween(start: string, end: string): number {
  const [startHour, startMinute] = start.split(':').map(Number)
  const [endHour, endMinute] = end.split(':').map(Number)
  const minutes = endHour * 60 + endMinute - (startHour * 60 + startMinute)
  return minutes > 0 ? minutes : 0
}

export interface CurrencyParts {
  /** "62.340" */
  whole: string
  /** ",00" */
  fraction: string
  /** "€" */
  symbol: string
}

/**
 * Splits a currency value so a hero figure can set the cents smaller than the euros.
 * @example splitCurrency(62340) → { whole: "62.340", fraction: ",00", symbol: "€" }
 */
export function splitCurrency(value: number): CurrencyParts {
  const parts = currencyFormatter.formatToParts(value)
  const pick = (types: string[]): string => parts.filter((part) => types.includes(part.type)).map((part) => part.value).join('')
  return {
    whole: pick(['minusSign', 'integer', 'group']),
    fraction: pick(['decimal', 'fraction']),
    symbol: pick(['currency']),
  }
}

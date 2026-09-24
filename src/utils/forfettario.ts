/**
 * The forfettario revenue ceiling. Above €85.000 of compensi in a year the
 * regime ends (from the following year; immediately above €100.000), so a
 * psychologist on forfettario watches this number more than any other.
 */

export const FORFETTARIO_THRESHOLD = 85_000
export const FORFETTARIO_HARD_THRESHOLD = 100_000

/** Share of the threshold at which the meter starts warning. */
const NEAR_RATIO = 0.85

export type ThresholdLevel = 'ok' | 'near' | 'over'

export interface ThresholdReading {
  /** Compensi so far in the year. */
  amount: number
  /** Linear projection to 31 December; null for a year that is already over or not started. */
  projection: number | null
  level: ThresholdLevel
  /** amount / threshold, not clamped: 1.1 means 10% over. */
  ratio: number
}

function dayOfYear(date: Date): number {
  const start = Date.UTC(date.getFullYear(), 0, 1)
  const now = Date.UTC(date.getFullYear(), date.getMonth(), date.getDate())
  return Math.floor((now - start) / 86_400_000) + 1
}

function daysInYear(year: number): number {
  return (Date.UTC(year + 1, 0, 1) - Date.UTC(year, 0, 1)) / 86_400_000
}

/**
 * Projects year-end compensi from the pace so far. Only meaningful for the
 * current year and only after the first month, when a run-rate means something.
 */
export function projectYearEnd(amountSoFar: number, year: number, today: Date = new Date()): number | null {
  if (year !== today.getFullYear()) return null
  const elapsedDays = dayOfYear(today)
  if (elapsedDays < 31 || amountSoFar <= 0) return null
  return Math.round((amountSoFar / elapsedDays) * daysInYear(year))
}

/** Reads the meter: where the year stands and where it is heading. */
export function readThreshold(amountSoFar: number, year: number, today: Date = new Date()): ThresholdReading {
  const projection = projectYearEnd(amountSoFar, year, today)
  const ratio = amountSoFar / FORFETTARIO_THRESHOLD
  const heading = projection ?? amountSoFar
  let level: ThresholdLevel = 'ok'
  if (amountSoFar > FORFETTARIO_THRESHOLD) level = 'over'
  else if (heading > FORFETTARIO_THRESHOLD || ratio >= NEAR_RATIO) level = 'near'
  return { amount: amountSoFar, projection, level, ratio }
}

/** Dates for a weekly recurring appointment. */
import { parseIsoDate, toIsoDate } from './format'

/**
 * Every date on the chosen weekdays (0 = Sunday … 6 = Saturday, as in
 * `Date#getDay`) within `weeks` weeks from `startIso`, the start day
 * included, sorted. Works on local calendar days, so no date shifts across
 * midnight in the Italian time zone.
 */
export function weeklyDates(startIso: string, weekdays: readonly number[], weeks: number): string[] {
  if (weekdays.length === 0 || weeks < 1) return []
  const start = parseIsoDate(startIso)
  const dates = new Set<string>()
  for (let week = 0; week < weeks; week++) {
    for (const weekday of weekdays) {
      const offset = ((weekday - start.getDay() + 7) % 7) + week * 7
      dates.add(toIsoDate(new Date(start.getFullYear(), start.getMonth(), start.getDate() + offset)))
    }
  }
  return [...dates].sort()
}

/**
 * Patient monograms: two initials on a muted tint that is stable for a name,
 * so the same person always wears the same colour across the app.
 */

export const MONOGRAM_TINT_COUNT = 6

/** Up to two initials from a display name ("Rossi Maria" → "RM", "Studio Blu" → "SB"). */
export function initialsOf(name: string): string {
  const words = name.trim().split(/\s+/).filter((word) => word.length > 0)
  if (words.length === 0) return '?'
  const first = words[0].charAt(0)
  const second = words.length > 1 ? words[1].charAt(0) : words[0].charAt(1)
  return `${first}${second}`.toLocaleUpperCase('it-IT')
}

/**
 * A tint index in 1..MONOGRAM_TINT_COUNT derived from the name (FNV-1a), not
 * from a database id, so a patient keeps their colour across exports and seeds.
 */
export function tintIndexOf(name: string): number {
  let hash = 0x811c9dc5
  for (const char of name.trim().toLocaleLowerCase('it-IT')) {
    hash ^= char.codePointAt(0) ?? 0
    hash = Math.imul(hash, 0x01000193)
  }
  return ((hash >>> 0) % MONOGRAM_TINT_COUNT) + 1
}

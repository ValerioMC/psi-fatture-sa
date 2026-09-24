/**
 * Appearance preference. "system" follows macOS/Windows; the explicit choices
 * pin the app regardless of the OS. Stored per machine in localStorage, which
 * is enough: the preference belongs to the device, not to the practice data.
 */

export type ThemePreference = 'system' | 'light' | 'dark'
export type ResolvedTheme = 'light' | 'dark'

export const THEME_STORAGE_KEY = 'psi-fatture.theme'

export function isThemePreference(value: unknown): value is ThemePreference {
  return value === 'system' || value === 'light' || value === 'dark'
}

export function resolveTheme(preference: ThemePreference, systemPrefersDark: boolean): ResolvedTheme {
  if (preference === 'system') return systemPrefersDark ? 'dark' : 'light'
  return preference
}

/** Reads the stored preference; storage can be missing or throw, which means "system". */
export function readThemePreference(storage: Pick<Storage, 'getItem'> | undefined): ThemePreference {
  try {
    const stored = storage?.getItem(THEME_STORAGE_KEY)
    return isThemePreference(stored) ? stored : 'system'
  } catch {
    return 'system'
  }
}

/** Persists the preference; returns false when storage refused the write. */
export function writeThemePreference(storage: Pick<Storage, 'setItem'> | undefined, preference: ThemePreference): boolean {
  try {
    storage?.setItem(THEME_STORAGE_KEY, preference)
    return storage !== undefined
  } catch {
    return false
  }
}

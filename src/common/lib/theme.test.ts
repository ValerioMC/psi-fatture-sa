import { describe, expect, it } from 'vitest'
import { readThemePreference, resolveTheme, writeThemePreference, THEME_STORAGE_KEY } from './theme'

describe('theme preference', () => {
  it('follows the system only when asked to', () => {
    expect(resolveTheme('system', true)).toBe('dark')
    expect(resolveTheme('system', false)).toBe('light')
    expect(resolveTheme('light', true)).toBe('light')
  })

  it('falls back to system on a missing, invalid or throwing storage', () => {
    expect(readThemePreference(undefined)).toBe('system')
    expect(readThemePreference({ getItem: () => 'purple' })).toBe('system')
    expect(readThemePreference({ getItem: () => { throw new Error('blocked') } })).toBe('system')
    expect(readThemePreference({ getItem: (key) => (key === THEME_STORAGE_KEY ? 'dark' : null) })).toBe('dark')
  })

  it('reports whether the preference was stored', () => {
    expect(writeThemePreference({ setItem: () => undefined }, 'dark')).toBe(true)
    expect(writeThemePreference({ setItem: () => { throw new Error('quota') } }, 'dark')).toBe(false)
  })
})

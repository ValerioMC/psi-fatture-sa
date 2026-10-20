import { describe, expect, it } from 'vitest'
import { hasPrimaryModifier, shortcutLabel } from './platform'

describe('platform shortcuts', () => {
  const mac = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0)'
  const windows = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'

  it('labels the primary modifier per platform', () => {
    expect(shortcutLabel('K', mac)).toBe('⌘K')
    expect(shortcutLabel('K', windows)).toBe('Ctrl K')
  })

  it('accepts only that modifier', () => {
    expect(hasPrimaryModifier({ metaKey: true, ctrlKey: false }, mac)).toBe(true)
    expect(hasPrimaryModifier({ metaKey: false, ctrlKey: true }, mac)).toBe(false)
    expect(hasPrimaryModifier({ metaKey: false, ctrlKey: true }, windows)).toBe(true)
  })
})

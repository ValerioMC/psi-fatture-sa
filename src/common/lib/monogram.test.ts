import { describe, expect, it } from 'vitest'
import { initialsOf, tintIndexOf, MONOGRAM_TINT_COUNT } from './monogram'

describe('monogram', () => {
  it('takes the first letter of the first two words', () => {
    expect(initialsOf('Rossi Maria')).toBe('RM')
    expect(initialsOf("  D'Angelo   Serena ")).toBe('DS')
  })

  it('uses two letters of a single word and a placeholder for nothing', () => {
    expect(initialsOf('Hera')).toBe('HE')
    expect(initialsOf('')).toBe('?')
  })

  it('gives the same name the same tint, whatever the case', () => {
    expect(tintIndexOf('Rossi Maria')).toBe(tintIndexOf('rossi maria'))
    const tint = tintIndexOf('Bianchi Luca')
    expect(tint).toBeGreaterThanOrEqual(1)
    expect(tint).toBeLessThanOrEqual(MONOGRAM_TINT_COUNT)
  })
})

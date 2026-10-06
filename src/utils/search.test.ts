import { describe, expect, it } from 'vitest'
import { searchKey } from './search'

describe('searchKey', () => {
  it('ignores case and accents so "Niccolò" is found by "niccolo"', () => {
    expect(searchKey('Niccolò')).toBe(searchKey('NICCOLO'))
    expect(searchKey('Niccolò')).toContain('niccolo')
  })

  it('keeps digits and punctuation, so invoice numbers still match', () => {
    expect(searchKey('FT-007/2026')).toBe('ft-007/2026')
  })
})

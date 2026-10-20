import { describe, expect, it } from 'vitest'
import { plural } from './labels'

describe('plural', () => {
  it('agrees the noun with the count', () => {
    expect(plural(1, 'fattura', 'fatture')).toBe('1 fattura')
    expect(plural(0, 'fattura', 'fatture')).toBe('0 fatture')
  })
})

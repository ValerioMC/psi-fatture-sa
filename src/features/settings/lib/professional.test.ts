import { describe, expect, it } from 'vitest'
import { professionalFullName } from './professional'

describe('professionalFullName', () => {
  it('joins the professional title and names, skipping the empty ones', () => {
    expect(professionalFullName({ title: 'Dott.ssa', first_name: 'Maria', last_name: 'Rossi' })).toBe('Dott.ssa Maria Rossi')
    expect(professionalFullName({ title: '', first_name: 'Maria', last_name: 'Rossi' })).toBe('Maria Rossi')
  })
})

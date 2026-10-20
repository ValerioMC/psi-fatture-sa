import { describe, expect, it } from 'vitest'
import { ageOn, clientDisplayName } from './client'

describe('client presentation', () => {
  it('puts the surname first for people and the business name for companies', () => {
    expect(clientDisplayName({ client_type: 'persona_fisica', first_name: 'Maria', last_name: 'Rossi' })).toBe('Rossi Maria')
    expect(clientDisplayName({ client_type: 'azienda', first_name: '', last_name: 'Studio Hera' })).toBe('Studio Hera')
  })

  it('computes age on a given day, before and after the birthday', () => {
    expect(ageOn('1990-09-25', new Date(2026, 8, 24))).toBe(35)
    expect(ageOn('1990-09-24', new Date(2026, 8, 24))).toBe(36)
    expect(ageOn(undefined)).toBeNull()
  })
})

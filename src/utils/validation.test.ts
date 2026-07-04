import { describe, expect, it } from 'vitest'
import {
  validateCap,
  validateCodiceFiscale,
  validateEmail,
  validateIban,
  validatePartitaIva,
  validateProvincia,
} from './validation'

describe('validateCodiceFiscale', () => {
  it('accepts a valid personal codice fiscale', () => {
    expect(validateCodiceFiscale('RSSMRA80A01H501U').valid).toBe(true)
  })

  it('is case-insensitive and trims whitespace', () => {
    expect(validateCodiceFiscale(' rssmra80a01h501u ').valid).toBe(true)
  })

  it('rejects a wrong check character', () => {
    const result = validateCodiceFiscale('RSSMRA80A01H501X')
    expect(result.valid).toBe(false)
    expect(result.message).toContain('controllo')
  })

  it('rejects wrong lengths', () => {
    expect(validateCodiceFiscale('RSSMRA80A01H501').valid).toBe(false)
    expect(validateCodiceFiscale('RSSMRA80A01H501UU').valid).toBe(false)
  })

  it('accepts an 11-digit entity code with valid checksum', () => {
    expect(validateCodiceFiscale('00743110157').valid).toBe(true)
  })

  it('accepts empty values (optional field)', () => {
    expect(validateCodiceFiscale('').valid).toBe(true)
  })
})

describe('validatePartitaIva', () => {
  it('accepts a valid partita IVA', () => {
    expect(validatePartitaIva('00743110157').valid).toBe(true)
  })

  it('rejects a wrong check digit', () => {
    expect(validatePartitaIva('00743110158').valid).toBe(false)
  })

  it('rejects non-numeric or wrong-length values', () => {
    expect(validatePartitaIva('0074311015').valid).toBe(false)
    expect(validatePartitaIva('0074311015A').valid).toBe(false)
  })

  it('accepts empty values (optional field)', () => {
    expect(validatePartitaIva('').valid).toBe(true)
  })
})

describe('validateIban', () => {
  it('accepts a valid Italian IBAN', () => {
    expect(validateIban('IT60X0542811101000000123456').valid).toBe(true)
  })

  it('accepts IBANs written with spaces', () => {
    expect(validateIban('IT60 X054 2811 1010 0000 0123 456').valid).toBe(true)
  })

  it('rejects an IBAN with a wrong checksum', () => {
    expect(validateIban('IT61X0542811101000000123456').valid).toBe(false)
  })

  it('rejects Italian IBANs of wrong length', () => {
    expect(validateIban('IT60X054281110100000012345').valid).toBe(false)
  })
})

describe('address and contact validators', () => {
  it('validates CAP', () => {
    expect(validateCap('00100').valid).toBe(true)
    expect(validateCap('0010').valid).toBe(false)
    expect(validateCap('0010A').valid).toBe(false)
  })

  it('validates provincia', () => {
    expect(validateProvincia('RM').valid).toBe(true)
    expect(validateProvincia('rm').valid).toBe(true)
    expect(validateProvincia('R1').valid).toBe(false)
  })

  it('validates email', () => {
    expect(validateEmail('mario.rossi@email.it').valid).toBe(true)
    expect(validateEmail('mario.rossi@email').valid).toBe(false)
    expect(validateEmail('@email.it').valid).toBe(false)
  })
})

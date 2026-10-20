/**
 * Italian fiscal data validation: codice fiscale, partita IVA, IBAN, CAP,
 * provincia, email. Every validator returns a ValidationResult so forms can
 * show a specific message next to the offending field.
 */

export interface ValidationResult {
  valid: boolean
  message?: string
}

const OK: ValidationResult = { valid: true }

function fail(message: string): ValidationResult {
  return { valid: false, message }
}

// ─── Codice fiscale ───────────────────────────────────────────────────────────

const CF_PATTERN = /^[A-Z]{6}\d{2}[A-Z]\d{2}[A-Z]\d{3}[A-Z]$/
const CF_ODD_VALUES: Record<string, number> = {
  '0': 1, '1': 0, '2': 5, '3': 7, '4': 9, '5': 13, '6': 15, '7': 17, '8': 19, '9': 21,
  A: 1, B: 0, C: 5, D: 7, E: 9, F: 13, G: 15, H: 17, I: 19, J: 21,
  K: 2, L: 4, M: 18, N: 20, O: 11, P: 3, Q: 6, R: 8, S: 12, T: 14,
  U: 16, V: 10, W: 22, X: 25, Y: 24, Z: 23,
}

function cfEvenValue(char: string): number {
  return char >= '0' && char <= '9'
    ? char.charCodeAt(0) - '0'.charCodeAt(0)
    : char.charCodeAt(0) - 'A'.charCodeAt(0)
}

/**
 * Validates an Italian codice fiscale.
 *
 * Accepts the 16-character personal format (with checksum verification,
 * including omocodia variants) and the 11-digit format used by legal entities.
 */
export function validateCodiceFiscale(value: string): ValidationResult {
  const cf = value.trim().toUpperCase()
  if (!cf) return OK

  if (/^\d{11}$/.test(cf)) return validatePartitaIva(cf)

  if (cf.length !== 16) {
    return fail('Il codice fiscale deve avere 16 caratteri')
  }

  // Omocodia can replace digits with letters, so checksum is the real test.
  const normalized = cf
  if (!/^[A-Z0-9]{16}$/.test(normalized)) {
    return fail('Il codice fiscale contiene caratteri non validi')
  }
  if (!CF_PATTERN.test(deOmocodia(normalized))) {
    return fail('Formato codice fiscale non valido')
  }

  let sum = 0
  for (let i = 0; i < 15; i++) {
    const char = normalized[i]
    sum += i % 2 === 0 ? CF_ODD_VALUES[char] : cfEvenValue(char)
  }
  const expected = String.fromCharCode('A'.charCodeAt(0) + (sum % 26))
  if (normalized[15] !== expected) {
    return fail('Codice fiscale non valido (carattere di controllo errato)')
  }
  return OK
}

const OMOCODIA_MAP: Record<string, string> = {
  L: '0', M: '1', N: '2', P: '3', Q: '4', R: '5', S: '6', T: '7', U: '8', V: '9',
}

/** Restores omocodia-substituted letters to digits in the numeric positions. */
function deOmocodia(cf: string): string {
  const numericPositions = [6, 7, 9, 10, 12, 13, 14]
  const chars = cf.split('')
  for (const pos of numericPositions) {
    const replacement = OMOCODIA_MAP[chars[pos]]
    if (replacement !== undefined) chars[pos] = replacement
  }
  return chars.join('')
}

// ─── Partita IVA ──────────────────────────────────────────────────────────────

/**
 * Validates an Italian partita IVA (11 digits, Luhn-style checksum).
 */
export function validatePartitaIva(value: string): ValidationResult {
  const piva = value.trim()
  if (!piva) return OK

  if (!/^\d{11}$/.test(piva)) {
    return fail('La partita IVA deve essere composta da 11 cifre')
  }

  let sum = 0
  for (let i = 0; i < 11; i++) {
    let digit = Number(piva[i])
    if (i % 2 === 1) {
      digit *= 2
      if (digit > 9) digit -= 9
    }
    sum += digit
  }
  if (sum % 10 !== 0) {
    return fail('Partita IVA non valida (cifra di controllo errata)')
  }
  return OK
}

// ─── IBAN ─────────────────────────────────────────────────────────────────────

/**
 * Validates an IBAN with the standard mod-97 check.
 * Italian IBANs must additionally be 27 characters long.
 */
export function validateIban(value: string): ValidationResult {
  const iban = value.replace(/\s+/g, '').toUpperCase()
  if (!iban) return OK

  if (!/^[A-Z]{2}\d{2}[A-Z0-9]+$/.test(iban)) {
    return fail('Formato IBAN non valido')
  }
  if (iban.startsWith('IT') && iban.length !== 27) {
    return fail("L'IBAN italiano deve avere 27 caratteri")
  }

  const rearranged = iban.slice(4) + iban.slice(0, 4)
  let remainder = 0
  for (const char of rearranged) {
    const digits = char >= 'A' ? String(char.charCodeAt(0) - 55) : char
    for (const d of digits) {
      remainder = (remainder * 10 + Number(d)) % 97
    }
  }
  if (remainder !== 1) {
    return fail('IBAN non valido (checksum errato)')
  }
  return OK
}

// ─── Address and contact fields ───────────────────────────────────────────────

/** Validates an Italian CAP (5 digits). */
export function validateCap(value: string): ValidationResult {
  const cap = value.trim()
  if (!cap) return OK
  return /^\d{5}$/.test(cap) ? OK : fail('Il CAP deve essere di 5 cifre')
}

/** Validates an Italian province code (2 letters). */
export function validateProvincia(value: string): ValidationResult {
  const prov = value.trim().toUpperCase()
  if (!prov) return OK
  return /^[A-Z]{2}$/.test(prov) ? OK : fail('La provincia deve essere di 2 lettere (es. RM)')
}

/** Validates an email address. */
export function validateEmail(value: string): ValidationResult {
  const email = value.trim()
  if (!email) return OK
  return /^[^\s@]+@[^\s@]+\.[^\s@]{2,}$/.test(email)
    ? OK
    : fail('Indirizzo email non valido')
}

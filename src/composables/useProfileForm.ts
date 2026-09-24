import { reactive } from 'vue'
import type { UpsertConfigInput } from '@/types'
import {
  validateCap,
  validateCodiceFiscale,
  validateEmail,
  validateIban,
  validatePartitaIva,
  validateProvincia,
  type ValidationResult,
} from '@/utils/validation'

/** The groups of the professional profile; onboarding asks them in steps, settings shows them all. */
export type ProfileSection = 'identity' | 'profession' | 'tax' | 'numbering' | 'studio' | 'payment'

export type ProfileField =
  | 'first_name' | 'last_name' | 'vat_number' | 'fiscal_code' | 'coefficient'
  | 'initial_invoice_number' | 'address' | 'city' | 'province' | 'zip_code' | 'pec_email' | 'iban'

export const SECTION_FIELDS: Readonly<Record<ProfileSection, readonly ProfileField[]>> = {
  identity: ['first_name', 'last_name', 'vat_number', 'fiscal_code'],
  profession: [],
  tax: ['coefficient'],
  numbering: ['initial_invoice_number'],
  studio: ['address', 'city', 'province', 'zip_code'],
  payment: ['pec_email', 'iban'],
}

export function emptyProfile(): UpsertConfigInput {
  return {
    title: 'Dott.ssa',
    first_name: '',
    last_name: '',
    vat_number: '',
    fiscal_code: '',
    tax_regime: 'forfettario',
    albo_number: '',
    albo_region: '',
    address: '',
    city: '',
    province: '',
    zip_code: '',
    country: 'IT',
    phone: '',
    pec_email: '',
    iban: '',
    coefficient: 78,
    profession: 'psicologo',
    is_psicoanalista: false,
    initial_invoice_number: 1,
  }
}

function required(value: string, message: string): ValidationResult {
  return value.trim() !== '' ? { valid: true } : { valid: false, message }
}

/** The format validators accept an empty value; a required field checks presence first. */
function requiredThen(value: string, message: string, rule: (value: string) => ValidationResult): ValidationResult {
  const presence = required(value, message)
  return presence.valid ? rule(value) : presence
}

/**
 * The professional profile as a form: the values, the per-field errors and
 * the rules. Everything printed on an invoice header is required; PEC and
 * IBAN are optional but must be well-formed when given.
 */
export function useProfileForm(initial: UpsertConfigInput = emptyProfile()) {
  const form = reactive<UpsertConfigInput>({ ...initial })
  const errors = reactive<Partial<Record<ProfileField, string>>>({})

  const RULES: Record<ProfileField, () => ValidationResult> = {
    first_name: () => required(form.first_name, 'Serve il nome.'),
    last_name: () => required(form.last_name, 'Serve il cognome.'),
    vat_number: () => requiredThen(form.vat_number, 'Serve la partita IVA.', validatePartitaIva),
    fiscal_code: () => requiredThen(form.fiscal_code, 'Serve il codice fiscale.', validateCodiceFiscale),
    coefficient: () =>
      Number.isFinite(form.coefficient) && form.coefficient > 0 && form.coefficient <= 100
        ? { valid: true }
        : { valid: false, message: 'Un valore tra 1 e 100.' },
    initial_invoice_number: () =>
      Number.isInteger(form.initial_invoice_number) && form.initial_invoice_number >= 1
        ? { valid: true }
        : { valid: false, message: 'Un numero intero da 1 in su.' },
    address: () => required(form.address, 'Serve l’indirizzo dello studio.'),
    city: () => required(form.city, 'Serve la città.'),
    province: () => requiredThen(form.province, 'Serve la provincia.', validateProvincia),
    zip_code: () => requiredThen(form.zip_code, 'Serve il CAP.', validateCap),
    pec_email: () => validateEmail(form.pec_email),
    iban: () => validateIban(form.iban),
  }

  function check(field: ProfileField): void {
    const result = RULES[field]()
    if (result.valid) delete errors[field]
    else errors[field] = result.message
  }

  /** Validates the given sections (all by default); true when none of their fields has an error. */
  function checkSections(sections: readonly ProfileSection[] = Object.keys(SECTION_FIELDS) as ProfileSection[]): boolean {
    const fields = sections.flatMap((section) => SECTION_FIELDS[section])
    fields.forEach(check)
    return fields.every((field) => errors[field] === undefined)
  }

  return { form, errors, check, checkSections }
}

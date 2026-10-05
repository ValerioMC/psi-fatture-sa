/**
 * The patient form's data: its blank and loaded values, the check of each field,
 * and the record card shown beside it. A company needs no first name but a P.IVA.
 */
import type { SegmentOption } from '@/components/ui/types'
import type { Client, ClientType, CreateClientInput, Invoice } from '@/types'
import { ageOn } from '@/utils/client'
import {
  validateCap,
  validateCodiceFiscale,
  validateEmail,
  validatePartitaIva,
  validateProvincia,
  type ValidationResult,
} from '@/utils/validation'

export type ClientField = 'first_name' | 'last_name' | 'fiscal_code' | 'vat_number' | 'email' | 'zip_code' | 'province'
export type ClientFormErrors = Partial<Record<ClientField, string>>

export const CLIENT_FIELDS: readonly ClientField[] = ['first_name', 'last_name', 'fiscal_code', 'vat_number', 'email', 'zip_code', 'province']

export const CLIENT_TYPE_OPTIONS: SegmentOption<ClientType>[] = [
  { value: 'persona_fisica', label: 'Persona fisica' },
  { value: 'azienda', label: 'Azienda o ente' },
]

export function blankClientForm(): CreateClientInput {
  return {
    client_type: 'persona_fisica',
    first_name: '',
    last_name: '',
    birth_date: undefined,
    gender: undefined,
    fiscal_code: '',
    vat_number: undefined,
    address: '',
    city: '',
    province: '',
    zip_code: '',
    email: undefined,
    phone: '',
    notes: undefined,
    sts_authorization: false,
    hide_quantity_in_invoice: null,
  }
}

export function formFromClient(client: Client): CreateClientInput {
  return {
    client_type: client.client_type,
    first_name: client.first_name,
    last_name: client.last_name,
    birth_date: client.birth_date,
    gender: client.gender,
    fiscal_code: client.fiscal_code,
    vat_number: client.vat_number,
    address: client.address,
    city: client.city,
    province: client.province,
    zip_code: client.zip_code,
    email: client.email,
    phone: client.phone,
    notes: client.notes,
    sts_authorization: client.sts_authorization,
    hide_quantity_in_invoice: client.hide_quantity_in_invoice,
  }
}

export function checkClientField(field: ClientField, form: CreateClientInput): ValidationResult {
  const company = form.client_type === 'azienda'
  switch (field) {
    case 'first_name':
      return company ? { valid: true } : required(form.first_name, 'Serve il nome.')
    case 'last_name':
      return required(form.last_name, company ? 'Serve la ragione sociale.' : 'Serve il cognome.')
    case 'fiscal_code': {
      const presence = required(form.fiscal_code, 'Serve il codice fiscale.')
      return presence.valid ? validateCodiceFiscale(form.fiscal_code) : presence
    }
    case 'vat_number':
      return company ? validatePartitaIva(form.vat_number ?? '') : { valid: true }
    case 'email':
      return validateEmail(form.email ?? '')
    case 'zip_code':
      return validateCap(form.zip_code)
    case 'province':
      return validateProvincia(form.province)
  }
}

/** The line under the name on the record card: the kind of patient, or the age when known. */
export function cardDescription(form: Pick<CreateClientInput, 'client_type' | 'birth_date'>, today: Date = new Date()): string {
  if (form.client_type === 'azienda') return 'Azienda o ente'
  const age = ageOn(form.birth_date, today)
  return age === null ? 'Persona fisica' : `${age} anni`
}

/** Street, then "CAP Città (PR)", leaving out what is blank. */
export function cardPlace(form: Pick<CreateClientInput, 'address' | 'zip_code' | 'city' | 'province'>): string[] {
  const place = [form.zip_code, form.city].filter(Boolean).join(' ')
  return [form.address, place + (form.province ? ` (${form.province})` : '')].filter((part) => part.trim() !== '')
}

export function newestFirst(invoices: readonly Invoice[]): Invoice[] {
  return [...invoices].sort((a, b) => b.issue_date.localeCompare(a.issue_date))
}

/** What the patient has been invoiced, cancelled invoices aside. */
export function historyTotal(invoices: readonly Pick<Invoice, 'status' | 'total_due'>[]): number {
  return invoices.filter((invoice) => invoice.status !== 'cancelled').reduce((sum, invoice) => sum + invoice.total_due, 0)
}

function required(value: string | undefined, message: string): ValidationResult {
  return value && value.trim() !== '' ? { valid: true } : { valid: false, message }
}

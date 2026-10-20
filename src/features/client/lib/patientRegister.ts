/** Whose record in the patient register still lacks what an invoice or its email needs. */
import type { Client } from '@/features/client/model/client'

export type MissingField = 'codice fiscale' | 'indirizzo' | 'email'

export type ClientForCompleteness = Pick<
  Client,
  'client_type' | 'fiscal_code' | 'vat_number' | 'address' | 'city' | 'zip_code' | 'email'
>

const blank = (value: string | undefined): boolean => (value ?? '').trim() === ''

/** Fields an invoice or its email would be missing, in the order the form asks for them. */
export function missingFields(client: ClientForCompleteness): MissingField[] {
  const missing: MissingField[] = []
  const hasTaxId = client.client_type === 'azienda' ? !blank(client.vat_number) || !blank(client.fiscal_code) : !blank(client.fiscal_code)
  if (!hasTaxId) missing.push('codice fiscale')
  if (blank(client.address) || blank(client.city) || blank(client.zip_code)) missing.push('indirizzo')
  if (blank(client.email)) missing.push('email')
  return missing
}

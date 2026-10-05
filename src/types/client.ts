export type ClientType = 'persona_fisica' | 'azienda'

export interface Client {
  id: number
  client_type: ClientType
  first_name: string
  last_name: string
  birth_date?: string
  gender?: string
  fiscal_code: string
  vat_number?: string
  address: string
  city: string
  province: string
  zip_code: string
  email?: string
  phone: string
  notes?: string
  sts_authorization: boolean
  /** Null follows the profile's setting. */
  hide_quantity_in_invoice: boolean | null
  created_at: string
  updated_at: string
}

export type CreateClientInput = Omit<Client, 'id' | 'created_at' | 'updated_at'>

export type UpdateClientInput = Omit<Client, 'created_at' | 'updated_at'>

export type TaxRegime = 'forfettario' | 'ordinario'

export type Profession = 'psicologo' | 'psicoterapeuta'

export interface ProfessionalConfig {
  id: number
  title: string
  first_name: string
  last_name: string
  vat_number: string
  fiscal_code: string
  tax_regime: TaxRegime
  albo_number: string
  albo_region: string
  address: string
  city: string
  province: string
  zip_code: string
  country: string
  phone: string
  pec_email: string
  iban: string
  coefficient: number
  profession: Profession
  is_psicoanalista: boolean
  specialization: string
  hide_quantity_in_invoice: boolean
  /** Leaves the bollo out of the ENPAP base in the forfettario regime too. */
  enpap_excludes_bollo: boolean
  initial_invoice_number: number
  created_at: string
  updated_at: string
}

export type UpsertConfigInput = Omit<ProfessionalConfig, 'id' | 'created_at' | 'updated_at'>

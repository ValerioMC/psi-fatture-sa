export interface Service {
  id: number
  name: string
  description: string
  default_price: number
  vat_rate: number
  is_active: boolean
  created_at: string
  updated_at: string
}

export type CreateServiceInput = Omit<Service, 'id' | 'created_at' | 'updated_at'>

export type UpdateServiceInput = Omit<Service, 'created_at' | 'updated_at'>

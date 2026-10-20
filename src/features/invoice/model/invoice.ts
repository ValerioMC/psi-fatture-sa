export type InvoiceStatus = 'draft' | 'issued' | 'paid' | 'overdue' | 'cancelled'

export type PaymentMethod = 'bonifico' | 'contanti' | 'pos' | 'altro'

/** Every amount of an invoice, as the backend computes and stores it. */
export interface InvoiceTotals {
  total_net: number
  total_tax: number
  contributo_enpap: number
  ritenuta_acconto: number
  marca_da_bollo: number
  total_gross: number
  total_due: number
}

export interface InvoiceLineInput {
  service_id?: number
  description: string
  quantity: number
  unit_price: number
  vat_rate: number
  /** Net amount typed by hand, replacing quantity × unit price. */
  amount_override?: number | null
}

export interface InvoiceLine extends InvoiceLineInput {
  id?: number
  invoice_id?: number
  line_total: number
}

export interface CreateInvoiceInput {
  client_id: number
  issue_date: string
  due_date?: string
  status: InvoiceStatus
  payment_method: PaymentMethod
  notes: string
  apply_enpap: boolean
  /** Omitted: the patient's setting, then the profile's (on update: the saved one). */
  hide_quantity?: boolean
  lines: InvoiceLineInput[]
}

export interface UpdateInvoiceInput extends CreateInvoiceInput {
  id: number
  /** New invoice number for manual renumbering; omit to keep the current one. */
  invoice_number?: string
  paid_date?: string
}

export interface Invoice {
  id: number
  client_id: number
  client_name: string
  invoice_number: string
  year: number
  issue_date: string
  due_date?: string
  status: InvoiceStatus
  payment_method: PaymentMethod
  notes: string
  apply_enpap: boolean
  contributo_enpap: number
  ritenuta_acconto: number
  marca_da_bollo: boolean
  total_net: number
  total_tax: number
  total_gross: number
  total_due: number
  paid_date?: string
  hide_quantity: boolean
  lines: InvoiceLine[]
  created_at: string
  updated_at: string
}

export interface MonthlyInvoicePreview {
  client_id: number
  client_name: string
  appointment_count: number
  lines: InvoiceLineInput[]
  estimated_net: number
  estimated_due: number
  /** Some session has no service or a zero price: it cannot be billed yet. */
  missing_price: boolean
}

export interface GenerateMonthlyInput {
  year: number
  month: number
  client_ids: number[]
  payment_method: PaymentMethod
  apply_enpap: boolean
  /** Defaults to the last day of the month. */
  issue_date?: string | null
}

export interface BulkUpdateStatusInput {
  ids: number[]
  status: InvoiceStatus
  paid_date?: string
}

export interface InvoiceFilters {
  year?: number
  status?: string
  client_id?: number
  search?: string
}

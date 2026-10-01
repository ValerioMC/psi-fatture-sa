/** TypeScript types mirroring Rust domain models. */

export type TaxRegime = 'forfettario' | 'ordinario'
export type Profession = 'psicologo' | 'psicoterapeuta'
export type ClientType = 'persona_fisica' | 'azienda'
export type InvoiceStatus = 'draft' | 'issued' | 'paid' | 'overdue' | 'cancelled'
export type PaymentMethod = 'bonifico' | 'contanti' | 'pos' | 'altro'
export type AppointmentStatus = 'scheduled' | 'completed' | 'cancelled'

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

export interface CreateAppointmentInput {
  client_id: number
  service_id?: number
  date: string
  start_time: string
  end_time: string
  status: AppointmentStatus
  notes: string
  recurrence_group_id?: number
}

export interface UpdateAppointmentInput extends CreateAppointmentInput {
  id: number
}

export interface CreateRecurringAppointmentsInput {
  client_id: number
  service_id?: number
  dates: string[]
  start_time: string
  end_time: string
  notes: string
}

export interface Appointment {
  id: number
  client_id: number
  client_name: string
  service_id?: number
  service_name?: string
  date: string
  start_time: string
  end_time: string
  status: AppointmentStatus
  notes: string
  recurrence_group_id?: number
  invoice_id?: number
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
}

export interface GenerateMonthlyInput {
  year: number
  month: number
  client_ids: number[]
  payment_method: PaymentMethod
  apply_enpap: boolean
}

export interface MonthlyRevenue {
  month: number
  month_name: string
  revenue: number
  invoice_count: number
}

export interface DashboardData {
  year: number
  total_revenue: number
  total_net_revenue: number
  paid_revenue: number
  unpaid_revenue: number
  total_invoices: number
  paid_invoices: number
  draft_invoices: number
  monthly_revenue: MonthlyRevenue[]
  recent_invoices: Invoice[]
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

// ─── Sistema Tessera Sanitaria ───────────────────────────────────────────────

export type TsEnvironment = 'test' | 'produzione'
export type TsOperation = 'invio' | 'sostituzione' | 'annullamento'
export type TsSubmissionStatus =
  | 'non_inviata'
  | 'inviata'
  | 'accettata'
  | 'scartata'
  | 'annullata'
  | 'sostituita'

/** How the Sistema TS identifies a document: issuer P.IVA, issue date, number. */
export interface TsDocumentId {
  vat_number: string
  issue_date: string
  number: string
}

export interface TsSubmission {
  id: number
  invoice_id: number
  invoice_number: string
  invoice_year: number
  client_name: string
  operation: TsOperation
  status: TsSubmissionStatus
  target_submission_id: number | null
  environment: TsEnvironment
  document: TsDocumentId | null
  protocol: string | null
  outcome_code: string | null
  outcome_message: string | null
  attempt_count: number
  last_error: string | null
  next_attempt_at: string
  last_attempt_at: string | null
  sent_at: string | null
  resolved_at: string | null
  created_at: string
  updated_at: string
}

export interface TsSubmissionFilters {
  invoice_id?: number
  year?: number
  status?: TsSubmissionStatus
}

/** Non-secret settings: `username` is the codice fiscale used to log in. */
export interface TsSettings {
  environment: TsEnvironment
  username: string
  vat_number: string
}

/** Whether the secrets are stored; their values never reach the UI. */
export interface TsCredentialsStatus {
  password_configured: boolean
  pincode_configured: boolean
}

export interface TsConnectionCheck {
  ok: boolean
  message: string
}

export interface TsDispatchSummary {
  accepted: number
  rejected: number
  retrying: number
  waiting_other_environment: number
  blocked: string | null
}

export interface TsMessage {
  code: string
  description: string
  kind: string
}

export interface TsExpenseTotal {
  expense_type: string
  amount: number
}

export interface TsRemoteDocument {
  id: TsDocumentId
  payment_date: string | null
  totals: TsExpenseTotal[]
  refunded_totals: TsExpenseTotal[]
  protocol: string | null
  sent_date: string | null
  send_kind: string | null
  cancelled: boolean
  messages: TsMessage[]
}

export type TsQueryResult =
  | { kind: 'found'; document: TsRemoteDocument }
  | { kind: 'not_found' }
  | { kind: 'refused'; messages: TsMessage[] }

export type TsReportBasis = 'invio' | 'pagamento'

export interface TsReportRow {
  vat_number: string
  issue_date: string
  document_number: string
  payment_date: string
  protocol: string
  sent_date: string
  send_kind: string
  amount: number
  refunded_amount: number
  invoice_id: number | null
}

// ─── Email ───────────────────────────────────────────────────────────────────

export type EmailProvider = 'psypec' | 'aruba_pec' | 'gmail' | 'custom'
export type EmailSecurity = 'tls' | 'starttls'
export type InvoiceEmailStatus = 'sent' | 'failed'

/** A provider's SMTP settings; `certified` marks a PEC mailbox. */
export interface EmailProviderPreset {
  provider: EmailProvider
  label: string
  host: string
  port: number
  security: EmailSecurity
  domains: string[]
  certified: boolean
  note: string
}

/** The mailbox invoices leave from; `saved` is false while it is only proposed from the profile. */
export interface EmailAccount {
  provider: EmailProvider
  sender_address: string
  sender_name: string
  smtp_host: string
  smtp_port: number
  security: EmailSecurity
  username: string
  bcc_self: boolean
  saved: boolean
}

export type UpdateEmailAccountInput = Omit<EmailAccount, 'saved'>

export interface EmailCredentialsStatus {
  password_configured: boolean
}

export interface EmailConnectionCheck {
  ok: boolean
  message: string
}

export interface EmailTemplate {
  subject: string
  body: string
}

export interface EmailPlaceholderInfo {
  key: string
  label: string
}

export type EmailPreview = EmailTemplate

/** The email proposed for one invoice; `recipient_on_file` is false when the patient has no address. */
export interface EmailDraft {
  invoice_id: number
  recipient: string
  recipient_on_file: boolean
  subject: string
  body: string
  attachment_name: string
}

export interface SendInvoiceEmailInput {
  invoice_id: number
  recipient: string
  subject: string
  body: string
  remember_recipient: boolean
}

/** One attempt to email an invoice; `sent_at` is UTC, "YYYY-MM-DD HH:MM:SS". */
export interface InvoiceEmail {
  id: number
  invoice_id: number
  recipient: string
  subject: string
  attachment_name: string
  status: InvoiceEmailStatus
  error: string | null
  sent_at: string
}

/** The acceptance of one version of the terms of use, with the specific approval of its limitation clauses. */
export interface TermsAcceptance {
  version: string
  clauses_approved: boolean
  accepted_at: string
}

export interface AcceptTermsInput {
  version: string
  terms_accepted: boolean
  clauses_approved: boolean
}

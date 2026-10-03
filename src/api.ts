/** Thin wrapper around Tauri invoke for all backend commands. */
import { invoke } from '@tauri-apps/api/core'
import type {
  AcceptTermsInput,
  Appointment,
  BackupFile,
  BackupOverview,
  BulkUpdateStatusInput,
  Client,
  CreateAppointmentInput,
  CreateClientInput,
  CreateInvoiceInput,
  CreateRecurringAppointmentsInput,
  CreateServiceInput,
  DashboardData,
  EmailAccount,
  EmailConnectionCheck,
  EmailCredentialsStatus,
  EmailDraft,
  EmailPlaceholderInfo,
  EmailPreview,
  EmailProviderPreset,
  EmailTemplate,
  GenerateMonthlyInput,
  Invoice,
  InvoiceEmail,
  InvoiceFilters,
  InvoiceLineInput,
  InvoiceTotals,
  MonthlyInvoicePreview,
  ProfessionalConfig,
  SendInvoiceEmailInput,
  Service,
  TermsAcceptance,
  TsConnectionCheck,
  TsCredentialsStatus,
  TsDispatchSummary,
  TsEnvironment,
  TsQueryResult,
  TsReportBasis,
  TsReportRow,
  TsSettings,
  TsSubmission,
  TsSubmissionFilters,
  UpdateAppointmentInput,
  UpdateClientInput,
  UpdateEmailAccountInput,
  UpdateInvoiceInput,
  UpdateCheck,
  UpdateServiceInput,
  UpsertConfigInput,
} from './types'

// ─── Config ──────────────────────────────────────────────────────────────────

export const getConfig = () => invoke<ProfessionalConfig | null>('get_config')
export const upsertConfig = (input: UpsertConfigInput) =>
  invoke<ProfessionalConfig>('upsert_config', { input })

// ─── Clients ─────────────────────────────────────────────────────────────────

export const listClients = (search?: string) =>
  invoke<Client[]>('list_clients', { search: search ?? null })
export const getClient = (id: number) => invoke<Client>('get_client', { id })
export const createClient = (input: CreateClientInput) =>
  invoke<Client>('create_client', { input })
export const updateClient = (input: UpdateClientInput) =>
  invoke<Client>('update_client', { input })
export const deleteClient = (id: number) => invoke<void>('delete_client', { id })

// ─── Services ────────────────────────────────────────────────────────────────

export const listServices = (activeOnly = false) =>
  invoke<Service[]>('list_services', { activeOnly })
export const getService = (id: number) => invoke<Service>('get_service', { id })
export const createService = (input: CreateServiceInput) =>
  invoke<Service>('create_service', { input })
export const updateService = (input: UpdateServiceInput) =>
  invoke<Service>('update_service', { input })
export const deleteService = (id: number) => invoke<void>('delete_service', { id })

// ─── Invoices ────────────────────────────────────────────────────────────────

export const listInvoices = (filters: InvoiceFilters) =>
  invoke<Invoice[]>('list_invoices', { filters })
export const getInvoice = (id: number) => invoke<Invoice>('get_invoice', { id })
export const createInvoice = (input: CreateInvoiceInput) =>
  invoke<Invoice>('create_invoice', { input })
export const updateInvoice = (input: UpdateInvoiceInput) =>
  invoke<Invoice>('update_invoice', { input })
export const deleteInvoice = (id: number) => invoke<void>('delete_invoice', { id })
export const getNextInvoiceNumber = (year: number) =>
  invoke<string>('get_next_invoice_number', { year })
export const previewInvoiceTotals = (lines: InvoiceLineInput[], applyEnpap: boolean) =>
  invoke<InvoiceTotals>('preview_invoice_totals', { lines, applyEnpap })
export const previewMonthlyInvoices = (year: number, month: number) =>
  invoke<MonthlyInvoicePreview[]>('preview_monthly_invoices', { year, month })
export const generateMonthlyInvoices = (input: GenerateMonthlyInput) =>
  invoke<Invoice[]>('generate_monthly_invoices', { input })
export const bulkUpdateInvoiceStatus = (input: BulkUpdateStatusInput) =>
  invoke<number>('bulk_update_invoice_status', { input })

// ─── Backup ──────────────────────────────────────────────────────────────────

export const exportBackup = (path: string) => invoke<void>('export_backup', { path })
export const getBackupFileName = () => invoke<string>('get_backup_file_name')
export const getBackupOverview = () => invoke<BackupOverview>('get_backup_overview')
export const createBackup = () => invoke<BackupFile>('create_backup')
export const revealBackupsFolder = () => invoke<void>('reveal_backups_folder')

// ─── Updates ─────────────────────────────────────────────────────────────────

export const getAppVersion = () => invoke<string>('get_app_version')
export const checkForUpdate = () => invoke<UpdateCheck>('check_for_update')
/** Mac only: installs the newest release over this app and restarts, so on success it never resolves. */
export const installUpdate = () => invoke<void>('install_update')

/** Renders the invoice PDF and opens it in the system viewer. */
export const openInvoicePdf = (invoiceId: number) => invoke<void>('open_invoice_pdf', { invoiceId })
export const saveInvoicePdf = (invoiceId: number, path: string) =>
  invoke<void>('save_invoice_pdf', { invoiceId, path })
export const getInvoicePdfName = (invoiceId: number) =>
  invoke<string>('get_invoice_pdf_name', { invoiceId })

// ─── Appointments ────────────────────────────────────────────────────────────

export const listAppointments = (
  dateFrom?: string,
  dateTo?: string,
  clientId?: number,
) =>
  invoke<Appointment[]>('list_appointments', {
    dateFrom: dateFrom ?? null,
    dateTo: dateTo ?? null,
    clientId: clientId ?? null,
  })
export const getAppointment = (id: number) => invoke<Appointment>('get_appointment', { id })
export const createAppointment = (input: CreateAppointmentInput) =>
  invoke<Appointment>('create_appointment', { input })
export const createRecurringAppointments = (input: CreateRecurringAppointmentsInput) =>
  invoke<Appointment[]>('create_recurring_appointments', { input })
export const updateAppointment = (input: UpdateAppointmentInput) =>
  invoke<Appointment>('update_appointment', { input })
export const deleteAppointment = (id: number) => invoke<void>('delete_appointment', { id })

// ─── Dashboard ───────────────────────────────────────────────────────────────

export const getDashboard = (year: number) => invoke<DashboardData>('get_dashboard', { year })

// ─── Sistema Tessera Sanitaria ───────────────────────────────────────────────

/** Production alone in a distributed build; test too in a developer one. */
export const getTsEnvironments = () => invoke<TsEnvironment[]>('get_ts_environments')
export const getTsSettings = () => invoke<TsSettings>('get_ts_settings')
export const updateTsSettings = (input: TsSettings) =>
  invoke<TsSettings>('update_ts_settings', { input })

export const getTsCredentialsStatus = () =>
  invoke<TsCredentialsStatus>('get_ts_credentials_status')
export const saveTsPincode = (pincode: string) =>
  invoke<TsCredentialsStatus>('save_ts_pincode', { pincode })
export const deleteTsPincode = () => invoke<TsCredentialsStatus>('delete_ts_pincode')
export const saveTsPassword = (password: string) =>
  invoke<TsCredentialsStatus>('save_ts_password', { password })
export const deleteTsPassword = () => invoke<TsCredentialsStatus>('delete_ts_password')
export const checkTsConnection = () => invoke<TsConnectionCheck>('check_ts_connection')

export const listTsSubmissions = (filters: TsSubmissionFilters = {}) =>
  invoke<TsSubmission[]>('list_ts_submissions', { filters })
export const enqueueTsSubmission = (invoiceId: number) =>
  invoke<TsSubmission>('enqueue_ts_submission', { invoiceId })
export const isTsInvoiceOutOfDate = (invoiceId: number) =>
  invoke<boolean>('is_ts_invoice_out_of_date', { invoiceId })
export const enqueueTsReplacement = (submissionId: number) =>
  invoke<TsSubmission>('enqueue_ts_replacement', { submissionId })
export const enqueueTsCancellation = (submissionId: number) =>
  invoke<TsSubmission>('enqueue_ts_cancellation', { submissionId })
export const withdrawTsSubmission = (submissionId: number) =>
  invoke<void>('withdraw_ts_submission', { submissionId })
export const dispatchTsQueue = () => invoke<TsDispatchSummary>('dispatch_ts_queue')

export const queryTsInvoice = (invoiceId: number) =>
  invoke<TsQueryResult>('query_ts_invoice', { invoiceId })
export const getTsMonthlyReport = (year: number, month: number, basis: TsReportBasis) =>
  invoke<TsReportRow[]>('get_ts_monthly_report', { year, month, basis })

// ─── Email ───────────────────────────────────────────────────────────────────

export const getEmailProviders = () => invoke<EmailProviderPreset[]>('get_email_providers')
export const getEmailAccount = () => invoke<EmailAccount>('get_email_account')
export const updateEmailAccount = (input: UpdateEmailAccountInput) =>
  invoke<EmailAccount>('update_email_account', { input })
export const getEmailCredentialsStatus = () =>
  invoke<EmailCredentialsStatus>('get_email_credentials_status')
export const saveEmailPassword = (password: string) =>
  invoke<EmailCredentialsStatus>('save_email_password', { password })
export const deleteEmailPassword = () => invoke<EmailCredentialsStatus>('delete_email_password')
export const checkEmailConnection = () => invoke<EmailConnectionCheck>('check_email_connection')

export const getEmailTemplate = () => invoke<EmailTemplate>('get_email_template')
export const updateEmailTemplate = (template: EmailTemplate) =>
  invoke<EmailTemplate>('update_email_template', { template })
export const resetEmailTemplate = () => invoke<EmailTemplate>('reset_email_template')
export const getEmailPlaceholders = () => invoke<EmailPlaceholderInfo[]>('get_email_placeholders')
export const previewEmailTemplate = (template: EmailTemplate) =>
  invoke<EmailPreview>('preview_email_template', { template })

export const prepareInvoiceEmail = (invoiceId: number) =>
  invoke<EmailDraft>('prepare_invoice_email', { invoiceId })
export const sendInvoiceEmail = (input: SendInvoiceEmailInput) =>
  invoke<InvoiceEmail>('send_invoice_email', { input })
export const sendPreparedInvoiceEmail = (invoiceId: number) =>
  invoke<InvoiceEmail>('send_prepared_invoice_email', { invoiceId })
export const listInvoiceEmails = (invoiceId?: number) =>
  invoke<InvoiceEmail[]>('list_invoice_emails', { filters: { invoice_id: invoiceId ?? null } })

export const getTermsAcceptance = (version: string) =>
  invoke<TermsAcceptance | null>('get_terms_acceptance', { version })
export const acceptTerms = (input: AcceptTermsInput) => invoke<TermsAcceptance>('accept_terms', { input })

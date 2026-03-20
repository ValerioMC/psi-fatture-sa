/** Thin wrapper around Tauri invoke for all backend commands. */
import { invoke } from '@tauri-apps/api/core'
import type {
  Appointment,
  Client,
  CreateAppointmentInput,
  CreateClientInput,
  CreateInvoiceInput,
  CreateRecurringAppointmentsInput,
  CreateServiceInput,
  DashboardData,
  Invoice,
  InvoiceFilters,
  ProfessionalConfig,
  Service,
  UpdateAppointmentInput,
  UpdateClientInput,
  UpdateInvoiceInput,
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

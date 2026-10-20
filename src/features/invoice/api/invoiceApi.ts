/** Invoices, their totals and monthly invoicing. */
import { invoke } from '@tauri-apps/api/core'
import type { BulkUpdateStatusInput, CreateInvoiceInput, GenerateMonthlyInput, Invoice, InvoiceFilters, InvoiceLineInput, InvoiceTotals, MonthlyInvoicePreview, UpdateInvoiceInput } from '@/features/invoice/model/invoice'

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

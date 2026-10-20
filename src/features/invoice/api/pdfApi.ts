/** The invoice as a PDF and as a printed page. */
import { invoke } from '@tauri-apps/api/core'

/** Renders the invoice PDF and opens it in the system viewer. */
export const openInvoicePdf = (invoiceId: number) => invoke<void>('open_invoice_pdf', { invoiceId })
export const saveInvoicePdf = (invoiceId: number, path: string) =>
  invoke<void>('save_invoice_pdf', { invoiceId, path })
export const getInvoicePdfName = (invoiceId: number) =>
  invoke<string>('get_invoice_pdf_name', { invoiceId })
/** Opens the system print dialog for the page the webview shows. */
export const printCurrentPage = () => invoke<void>('print_current_page')

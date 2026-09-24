/** Builds update payloads from a saved invoice, for one-click status changes. */
import type { Invoice, InvoiceStatus, UpdateInvoiceInput } from '@/types'

/**
 * The full update payload for an invoice with a new status. The backend
 * replaces the whole invoice, so every field and line is sent back as saved;
 * a paid date is kept only for a paid invoice.
 */
export function withStatus(invoice: Invoice, status: InvoiceStatus, paidDate?: string): UpdateInvoiceInput {
  return {
    id: invoice.id,
    client_id: invoice.client_id,
    issue_date: invoice.issue_date,
    due_date: invoice.due_date,
    status,
    payment_method: invoice.payment_method,
    notes: invoice.notes,
    apply_enpap: invoice.apply_enpap,
    paid_date: status === 'paid' ? paidDate ?? invoice.paid_date : undefined,
    lines: invoice.lines.map((line) => ({
      service_id: line.service_id,
      description: line.description,
      quantity: line.quantity,
      unit_price: line.unit_price,
      vat_rate: line.vat_rate,
    })),
  }
}

import { describe, expect, it } from 'vitest'
import { withStatus } from './invoiceInput'
import type { Invoice } from '@/features/invoice/model/invoice'

describe('withStatus', () => {
  const invoice: Invoice = {
    id: 7, client_id: 3, client_name: 'Rossi Maria', invoice_number: '12', year: 2026,
    issue_date: '2026-09-01', due_date: '2026-10-01', status: 'issued', payment_method: 'bonifico',
    notes: 'nota', apply_enpap: true, contributo_enpap: 1.6, ritenuta_acconto: 0, marca_da_bollo: true,
    total_net: 80, total_tax: 0, total_gross: 81.6, total_due: 83.6, paid_date: undefined, hide_quantity: true,
    lines: [{ id: 1, invoice_id: 7, service_id: 2, description: 'Seduta', quantity: 1, unit_price: 80, vat_rate: 0, line_total: 80, amount_override: null }],
    created_at: '', updated_at: '',
  }

  it('sends back every saved field with the new status and paid date', () => {
    expect(withStatus(invoice, 'paid', '2026-09-24')).toEqual({
      id: 7, client_id: 3, issue_date: '2026-09-01', due_date: '2026-10-01', status: 'paid',
      payment_method: 'bonifico', notes: 'nota', apply_enpap: true, hide_quantity: true, paid_date: '2026-09-24',
      lines: [{ service_id: 2, description: 'Seduta', quantity: 1, unit_price: 80, vat_rate: 0, amount_override: null }],
    })
  })

  it('drops the paid date for any other status', () => {
    expect(withStatus({ ...invoice, status: 'paid', paid_date: '2026-09-20' }, 'issued').paid_date).toBeUndefined()
  })
})

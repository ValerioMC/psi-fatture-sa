import { describe, expect, it } from 'vitest'
import { readThemePreference, resolveTheme, writeThemePreference, THEME_STORAGE_KEY } from './theme'
import { withStatus } from './invoiceInput'
import { errorMessage, toastDuration, TOAST_ACTION_BONUS_MS, TOAST_DURATION_MS } from '@/stores/toast'
import type { Invoice } from '@/types'

describe('theme preference', () => {
  it('follows the system only when asked to', () => {
    expect(resolveTheme('system', true)).toBe('dark')
    expect(resolveTheme('system', false)).toBe('light')
    expect(resolveTheme('light', true)).toBe('light')
  })

  it('falls back to system on a missing, invalid or throwing storage', () => {
    expect(readThemePreference(undefined)).toBe('system')
    expect(readThemePreference({ getItem: () => 'purple' })).toBe('system')
    expect(readThemePreference({ getItem: () => { throw new Error('blocked') } })).toBe('system')
    expect(readThemePreference({ getItem: (key) => (key === THEME_STORAGE_KEY ? 'dark' : null) })).toBe('dark')
  })

  it('reports whether the preference was stored', () => {
    expect(writeThemePreference({ setItem: () => undefined }, 'dark')).toBe(true)
    expect(writeThemePreference({ setItem: () => { throw new Error('quota') } }, 'dark')).toBe(false)
  })
})

describe('toast', () => {
  it('keeps errors longer than successes and extends for an action', () => {
    expect(TOAST_DURATION_MS.error).toBeGreaterThan(TOAST_DURATION_MS.success)
    expect(toastDuration('success', true)).toBe(TOAST_DURATION_MS.success + TOAST_ACTION_BONUS_MS)
  })

  it('translates database errors into sentences', () => {
    expect(errorMessage('error returned from database: FOREIGN KEY constraint failed')).toMatch(/collegato a fatture/)
    expect(errorMessage(new Error('Disco pieno'))).toBe('Disco pieno')
    expect(errorMessage(undefined)).toMatch(/imprevisto/)
  })
})

describe('withStatus', () => {
  const invoice: Invoice = {
    id: 7, client_id: 3, client_name: 'Rossi Maria', invoice_number: '12', year: 2026,
    issue_date: '2026-09-01', due_date: '2026-10-01', status: 'issued', payment_method: 'bonifico',
    notes: 'nota', apply_enpap: true, contributo_enpap: 1.6, ritenuta_acconto: 0, marca_da_bollo: true,
    total_net: 80, total_tax: 0, total_gross: 81.6, total_due: 83.6, paid_date: undefined,
    lines: [{ id: 1, invoice_id: 7, service_id: 2, description: 'Seduta', quantity: 1, unit_price: 80, vat_rate: 0, line_total: 80 }],
    created_at: '', updated_at: '',
  }

  it('sends back every saved field with the new status and paid date', () => {
    expect(withStatus(invoice, 'paid', '2026-09-24')).toEqual({
      id: 7, client_id: 3, issue_date: '2026-09-01', due_date: '2026-10-01', status: 'paid',
      payment_method: 'bonifico', notes: 'nota', apply_enpap: true, paid_date: '2026-09-24',
      lines: [{ service_id: 2, description: 'Seduta', quantity: 1, unit_price: 80, vat_rate: 0 }],
    })
  })

  it('drops the paid date for any other status', () => {
    expect(withStatus({ ...invoice, status: 'paid', paid_date: '2026-09-20' }, 'issued').paid_date).toBeUndefined()
  })
})

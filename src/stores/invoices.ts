import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  listInvoices,
  getInvoice,
  createInvoice,
  updateInvoice,
  deleteInvoice,
  getNextInvoiceNumber,
  bulkUpdateInvoiceStatus,
} from '@/api'
import type {
  Invoice,
  InvoiceFilters,
  InvoiceStatus,
  CreateInvoiceInput,
  UpdateInvoiceInput,
} from '@/types'

export const useInvoicesStore = defineStore('invoices', () => {
  const invoices = ref<Invoice[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchInvoices(filters: InvoiceFilters) {
    loading.value = true
    error.value = null
    try {
      invoices.value = await listInvoices(filters)
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function fetchInvoice(id: number) {
    return getInvoice(id)
  }

  async function addInvoice(input: CreateInvoiceInput) {
    const invoice = await createInvoice(input)
    invoices.value.unshift(invoice)
    return invoice
  }

  async function editInvoice(input: UpdateInvoiceInput) {
    const updated = await updateInvoice(input)
    const idx = invoices.value.findIndex((i) => i.id === updated.id)
    if (idx !== -1) invoices.value[idx] = updated
    return updated
  }

  async function removeInvoice(id: number) {
    await deleteInvoice(id)
    invoices.value = invoices.value.filter((i) => i.id !== id)
  }

  /** Updates the status of multiple invoices and refreshes local state. */
  async function bulkUpdateStatus(ids: number[], status: InvoiceStatus, paidDate?: string) {
    await bulkUpdateInvoiceStatus({ ids, status, paid_date: paidDate })
    for (const inv of invoices.value) {
      if (ids.includes(inv.id)) {
        inv.status = status
        inv.paid_date = paidDate
      }
    }
  }

  async function nextNumber(year: number) {
    return getNextInvoiceNumber(year)
  }

  return {
    invoices,
    loading,
    error,
    fetchInvoices,
    fetchInvoice,
    addInvoice,
    editInvoice,
    removeInvoice,
    bulkUpdateStatus,
    nextNumber,
  }
})

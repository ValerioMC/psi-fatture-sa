/** Deleting one invoice from the list, after the user confirms it. */
import { ref } from 'vue'
import { useInvoicesStore } from '@/features/invoice/store/invoices'
import { useToastStore } from '@/common/store/toast'
import type { Invoice } from '@/features/invoice/model/invoice'

export function useInvoiceDeletion() {
  const invoicesStore = useInvoicesStore()
  const toast = useToastStore()
  const toDelete = ref<Invoice | null>(null)
  const deleting = ref(false)

  async function confirmDelete(): Promise<void> {
    const invoice = toDelete.value
    if (invoice === null) return
    deleting.value = true
    try {
      await invoicesStore.removeInvoice(invoice.id)
      toast.notify(`Fattura N. ${invoice.invoice_number} eliminata`)
      toDelete.value = null
    } catch (error) {
      toast.notifyError(error, 'Eliminazione non riuscita')
    } finally {
      deleting.value = false
    }
  }

  return { toDelete, deleting, confirmDelete }
}

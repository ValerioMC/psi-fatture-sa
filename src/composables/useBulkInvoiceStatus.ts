/** Moves the selected invoices to one status at once; "paid" dates them today. */
import { computed, ref } from 'vue'
import { useInvoicesStore } from '@/stores/invoices'
import { useToastStore } from '@/stores/toast'
import type { InvoiceStatus } from '@/types'
import type { InvoiceSelection } from '@/composables/useInvoiceSelection'
import { todayIso } from '@/utils/format'
import { describeBulkStatus } from '@/utils/invoiceList'
import { plural } from '@/utils/labels'

export function useBulkInvoiceStatus(selection: InvoiceSelection) {
  const invoicesStore = useInvoicesStore()
  const toast = useToastStore()
  const bulkTarget = ref<InvoiceStatus>('paid')
  const bulkConfirmOpen = ref(false)
  const bulkRunning = ref(false)

  const bulkBlastRadius = computed(() => describeBulkStatus(selection.selectedInvoices.value, bulkTarget.value))

  async function runBulk(): Promise<void> {
    bulkRunning.value = true
    const ids = [...selection.selected.value]
    try {
      await invoicesStore.bulkUpdateStatus(ids, bulkTarget.value, bulkTarget.value === 'paid' ? todayIso() : undefined)
      toast.notify(`${plural(ids.length, 'fattura aggiornata', 'fatture aggiornate')}`)
      selection.clearSelection()
      bulkConfirmOpen.value = false
    } catch (error) {
      toast.notifyError(error, 'Aggiornamento non riuscito')
    } finally {
      bulkRunning.value = false
    }
  }

  return { bulkTarget, bulkConfirmOpen, bulkRunning, bulkBlastRadius, runBulk }
}

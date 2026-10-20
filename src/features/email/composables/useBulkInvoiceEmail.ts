/**
 * Emails the selected invoices one by one with the saved template. One login comes
 * first, so a wrong password stops before any invoice instead of failing each one.
 */
import { computed, ref } from 'vue'
import { checkEmailConnection } from '@/features/email/api/emailApi'
import { useEmailStore } from '@/features/email/store/email'
import { errorMessage, useToastStore } from '@/common/store/toast'
import type { Invoice } from '@/features/invoice/model/invoice'
import type { InvoiceSelection } from '@/features/invoice/composables/useInvoiceSelection'
import {
  describeBulkEmail,
  describeBulkEmailOutcome,
  describeSendProgress,
  isEmailable,
  type SendProgress,
} from '@/features/invoice/lib/invoiceList'

export function useBulkInvoiceEmail(selection: InvoiceSelection) {
  const email = useEmailStore()
  const toast = useToastStore()
  const bulkEmailOpen = ref(false)
  const bulkEmailProgress = ref<SendProgress | null>(null)

  const emailable = computed(() => selection.selectedInvoices.value.filter(isEmailable))
  const bulkEmailBlastRadius = computed(() =>
    describeBulkEmail(
      emailable.value.length,
      emailable.value.filter((invoice) => email.stateOf(invoice.id).kind === 'sent').length,
      selection.selectedInvoices.value.length - emailable.value.length,
    ),
  )
  const bulkEmailMessage = computed(() => describeSendProgress(bulkEmailProgress.value))

  async function runBulkEmail(): Promise<void> {
    const targets = [...emailable.value]
    bulkEmailProgress.value = { done: 0, total: targets.length }
    try {
      const check = await checkEmailConnection()
      if (!check.ok) {
        toast.notifyError(check.message, 'Nessuna email inviata')
        return
      }
      const failures = await sendEach(targets)
      const outcome = describeBulkEmailOutcome(targets.length - failures.length, failures)
      if (failures.length === 0) toast.notify(outcome)
      else toast.notifyError(outcome)
      selection.clearSelection()
      bulkEmailOpen.value = false
    } catch (error) {
      toast.notifyError(error, 'Invio non riuscito')
    } finally {
      bulkEmailProgress.value = null
    }
  }

  /** Returns one line per invoice that did not leave, naming it. */
  async function sendEach(targets: readonly Invoice[]): Promise<string[]> {
    const failures: string[] = []
    for (const [index, invoice] of targets.entries()) {
      try {
        await email.sendPrepared(invoice.id)
      } catch (error) {
        failures.push(`N. ${invoice.invoice_number}: ${errorMessage(error)}`)
      }
      bulkEmailProgress.value = { done: index + 1, total: targets.length }
    }
    return failures
  }

  return { bulkEmailOpen, bulkEmailProgress, emailable, bulkEmailBlastRadius, bulkEmailMessage, runBulkEmail }
}

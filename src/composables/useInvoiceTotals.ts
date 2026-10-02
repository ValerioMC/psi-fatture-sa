/**
 * The live totals of an invoice being written, computed by the backend with the
 * same code that saves it. Requests are debounced; a late answer to an older
 * edit never overwrites a newer one.
 */
import { ref, type Ref } from 'vue'
import { watchDebounced } from '@vueuse/core'
import { previewInvoiceTotals } from '@/api'
import { useToastStore } from '@/stores/toast'
import type { InvoiceLineInput, InvoiceTotals } from '@/types'
import { toLineInput } from '@/utils/tax'

const ZERO_TOTALS: InvoiceTotals = {
  total_net: 0,
  total_tax: 0,
  contributo_enpap: 0,
  ritenuta_acconto: 0,
  marca_da_bollo: 0,
  total_gross: 0,
  total_due: 0,
}

const DEBOUNCE_MS = 120

export function useInvoiceTotals(
  lines: () => readonly InvoiceLineInput[],
  applyEnpap: () => boolean,
): Ref<InvoiceTotals> {
  const toast = useToastStore()
  const totals = ref<InvoiceTotals>({ ...ZERO_TOTALS })
  let latest = 0

  async function refresh(): Promise<void> {
    const request = ++latest
    try {
      const next = await previewInvoiceTotals(lines().map(toLineInput), applyEnpap())
      if (request === latest) totals.value = next
    } catch (error) {
      if (request === latest) toast.notifyError(error, 'Riepilogo non aggiornato')
    }
  }

  watchDebounced([lines, applyEnpap], refresh, { deep: true, debounce: DEBOUNCE_MS, immediate: true })
  return totals
}

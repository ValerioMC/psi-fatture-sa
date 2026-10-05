/** Ticked invoices of the list. "Select all" acts on the visible rows; the selection can outlive a filter. */
import { computed, ref } from 'vue'
import type { Invoice } from '@/types'

export type InvoiceSelection = ReturnType<typeof useInvoiceSelection>

export function useInvoiceSelection(visible: () => readonly Invoice[], all: () => readonly Invoice[]) {
  const selected = ref<Set<number>>(new Set())

  const allVisibleSelected = computed(() => visible().length > 0 && visible().every((invoice) => selected.value.has(invoice.id)))
  const someVisibleSelected = computed(() => !allVisibleSelected.value && visible().some((invoice) => selected.value.has(invoice.id)))
  const selectedInvoices = computed(() => all().filter((invoice) => selected.value.has(invoice.id)))

  function toggleAll(): void {
    selected.value = allVisibleSelected.value ? new Set() : new Set(visible().map((invoice) => invoice.id))
  }

  function toggle(id: number): void {
    const next = new Set(selected.value)
    if (next.has(id)) next.delete(id)
    else next.add(id)
    selected.value = next
  }

  function clearSelection(): void {
    selected.value = new Set()
  }

  return { selected, allVisibleSelected, someVisibleSelected, selectedInvoices, toggleAll, toggle, clearSelection }
}

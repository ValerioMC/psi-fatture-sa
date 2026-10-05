/**
 * Whether the invoice hides quantity and unit price. The patient's setting wins over
 * the profile's, and either is followed only until the user flips the switch.
 */
import { computed, ref, watch } from 'vue'
import { useClientsStore } from '@/stores/clients'
import { useConfigStore } from '@/stores/config'
import type { InvoiceForm } from '@/utils/invoiceForm'

export function useHideQuantityDefault(form: InvoiceForm, isEdit: boolean) {
  const clientsStore = useClientsStore()
  const configStore = useConfigStore()
  /** An edited invoice keeps what was saved. */
  const hideQuantityChosen = ref(isEdit)

  const selectedClient = computed(() => clientsStore.clients.find((client) => client.id === form.client_id))
  const patientHideQuantity = computed(() => selectedClient.value?.hide_quantity_in_invoice ?? null)
  const defaultHideQuantity = computed(() => patientHideQuantity.value ?? configStore.config?.hide_quantity_in_invoice ?? false)
  const hideQuantityOrigin = computed(() =>
    patientHideQuantity.value === null ? 'Predefinito dalle impostazioni di fatturazione.' : 'Predefinito per questo paziente.',
  )

  watch(defaultHideQuantity, (hide) => {
    if (!hideQuantityChosen.value) form.hide_quantity = hide
  }, { immediate: true })

  function chooseHideQuantity(hide: boolean): void {
    hideQuantityChosen.value = true
    form.hide_quantity = hide
  }

  return { hideQuantityChosen, hideQuantityOrigin, chooseHideQuantity }
}

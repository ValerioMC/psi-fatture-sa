/**
 * Loads, checks and saves the invoice the form edits. A new invoice starts with
 * one blank line and shows the number it will get; editing loads the saved one.
 */
import { onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getInvoice } from '@/api'
import { useClientsStore } from '@/stores/clients'
import { useConfigStore } from '@/stores/config'
import { useInvoicesStore } from '@/stores/invoices'
import { useServicesStore } from '@/stores/services'
import { useStsStore } from '@/stores/sts'
import { errorMessage, useToastStore } from '@/stores/toast'
import { todayIso } from '@/utils/format'
import {
  blankInvoiceForm,
  blankLine,
  createInvoiceInput,
  formFromInvoice,
  invoiceFormErrors,
  updateInvoiceInput,
  type InvoiceFieldKey,
  type InvoiceForm,
  type InvoiceFormErrors,
} from '@/utils/invoiceForm'

export function useInvoiceForm(editId: number | null) {
  const route = useRoute()
  const router = useRouter()
  const invoicesStore = useInvoicesStore()
  const clientsStore = useClientsStore()
  const servicesStore = useServicesStore()
  const configStore = useConfigStore()
  const sts = useStsStore()
  const toast = useToastStore()

  const isEdit = editId !== null
  const loading = ref(isEdit)
  const loadError = ref<string | null>(null)
  const saving = ref(false)
  const nextNumber = ref<string | null>(null)
  let lineKey = 0
  const nextKey = (): number => lineKey++
  const form = reactive<InvoiceForm>(blankInvoiceForm(blankLine(nextKey()), todayIso()))
  const errors = reactive<InvoiceFormErrors>({})

  /** A `?client=` in the URL preselects the patient of a new invoice. */
  async function load(): Promise<void> {
    try {
      await Promise.all([
        clientsStore.fetchClients(),
        servicesStore.fetchServices(false),
        configStore.isConfigured ? Promise.resolve() : configStore.loadConfig(),
        isEdit && !sts.loaded ? sts.load() : Promise.resolve(),
      ])
      const presetClient = Number(route.query.client)
      if (!isEdit && Number.isInteger(presetClient) && presetClient > 0) form.client_id = presetClient
      if (editId === null) {
        nextNumber.value = await invoicesStore.nextNumber(new Date().getFullYear())
        return
      }
      Object.assign(form, formFromInvoice(await getInvoice(editId), nextKey))
    } catch (error) {
      loadError.value = errorMessage(error)
    } finally {
      loading.value = false
    }
  }

  function validate(): boolean {
    for (const key of Object.keys(errors) as InvoiceFieldKey[]) delete errors[key]
    Object.assign(errors, invoiceFormErrors(form, isEdit))
    return Object.keys(errors).length === 0
  }

  async function onSubmit(): Promise<void> {
    if (form.lines.length === 0) {
      toast.notifyError('Aggiungi almeno una riga alla fattura.')
      return
    }
    if (!validate() || form.client_id === null) return
    saving.value = true
    try {
      const saved = editId !== null
        ? await invoicesStore.editInvoice(updateInvoiceInput(form, form.client_id, editId))
        : await invoicesStore.addInvoice(createInvoiceInput(form, form.client_id))
      toast.notify(`Fattura N. ${saved.invoice_number} ${editId !== null ? 'aggiornata' : 'creata'}`)
      void router.push(`/invoices/${saved.id}`)
    } catch (error) {
      toast.notifyError(error, 'Salvataggio non riuscito')
    } finally {
      saving.value = false
    }
  }

  onMounted(load)

  return { form, errors, loading, loadError, saving, nextNumber, nextKey, onSubmit }
}

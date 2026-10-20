/**
 * Loads, checks, saves and deletes the patient the form edits. Fields are checked
 * when left and all again on save; editing also loads the patient's invoices.
 */
import { computed, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { getClient } from '@/features/client/api/clientApi'
import { listInvoices } from '@/features/invoice/api/invoiceApi'
import { useClientsStore } from '@/features/client/store/clients'
import { useConfigStore } from '@/features/settings/store/config'
import { errorMessage, useToastStore } from '@/common/store/toast'
import type { CreateClientInput } from '@/features/client/model/client'
import type { Invoice } from '@/features/invoice/model/invoice'
import { clientDisplayName } from '@/features/client/lib/client'
import {
  CLIENT_FIELDS,
  blankClientForm,
  checkClientField,
  formFromClient,
  newestFirst,
  type ClientField,
  type ClientFormErrors,
} from '@/features/client/lib/clientForm'

export function useClientForm(editId: number | null) {
  const router = useRouter()
  const clientsStore = useClientsStore()
  const configStore = useConfigStore()
  const toast = useToastStore()

  const loading = ref(editId !== null)
  const loadError = ref<string | null>(null)
  const saving = ref(false)
  const deleteOpen = ref(false)
  const deleting = ref(false)
  const form = reactive<CreateClientInput>(blankClientForm())
  const errors = reactive<ClientFormErrors>({})
  /** The patient's invoices, newest first: the history an edit is made against. */
  const history = ref<Invoice[]>([])
  const displayName = computed(() => clientDisplayName(form))

  function check(field: ClientField): void {
    const result = checkClientField(field, form)
    if (result.valid) delete errors[field]
    else errors[field] = result.message
  }

  function checkAll(): boolean {
    CLIENT_FIELDS.forEach(check)
    return Object.keys(errors).length === 0
  }

  /** The history is a courtesy: when it cannot load, the form still works. */
  async function load(): Promise<void> {
    if (configStore.config === null) void configStore.loadConfig().catch(() => undefined)
    if (editId === null) return
    void listInvoices({ client_id: editId })
      .then((invoices) => { history.value = newestFirst(invoices) })
      .catch(() => { history.value = [] })
    try {
      Object.assign(form, formFromClient(await getClient(editId)))
    } catch (error) {
      loadError.value = errorMessage(error)
    } finally {
      loading.value = false
    }
  }

  async function onSubmit(): Promise<void> {
    if (!checkAll()) {
      toast.notifyError('Alcuni campi vanno corretti prima di salvare.')
      return
    }
    saving.value = true
    try {
      if (editId !== null) await clientsStore.editClient({ id: editId, ...form })
      else await clientsStore.addClient({ ...form })
      toast.notify(`${displayName.value} ${editId !== null ? 'aggiornato' : 'aggiunto ai pazienti'}`)
      void router.push('/clients')
    } catch (error) {
      toast.notifyError(error, 'Salvataggio non riuscito')
    } finally {
      saving.value = false
    }
  }

  async function confirmDelete(): Promise<void> {
    if (editId === null) return
    deleting.value = true
    try {
      await clientsStore.removeClient(editId)
      toast.notify(`${displayName.value} eliminato dall'anagrafica`)
      void router.push('/clients')
    } catch (error) {
      toast.notifyError(error, 'Eliminazione non riuscita')
      deleteOpen.value = false
    } finally {
      deleting.value = false
    }
  }

  onMounted(load)

  return { form, errors, loading, loadError, saving, deleteOpen, deleting, history, displayName, check, onSubmit, confirmDelete }
}

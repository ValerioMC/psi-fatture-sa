<script setup lang="ts">
/**
 * A patient's record: identity for the invoice, address for the PDF, contacts,
 * and the Sistema Tessera Sanitaria consent. Codice fiscale, P.IVA, CAP,
 * provincia and email are checked when the field is left, and again on save.
 */
import { computed, onMounted, reactive, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { ArrowRight, Check, FilePlus, FileText, IdCard, MapPin, ShieldCheck, ShieldOff, Trash2, TriangleAlert } from 'lucide-vue-next'
import { useClientsStore } from '@/stores/clients'
import { errorMessage, useToastStore } from '@/stores/toast'
import { getClient, listInvoices } from '@/api'
import type { ClientType, CreateClientInput, Invoice } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import FormField from '@/components/ui/FormField.vue'
import FormSection from '@/components/ui/FormSection.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import CardHeader from '@/components/ui/CardHeader.vue'
import PatientMonogram from '@/components/ui/PatientMonogram.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import type { SegmentOption } from '@/components/ui/types'
import { ageOn, clientDisplayName } from '@/utils/client'
import { formatCurrency, formatDateShort } from '@/utils/format'
import { plural } from '@/utils/labels'
import {
  validateCap,
  validateCodiceFiscale,
  validateEmail,
  validatePartitaIva,
  validateProvincia,
  type ValidationResult,
} from '@/utils/validation'

const route = useRoute()
const router = useRouter()
const clientsStore = useClientsStore()
const toast = useToastStore()

const editId = route.params.id ? Number(route.params.id) : null
const isEdit = editId !== null
const loading = ref(isEdit)
const loadError = ref<string | null>(null)
const saving = ref(false)
const deleteOpen = ref(false)
const deleting = ref(false)

const form = reactive<CreateClientInput>({
  client_type: 'persona_fisica',
  first_name: '',
  last_name: '',
  birth_date: undefined,
  gender: undefined,
  fiscal_code: '',
  vat_number: undefined,
  address: '',
  city: '',
  province: '',
  zip_code: '',
  email: undefined,
  phone: '',
  notes: undefined,
  sts_authorization: false,
})

const TYPE_OPTIONS: SegmentOption<ClientType>[] = [
  { value: 'persona_fisica', label: 'Persona fisica' },
  { value: 'azienda', label: 'Azienda o ente' },
]

type Field = 'first_name' | 'last_name' | 'fiscal_code' | 'vat_number' | 'email' | 'zip_code' | 'province'
const errors = reactive<Partial<Record<Field, string>>>({})

function required(value: string | undefined, message: string): ValidationResult {
  return value && value.trim() !== '' ? { valid: true } : { valid: false, message }
}

const VALIDATORS: Record<Field, () => ValidationResult> = {
  first_name: () => (form.client_type === 'azienda' ? { valid: true } : required(form.first_name, 'Serve il nome.')),
  last_name: () => required(form.last_name, form.client_type === 'azienda' ? 'Serve la ragione sociale.' : 'Serve il cognome.'),
  fiscal_code: () => {
    const presence = required(form.fiscal_code, 'Serve il codice fiscale.')
    return presence.valid ? validateCodiceFiscale(form.fiscal_code) : presence
  },
  vat_number: () => (form.client_type === 'azienda' ? validatePartitaIva(form.vat_number ?? '') : { valid: true }),
  email: () => validateEmail(form.email ?? ''),
  zip_code: () => validateCap(form.zip_code),
  province: () => validateProvincia(form.province),
}

function check(field: Field): void {
  const result = VALIDATORS[field]()
  if (result.valid) delete errors[field]
  else errors[field] = result.message
}

function checkAll(): boolean {
  ;(Object.keys(VALIDATORS) as Field[]).forEach(check)
  return Object.keys(errors).length === 0
}

const displayName = computed(() => clientDisplayName(form))

/** The record card beside the form: how the patient reads at a glance, updated as it is typed. */
const cardDescription = computed(() => {
  if (form.client_type === 'azienda') return 'Azienda o ente'
  const age = ageOn(form.birth_date)
  return age === null ? 'Persona fisica' : `${age} anni`
})
const cardPlace = computed(() => {
  const place = [form.zip_code, form.city].filter(Boolean).join(' ')
  return [form.address, place + (form.province ? ` (${form.province})` : '')].filter((part) => part.trim() !== '')
})

// The patient's invoices, newest first: the history an edit is made against.
const history = ref<Invoice[]>([])
const historyTotal = computed(() =>
  history.value.filter((invoice) => invoice.status !== 'cancelled').reduce((sum, invoice) => sum + invoice.total_due, 0),
)

onMounted(async () => {
  if (editId === null) return
  void listInvoices({ client_id: editId })
    .then((invoices) => {
      history.value = [...invoices].sort((a, b) => b.issue_date.localeCompare(a.issue_date))
    })
    .catch(() => { history.value = [] })
  try {
    const client = await getClient(editId)
    Object.assign(form, {
      client_type: client.client_type,
      first_name: client.first_name,
      last_name: client.last_name,
      birth_date: client.birth_date,
      gender: client.gender,
      fiscal_code: client.fiscal_code,
      vat_number: client.vat_number,
      address: client.address,
      city: client.city,
      province: client.province,
      zip_code: client.zip_code,
      email: client.email,
      phone: client.phone,
      notes: client.notes,
      sts_authorization: client.sts_authorization,
    })
  } catch (error) {
    loadError.value = errorMessage(error)
  } finally {
    loading.value = false
  }
})

async function onSubmit(): Promise<void> {
  if (!checkAll()) {
    toast.notifyError('Alcuni campi vanno corretti prima di salvare.')
    return
  }
  saving.value = true
  try {
    if (editId !== null) {
      await clientsStore.editClient({ id: editId, ...form })
      toast.notify(`${displayName.value} aggiornato`)
    } else {
      await clientsStore.addClient({ ...form })
      toast.notify(`${displayName.value} aggiunto ai pazienti`)
    }
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
</script>

<template>
  <div>
    <PageHeader
      :title="isEdit ? (displayName || 'Paziente') : 'Nuovo paziente'"
      :back="{ to: '/clients', label: 'Pazienti' }"
    >
      <AppButton v-if="isEdit" :icon="FilePlus" :to="{ path: '/invoices/new', query: { client: editId } }">Nuova fattura</AppButton>
      <AppButton variant="primary" type="submit" form="client-form" :icon="Check" :loading="saving">
        {{ isEdit ? 'Salva' : 'Aggiungi paziente' }}
      </AppButton>
    </PageHeader>

    <div class="page grid grid-cols-1 items-start gap-5 pt-6 pb-12 xl:grid-cols-[minmax(0,1fr)_21rem] 2xl:grid-cols-[minmax(0,1fr)_24rem]">
      <div class="min-w-0">
        <div v-if="loading" class="sheet h-96 p-6" role="status" aria-busy="true">
          <span class="sr-only">Caricamento del paziente</span>
          <div class="skeleton h-4 w-1/3" />
        </div>

        <EmptyState v-else-if="loadError" :icon="TriangleAlert" title="Paziente non trovato" :description="loadError">
          <AppButton to="/clients">Torna ai pazienti</AppButton>
        </EmptyState>

        <AppCard v-else class="settle px-8 py-4">
          <form id="client-form" novalidate @submit.prevent="onSubmit">
            <FormSection title="Chi è" description="Nome e codice fiscale compaiono in fattura come intestatario.">
              <SegmentedControl v-model="form.client_type" :options="TYPE_OPTIONS" label="Tipo di paziente" class="mb-5" />
              <div class="grid grid-cols-2 gap-x-4 gap-y-5">
                <template v-if="form.client_type === 'persona_fisica'">
                  <FormField v-slot="{ id, invalid, describedBy }" label="Nome" required :error="errors.first_name">
                    <input :id="id" v-model="form.first_name" type="text" class="field" autocomplete="off" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="check('first_name')" />
                  </FormField>
                  <FormField v-slot="{ id, invalid, describedBy }" label="Cognome" required :error="errors.last_name">
                    <input :id="id" v-model="form.last_name" type="text" class="field" autocomplete="off" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="check('last_name')" />
                  </FormField>
                  <FormField v-slot="{ id }" label="Data di nascita" optional>
                    <input :id="id" v-model="form.birth_date" type="date" class="field" />
                  </FormField>
                  <FormField v-slot="{ id }" label="Sesso" optional>
                    <select :id="id" v-model="form.gender" class="field">
                      <option :value="undefined">Non indicato</option>
                      <option value="F">Femmina</option>
                      <option value="M">Maschio</option>
                    </select>
                  </FormField>
                </template>
                <template v-else>
                  <FormField v-slot="{ id, invalid, describedBy }" class="col-span-2" label="Ragione sociale" required :error="errors.last_name">
                    <input :id="id" v-model="form.last_name" type="text" class="field" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="check('last_name')" />
                  </FormField>
                  <FormField v-slot="{ id, invalid, describedBy }" label="Partita IVA" required :error="errors.vat_number">
                    <input :id="id" v-model="form.vat_number" type="text" inputmode="numeric" class="field field-mono" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="check('vat_number')" />
                  </FormField>
                </template>
                <FormField v-slot="{ id, invalid, describedBy }" :class="form.client_type === 'azienda' ? '' : 'col-span-2'" label="Codice fiscale" required :error="errors.fiscal_code">
                  <input
                    :id="id"
                    v-model="form.fiscal_code"
                    type="text"
                    class="field field-mono uppercase"
                    maxlength="16"
                    autocomplete="off"
                    spellcheck="false"
                    :aria-invalid="invalid"
                    :aria-describedby="describedBy"
                    @input="form.fiscal_code = form.fiscal_code.toUpperCase()"
                    @blur="check('fiscal_code')"
                  />
                </FormField>
              </div>
            </FormSection>

            <FormSection title="Indirizzo" description="Richiesto in fattura per i pazienti residenti in Italia.">
              <div class="grid grid-cols-[1fr_5rem_6.5rem] gap-x-4 gap-y-5">
                <FormField v-slot="{ id }" class="col-span-3" label="Via e numero civico">
                  <input :id="id" v-model="form.address" type="text" class="field" autocomplete="off" />
                </FormField>
                <FormField v-slot="{ id }" label="Città">
                  <input :id="id" v-model="form.city" type="text" class="field" autocomplete="off" />
                </FormField>
                <FormField v-slot="{ id, invalid, describedBy }" label="Prov." :error="errors.province">
                  <input :id="id" v-model="form.province" type="text" maxlength="2" class="field uppercase" :aria-invalid="invalid" :aria-describedby="describedBy" @input="form.province = form.province.toUpperCase()" @blur="check('province')" />
                </FormField>
                <FormField v-slot="{ id, invalid, describedBy }" label="CAP" :error="errors.zip_code">
                  <input :id="id" v-model="form.zip_code" type="text" inputmode="numeric" maxlength="5" class="field tabular" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="check('zip_code')" />
                </FormField>
              </div>
            </FormSection>

            <FormSection title="Contatti" description="Per ritrovare il paziente e mandargli la fattura.">
              <div class="grid grid-cols-2 gap-x-4 gap-y-5">
                <FormField v-slot="{ id, invalid, describedBy }" label="Email" optional :error="errors.email">
                  <input :id="id" v-model="form.email" type="email" class="field" autocomplete="off" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="check('email')" />
                </FormField>
                <FormField v-slot="{ id }" label="Telefono" optional>
                  <input :id="id" v-model="form.phone" type="tel" class="field tabular" autocomplete="off" />
                </FormField>
                <FormField v-slot="{ id }" class="col-span-2" label="Note" optional hint="Restano nell'app: non compaiono in fattura.">
                  <textarea :id="id" v-model="form.notes" rows="3" class="field" />
                </FormField>
              </div>
            </FormSection>

            <FormSection title="Sistema Tessera Sanitaria" description="Le spese sanitarie vanno trasmesse per la dichiarazione precompilata, salvo opposizione del paziente.">
              <ToggleSwitch
                v-model="form.sts_authorization"
                label="Il paziente autorizza la trasmissione"
                description="In fattura comparirà la casella “Autorizza” spuntata; altrimenti “Non autorizza”."
              />
            </FormSection>
          </form>
        </AppCard>

        <div v-if="isEdit && !loading && !loadError" class="mt-4 flex justify-end">
          <AppButton variant="danger-quiet" :icon="Trash2" @click="deleteOpen = true">Elimina paziente</AppButton>
        </div>
      </div>

      <!-- ── The record card and the history, beside the form ─────────────── -->
      <aside v-if="!loadError" class="settle space-y-5 xl:sticky xl:top-32" style="--settle: 1">
        <AppCard :padded="false" class="record-card overflow-hidden">
          <div class="record-band" aria-hidden="true" />
          <div class="-mt-8 px-5 pb-5">
            <div class="record-avatar inline-grid rounded-full">
              <PatientMonogram :name="displayName || '?'" size="lg" />
            </div>
            <p class="mt-3 truncate text-lg font-semibold text-text">{{ displayName || 'Nuovo paziente' }}</p>
            <p class="text-sm text-text-subtle">{{ cardDescription }}</p>

            <dl class="mt-4 space-y-3 border-t border-dashed border-border-strong pt-4 text-sm">
              <div class="flex items-start gap-2.5">
                <IdCard :size="15" :stroke-width="1.75" class="mt-0.5 shrink-0 text-text-subtle" aria-hidden="true" />
                <div class="min-w-0">
                  <dt class="sr-only">Codice fiscale</dt>
                  <dd class="truncate font-mono tracking-[0.04em] text-text">{{ form.fiscal_code || '— — —' }}</dd>
                </div>
              </div>
              <div class="flex items-start gap-2.5">
                <MapPin :size="15" :stroke-width="1.75" class="mt-0.5 shrink-0 text-text-subtle" aria-hidden="true" />
                <div class="min-w-0">
                  <dt class="sr-only">Indirizzo</dt>
                  <dd v-if="cardPlace.length > 0" class="text-text-muted">
                    <span v-for="line in cardPlace" :key="line" class="block truncate">{{ line }}</span>
                  </dd>
                  <dd v-else class="text-text-subtle">Indirizzo non indicato</dd>
                </div>
              </div>
              <div class="flex items-start gap-2.5">
                <component
                  :is="form.sts_authorization ? ShieldCheck : ShieldOff"
                  :size="15"
                  :stroke-width="1.75"
                  class="mt-0.5 shrink-0"
                  :class="form.sts_authorization ? 'text-safe' : 'text-text-subtle'"
                  aria-hidden="true"
                />
                <div>
                  <dt class="sr-only">Sistema Tessera Sanitaria</dt>
                  <dd :class="form.sts_authorization ? 'text-safe' : 'text-text-muted'">
                    {{ form.sts_authorization ? 'Autorizza la trasmissione STS' : 'Non autorizza la trasmissione STS' }}
                  </dd>
                </div>
              </div>
            </dl>
          </div>
        </AppCard>

        <AppCard v-if="isEdit" :padded="false">
          <CardHeader title="Fatture" :icon="FileText" divided>
            <template #subtitle>
              <template v-if="history.length > 0">{{ plural(history.length, 'fattura', 'fatture') }} · {{ formatCurrency(historyTotal) }}</template>
              <template v-else>Nessuna fattura ancora</template>
            </template>
          </CardHeader>
          <ul v-if="history.length > 0" class="p-1.5">
            <li v-for="invoice in history.slice(0, 5)" :key="invoice.id">
              <RouterLink
                :to="`/invoices/${invoice.id}`"
                class="group flex items-center gap-3 rounded-control px-3 py-2 transition-colors hover:bg-surface-hover focus-ring"
              >
                <span class="min-w-0 flex-1">
                  <span class="tabular block text-sm font-medium text-text">N. {{ invoice.invoice_number }}/{{ invoice.year }}</span>
                  <span class="block text-xs text-text-subtle">{{ formatDateShort(invoice.issue_date) }}</span>
                </span>
                <span class="tabular text-sm font-medium text-text">{{ formatCurrency(invoice.total_due) }}</span>
                <StatusBadge type="invoice" :status="invoice.status" />
              </RouterLink>
            </li>
          </ul>
          <div v-if="history.length > 5" class="border-t border-border px-5 py-2.5">
            <RouterLink :to="{ path: '/invoices', query: { year: 0, q: displayName } }" class="group inline-flex items-center gap-1 text-sm font-medium text-accent focus-ring">
              Tutte le fatture <ArrowRight :size="14" class="transition-transform group-hover:translate-x-0.5" aria-hidden="true" />
            </RouterLink>
          </div>
        </AppCard>
      </aside>
    </div>

    <ConfirmDialog
      :open="deleteOpen"
      title="Eliminare il paziente?"
      message="L'operazione non si può annullare."
      :blast-radius="`${displayName} verrà tolto dall'anagrafica. Se ha fatture o appuntamenti, l'eliminazione verrà rifiutata per non lasciarli senza intestatario.`"
      :loading="deleting"
      @confirm="confirmDelete"
      @cancel="deleteOpen = false"
    />
  </div>
</template>

<style scoped>
/* The record card's head: a band of the patient's ink, the monogram set into its lower edge. */
.record-band {
  height: 4.5rem;
  background:
    radial-gradient(120% 140% at 0% 0%, color-mix(in srgb, var(--accent) 22%, transparent), transparent 60%),
    repeating-linear-gradient(135deg, color-mix(in srgb, var(--accent) 7%, transparent) 0 1px, transparent 1px 9px),
    color-mix(in srgb, var(--accent) 6%, var(--surface-raised));
  border-bottom: 1px solid var(--border);
}
.record-avatar {
  padding: 3px;
  background: var(--surface-raised);
  box-shadow: 0 0 0 1px var(--border), 0 4px 10px -4px rgb(40 34 20 / 0.2);
}
</style>

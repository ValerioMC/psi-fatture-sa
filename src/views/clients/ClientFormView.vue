<script setup lang="ts">
/**
 * A patient's record: identity for the invoice, address for the PDF, contacts,
 * and the Sistema Tessera Sanitaria consent. Codice fiscale, P.IVA, CAP,
 * provincia and email are checked when the field is left, and again on save.
 */
import { computed, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Check, FilePlus, Trash2, TriangleAlert } from 'lucide-vue-next'
import { useClientsStore } from '@/stores/clients'
import { errorMessage, useToastStore } from '@/stores/toast'
import { getClient } from '@/api'
import type { ClientType, CreateClientInput } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import FormField from '@/components/ui/FormField.vue'
import FormSection from '@/components/ui/FormSection.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import type { SegmentOption } from '@/components/ui/types'
import { clientDisplayName } from '@/utils/client'
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

onMounted(async () => {
  if (editId === null) return
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

    <div class="mx-auto max-w-[60rem] px-8 pt-6 pb-12">
      <div v-if="loading" class="h-96 rounded-card border border-border bg-surface-raised p-6" role="status" aria-busy="true">
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

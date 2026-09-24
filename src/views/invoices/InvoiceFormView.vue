<script setup lang="ts">
/**
 * Writing an invoice. The form on the left, a live summary on the right that
 * recomputes ENPAP, bollo and ritenuta as you type, with the save button next
 * to the total it saves. Errors are shown on the field they belong to.
 */
import { computed, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Check, Plus, Trash2, TriangleAlert } from 'lucide-vue-next'
import { useInvoicesStore } from '@/stores/invoices'
import { useClientsStore } from '@/stores/clients'
import { useServicesStore } from '@/stores/services'
import { useConfigStore } from '@/stores/config'
import { errorMessage, useToastStore } from '@/stores/toast'
import { getInvoice } from '@/api'
import type { CreateInvoiceInput, InvoiceLineInput, InvoiceStatus, PaymentMethod, UpdateInvoiceInput } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import FormField from '@/components/ui/FormField.vue'
import ComboBox from '@/components/ui/ComboBox.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import type { ComboOption, SegmentOption } from '@/components/ui/types'
import { formatCurrency, todayIso } from '@/utils/format'
import { calculateInvoiceTotals } from '@/utils/tax'
import { clientDisplayName } from '@/utils/client'
import { INVOICE_STATUS, INVOICE_STATUS_ORDER, PAYMENT_METHOD_LABEL, PAYMENT_METHOD_ORDER } from '@/utils/labels'

const route = useRoute()
const router = useRouter()
const invoicesStore = useInvoicesStore()
const clientsStore = useClientsStore()
const servicesStore = useServicesStore()
const configStore = useConfigStore()
const toast = useToastStore()

const editId = route.params.id ? Number(route.params.id) : null
const isEdit = editId !== null
const loading = ref(isEdit)
const loadError = ref<string | null>(null)
const saving = ref(false)
const nextNumber = ref<string | null>(null)

interface FormLine extends InvoiceLineInput {
  key: number
}

let lineKey = 0
function blankLine(): FormLine {
  return { key: lineKey++, service_id: undefined, description: '', quantity: 1, unit_price: 0, vat_rate: 0 }
}

const form = reactive({
  client_id: null as number | null,
  invoice_number: '',
  issue_date: todayIso(),
  due_date: '',
  status: 'draft' as InvoiceStatus,
  payment_method: 'bonifico' as PaymentMethod,
  notes: '',
  apply_enpap: true,
  paid_date: '',
  lines: [blankLine()] as FormLine[],
})

type FieldKey = 'client' | 'invoice_number' | 'issue_date' | 'due_date' | `line-${number}`
const errors = reactive<Partial<Record<FieldKey, string>>>({})

const clientOptions = computed<ComboOption[]>(() =>
  clientsStore.clients.map((client) => ({ value: client.id, label: clientDisplayName(client), detail: client.fiscal_code })),
)

const STATUS_OPTIONS: SegmentOption<InvoiceStatus>[] = INVOICE_STATUS_ORDER.map((value) => ({
  value,
  label: INVOICE_STATUS[value].label,
  tone: INVOICE_STATUS[value].tone,
}))

const taxRegime = computed(() => configStore.config?.tax_regime ?? 'forfettario')
const totals = computed(() => calculateInvoiceTotals(form.lines, taxRegime.value, form.apply_enpap))

function addLine(): void {
  form.lines.push(blankLine())
}

function removeLine(key: number): void {
  form.lines = form.lines.filter((line) => line.key !== key)
}

function applyService(line: FormLine): void {
  const service = servicesStore.services.find((candidate) => candidate.id === line.service_id)
  if (service === undefined) return
  line.description = service.name
  line.unit_price = service.default_price
  line.vat_rate = service.vat_rate
}

onMounted(async () => {
  try {
    await Promise.all([
      clientsStore.fetchClients(),
      servicesStore.fetchServices(false),
      configStore.isConfigured ? Promise.resolve() : configStore.loadConfig(),
    ])
    const presetClient = Number(route.query.client)
    if (!isEdit && Number.isInteger(presetClient) && presetClient > 0) form.client_id = presetClient
    if (!isEdit) {
      nextNumber.value = await invoicesStore.nextNumber(new Date().getFullYear())
      return
    }
    const invoice = await getInvoice(editId)
    Object.assign(form, {
      client_id: invoice.client_id,
      invoice_number: invoice.invoice_number,
      issue_date: invoice.issue_date,
      due_date: invoice.due_date ?? '',
      status: invoice.status,
      payment_method: invoice.payment_method,
      notes: invoice.notes,
      apply_enpap: invoice.apply_enpap,
      paid_date: invoice.paid_date ?? '',
      lines: invoice.lines.map((line) => ({
        key: lineKey++,
        service_id: line.service_id,
        description: line.description,
        quantity: line.quantity,
        unit_price: line.unit_price,
        vat_rate: line.vat_rate,
      })),
    })
  } catch (error) {
    loadError.value = errorMessage(error)
  } finally {
    loading.value = false
  }
})

function validate(): boolean {
  for (const key of Object.keys(errors) as FieldKey[]) delete errors[key]
  if (form.client_id === null) errors.client = 'Scegli il paziente a cui intestare la fattura.'
  if (isEdit && form.invoice_number.trim() !== '') {
    const number = Number(form.invoice_number.trim())
    if (!Number.isInteger(number) || number < 1) errors.invoice_number = 'Un numero intero maggiore di zero.'
  }
  if (!form.issue_date) errors.issue_date = 'Indica la data di emissione.'
  if (form.due_date && form.due_date < form.issue_date) errors.due_date = 'La scadenza non può precedere l’emissione.'
  form.lines.forEach((line, index) => {
    const key = `line-${index}` as const
    if (!line.description.trim()) errors[key] = 'Serve una descrizione.'
    else if (!Number.isInteger(line.quantity) || line.quantity < 1) errors[key] = 'La quantità è un intero da 1 in su.'
    else if (!Number.isFinite(line.unit_price) || line.unit_price < 0) errors[key] = 'Prezzo non valido.'
    else if (!Number.isFinite(line.vat_rate) || line.vat_rate < 0 || line.vat_rate > 100) errors[key] = 'IVA tra 0 e 100.'
  })
  return Object.keys(errors).length === 0
}

async function onSubmit(): Promise<void> {
  if (form.lines.length === 0) {
    toast.notifyError('Aggiungi almeno una riga alla fattura.')
    return
  }
  if (!validate() || form.client_id === null) return

  const lines: InvoiceLineInput[] = form.lines.map(({ key: _key, ...line }) => ({ ...line, service_id: line.service_id || undefined }))
  const base: CreateInvoiceInput = {
    client_id: form.client_id,
    issue_date: form.issue_date,
    due_date: form.due_date || undefined,
    status: form.status,
    payment_method: form.payment_method,
    notes: form.notes,
    apply_enpap: form.apply_enpap,
    lines,
  }

  saving.value = true
  try {
    if (editId !== null) {
      const input: UpdateInvoiceInput = {
        ...base,
        id: editId,
        invoice_number: form.invoice_number.trim() || undefined,
        paid_date: form.status === 'paid' ? form.paid_date || undefined : undefined,
      }
      const saved = await invoicesStore.editInvoice(input)
      toast.notify(`Fattura N. ${saved.invoice_number} aggiornata`)
      void router.push(`/invoices/${saved.id}`)
    } else {
      const saved = await invoicesStore.addInvoice(base)
      toast.notify(`Fattura N. ${saved.invoice_number} creata`)
      void router.push(`/invoices/${saved.id}`)
    }
  } catch (error) {
    toast.notifyError(error, 'Salvataggio non riuscito')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div>
    <PageHeader
      :title="isEdit ? `Modifica fattura N. ${form.invoice_number}` : 'Nuova fattura'"
      :back="isEdit ? { to: `/invoices/${editId}`, label: 'Fattura' } : { to: '/invoices', label: 'Fatture' }"
    />

    <div class="mx-auto max-w-[72rem] px-8 pt-6 pb-12">
      <div v-if="loading" class="h-96 rounded-card border border-border bg-surface-raised p-6" role="status" aria-busy="true">
        <span class="sr-only">Caricamento della fattura</span>
        <div class="skeleton h-4 w-1/3" />
      </div>

      <EmptyState v-else-if="loadError" :icon="TriangleAlert" title="Non è stato possibile aprire la fattura" :description="loadError">
        <AppButton to="/invoices">Torna alle fatture</AppButton>
      </EmptyState>

      <form v-else class="grid grid-cols-1 items-start gap-5 lg:grid-cols-[1fr_19rem]" novalidate @submit.prevent="onSubmit">
        <div class="space-y-5">
          <!-- ── Who and when ─────────────────────────────────────────────── -->
          <AppCard class="settle">
            <h2 class="mb-5 text-lg font-medium text-text">Intestazione</h2>
            <div class="grid grid-cols-2 gap-x-4 gap-y-5">
              <FormField v-slot="{ id, invalid, describedBy }" class="col-span-2" label="Paziente" required :error="errors.client">
                <ComboBox
                  :id="id"
                  v-model="form.client_id"
                  :options="clientOptions"
                  placeholder="Cerca per cognome o codice fiscale"
                  :invalid="invalid"
                  :described-by="describedBy"
                />
              </FormField>

              <FormField v-slot="{ id, invalid, describedBy }" label="Data di emissione" required :error="errors.issue_date">
                <input :id="id" v-model="form.issue_date" type="date" class="field" :aria-invalid="invalid" :aria-describedby="describedBy" />
              </FormField>
              <FormField v-slot="{ id, invalid, describedBy }" label="Scadenza" optional :error="errors.due_date">
                <input :id="id" v-model="form.due_date" type="date" class="field" :aria-invalid="invalid" :aria-describedby="describedBy" />
              </FormField>

              <FormField v-slot="{ id }" label="Metodo di pagamento">
                <select :id="id" v-model="form.payment_method" class="field">
                  <option v-for="method in PAYMENT_METHOD_ORDER" :key="method" :value="method">{{ PAYMENT_METHOD_LABEL[method] }}</option>
                </select>
              </FormField>
              <FormField
                v-if="isEdit"
                v-slot="{ id, invalid, describedBy }"
                label="Numero"
                hint="Cambialo solo per correggere la numerazione: deve restare unico nell'anno."
                :error="errors.invoice_number"
              >
                <input :id="id" v-model="form.invoice_number" type="text" inputmode="numeric" class="field tabular" :aria-invalid="invalid" :aria-describedby="describedBy" />
              </FormField>
              <div v-else class="self-end pb-2 text-sm text-text-subtle">
                <template v-if="nextNumber">Riceverà il numero <span class="tabular font-medium text-text-muted">{{ nextNumber }}</span> al salvataggio.</template>
              </div>
            </div>
          </AppCard>

          <!-- ── Lines ──────────────────────────────────────────────────────── -->
          <AppCard class="settle" :padded="false" style="--settle: 1">
            <div class="flex items-center justify-between px-5 pt-5 pb-4">
              <h2 class="text-lg font-medium text-text">Prestazioni</h2>
              <AppButton size="sm" :icon="Plus" @click="addLine">Aggiungi riga</AppButton>
            </div>
            <div class="grid grid-cols-[minmax(0,1fr)_4.5rem_7rem_4.5rem_2rem] gap-2 border-y border-border bg-surface px-5 py-2 text-xs font-medium text-text-subtle" aria-hidden="true">
              <span>Descrizione</span><span class="text-right">Qtà</span><span class="text-right">Prezzo €</span><span class="text-right">IVA %</span><span />
            </div>
            <TransitionGroup name="list" tag="div" class="relative">
              <div v-for="(line, index) in form.lines" :key="line.key" class="border-b border-border px-5 py-3 last:border-b-0">
                <div class="grid grid-cols-[minmax(0,1fr)_4.5rem_7rem_4.5rem_2rem] items-start gap-2">
                  <div class="space-y-2">
                    <select
                      v-model="line.service_id"
                      class="field field-sm text-text-muted"
                      :aria-label="`Prestazione dal catalogo, riga ${index + 1}`"
                      @change="applyService(line)"
                    >
                      <option :value="undefined">Dal catalogo…</option>
                      <option v-for="service in servicesStore.services" :key="service.id" :value="service.id">{{ service.name }}</option>
                    </select>
                    <input
                      v-model="line.description"
                      type="text"
                      class="field field-sm"
                      placeholder="Descrizione della prestazione"
                      :aria-label="`Descrizione, riga ${index + 1}`"
                      :aria-invalid="errors[`line-${index}`] ? true : undefined"
                    />
                  </div>
                  <input v-model.number="line.quantity" type="number" min="1" step="1" class="field field-sm tabular text-right" :aria-label="`Quantità, riga ${index + 1}`" />
                  <input v-model.number="line.unit_price" type="number" min="0" step="0.01" class="field field-sm tabular text-right" :aria-label="`Prezzo unitario, riga ${index + 1}`" />
                  <input v-model.number="line.vat_rate" type="number" min="0" max="100" step="1" class="field field-sm tabular text-right" :aria-label="`Aliquota IVA, riga ${index + 1}`" />
                  <AppButton
                    variant="danger-quiet"
                    size="sm"
                    :icon="Trash2"
                    :label="`Rimuovi riga ${index + 1}`"
                    :disabled="form.lines.length === 1"
                    @click="removeLine(line.key)"
                  />
                </div>
                <p v-if="errors[`line-${index}`]" class="mt-1.5 text-xs text-danger" role="alert">{{ errors[`line-${index}`] }}</p>
              </div>
            </TransitionGroup>
          </AppCard>

          <!-- ── Status and notes ─────────────────────────────────────────── -->
          <AppCard class="settle space-y-5" style="--settle: 2">
            <div>
              <p class="mb-1.5 text-sm font-medium text-text-muted">Stato</p>
              <SegmentedControl v-model="form.status" :options="STATUS_OPTIONS" label="Stato della fattura" />
            </div>
            <FormField v-if="form.status === 'paid'" v-slot="{ id, describedBy }" label="Pagata il" hint="Se la lasci vuota vale la data di oggi." class="max-w-60">
              <input :id="id" v-model="form.paid_date" type="date" class="field" :aria-describedby="describedBy" />
            </FormField>
            <ToggleSwitch v-model="form.apply_enpap" label="Contributo integrativo ENPAP 2%" description="Addebitato al paziente sull'imponibile, come previsto per gli psicologi." />
            <FormField v-slot="{ id }" label="Note" optional>
              <textarea :id="id" v-model="form.notes" rows="2" class="field" placeholder="Compaiono in fondo alla fattura" />
            </FormField>
          </AppCard>
        </div>

        <!-- ── Live summary and save ─────────────────────────────────────── -->
        <aside class="settle lg:sticky lg:top-30" style="--settle: 1">
          <AppCard>
            <h2 class="text-lg font-medium text-text">Riepilogo</h2>
            <dl class="mt-4 space-y-2 text-base">
              <div class="flex justify-between"><dt class="text-text-muted">Imponibile</dt><dd class="tabular text-text">{{ formatCurrency(totals.total_net) }}</dd></div>
              <div v-if="totals.total_tax > 0" class="flex justify-between"><dt class="text-text-muted">IVA</dt><dd class="tabular text-text">{{ formatCurrency(totals.total_tax) }}</dd></div>
              <div v-if="form.apply_enpap" class="flex justify-between"><dt class="text-text-muted">ENPAP 2%</dt><dd class="tabular text-text">{{ formatCurrency(totals.contributo_enpap) }}</dd></div>
              <div v-if="totals.marca_da_bollo > 0" class="flex justify-between"><dt class="text-text-muted">Marca da bollo</dt><dd class="tabular text-text">{{ formatCurrency(totals.marca_da_bollo) }}</dd></div>
              <div v-if="totals.ritenuta_acconto > 0" class="flex justify-between"><dt class="text-text-muted">Ritenuta 20%</dt><dd class="tabular text-text">−{{ formatCurrency(totals.ritenuta_acconto) }}</dd></div>
            </dl>
            <div class="mt-4 border-t border-border pt-4">
              <p class="label-quiet">Totale dovuto</p>
              <p class="mt-0.5 text-[2rem] font-semibold leading-tight tracking-[-0.02em] text-text" aria-live="polite">{{ formatCurrency(totals.total_due) }}</p>
            </div>
            <AppButton type="submit" variant="primary" class="mt-5" block :icon="Check" :loading="saving">
              {{ isEdit ? 'Salva modifiche' : 'Crea fattura' }}
            </AppButton>
            <p v-if="Object.keys(errors).length > 0" class="mt-3 text-center text-xs text-danger" role="alert">Controlla i campi segnalati.</p>
          </AppCard>
          <p class="mt-3 px-1 text-xs text-text-subtle">
            {{ taxRegime === 'forfettario' ? 'Regime forfettario: niente IVA né ritenuta; bollo da 2 € sopra i 77,47 €.' : 'Regime ordinario: la ritenuta d’acconto del 20% è calcolata in automatico.' }}
          </p>
        </aside>
      </form>
    </div>
  </div>
</template>

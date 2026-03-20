<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Plus, Trash2 } from 'lucide-vue-next'
import { useInvoicesStore } from '@/stores/invoices'
import { useClientsStore } from '@/stores/clients'
import { useServicesStore } from '@/stores/services'
import { useConfigStore } from '@/stores/config'
import { getInvoice } from '@/api'
import type {
  CreateInvoiceInput,
  UpdateInvoiceInput,
  InvoiceLineInput,
  InvoiceStatus,
  PaymentMethod,
} from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import { formatCurrency } from '@/utils/format'
import { calculateInvoiceTotals } from '@/utils/tax'

const route = useRoute()
const router = useRouter()
const invoicesStore = useInvoicesStore()
const clientsStore = useClientsStore()
const servicesStore = useServicesStore()
const configStore = useConfigStore()

const editId = computed(() =>
  route.params.id ? Number(route.params.id) : null,
)
const isEdit = computed(() => editId.value !== null)
const loading = ref(false)
const saving = ref(false)
const error = ref<string | null>(null)

const TODAY = new Date().toISOString().slice(0, 10)

interface FormLine extends InvoiceLineInput {
  _key: number
}

let lineKeyCounter = 0
function newLine(): FormLine {
  return {
    _key: lineKeyCounter++,
    service_id: undefined,
    description: '',
    quantity: 1,
    unit_price: 0,
    vat_rate: 0,
  }
}

const form = reactive({
  client_id: 0,
  issue_date: TODAY,
  due_date: '' as string | undefined,
  status: 'draft' as InvoiceStatus,
  payment_method: 'bonifico' as PaymentMethod,
  notes: '',
  apply_enpap: true,
  lines: [newLine()] as FormLine[],
})

const PAYMENT_METHODS: Array<{ value: PaymentMethod; label: string }> = [
  { value: 'bonifico', label: 'Bonifico bancario' },
  { value: 'contanti', label: 'Contanti' },
  { value: 'pos', label: 'POS / Carta' },
  { value: 'altro', label: 'Altro' },
]

const STATUS_OPTIONS: Array<{ value: InvoiceStatus; label: string }> = [
  { value: 'draft', label: 'Bozza' },
  { value: 'issued', label: 'Emessa' },
  { value: 'paid', label: 'Pagata' },
  { value: 'cancelled', label: 'Annullata' },
]

const taxRegime = computed(() => configStore.config?.tax_regime ?? 'forfettario')

const totals = computed(() =>
  calculateInvoiceTotals(form.lines, taxRegime.value, form.apply_enpap),
)

function addLine() {
  form.lines.push(newLine())
}

function removeLine(key: number) {
  const idx = form.lines.findIndex((l) => l._key === key)
  if (idx !== -1) form.lines.splice(idx, 1)
}

function onServiceSelect(line: FormLine) {
  if (!line.service_id) return
  const service = servicesStore.services.find((s) => s.id === line.service_id)
  if (!service) return
  line.description = service.name
  line.unit_price = service.default_price
  line.vat_rate = service.vat_rate
}

onMounted(async () => {
  await Promise.all([
    clientsStore.fetchClients(),
    servicesStore.fetchServices(false),
    configStore.isConfigured ? Promise.resolve() : configStore.loadConfig(),
  ])

  if (!isEdit.value) return
  loading.value = true
  try {
    const invoice = await getInvoice(editId.value!)
    form.client_id = invoice.client_id
    form.issue_date = invoice.issue_date
    form.due_date = invoice.due_date ?? ''
    form.status = invoice.status
    form.payment_method = invoice.payment_method
    form.notes = invoice.notes
    form.apply_enpap = invoice.apply_enpap
    form.lines = invoice.lines.map((l) => ({
      _key: lineKeyCounter++,
      service_id: l.service_id,
      description: l.description,
      quantity: l.quantity,
      unit_price: l.unit_price,
      vat_rate: l.vat_rate,
    }))
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})

async function onSubmit() {
  if (form.lines.length === 0) {
    error.value = 'Aggiungi almeno una riga alla fattura.'
    return
  }

  saving.value = true
  error.value = null

  const lineInputs: InvoiceLineInput[] = form.lines.map(({ _key, ...rest }) => ({
    ...rest,
    service_id: rest.service_id || undefined,
  }))

  try {
    if (isEdit.value) {
      const input: UpdateInvoiceInput = {
        id: editId.value!,
        client_id: form.client_id,
        issue_date: form.issue_date,
        due_date: form.due_date || undefined,
        status: form.status,
        payment_method: form.payment_method,
        notes: form.notes,
        apply_enpap: form.apply_enpap,
        lines: lineInputs,
      }
      await invoicesStore.editInvoice(input)
    } else {
      const input: CreateInvoiceInput = {
        client_id: form.client_id,
        issue_date: form.issue_date,
        due_date: form.due_date || undefined,
        status: form.status,
        payment_method: form.payment_method,
        notes: form.notes,
        apply_enpap: form.apply_enpap,
        lines: lineInputs,
      }
      await invoicesStore.addInvoice(input)
    }
    router.push('/invoices')
  } catch (e) {
    error.value = String(e)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="p-8">
    <PageHeader
      :title="isEdit ? 'Modifica Fattura' : 'Nuova Fattura'"
      :subtitle="isEdit ? 'Aggiorna i dati della fattura.' : 'Crea una nuova fattura.'"
    />

    <div v-if="loading" class="text-sm text-gray-400">Caricamento...</div>

    <form v-else class="grid grid-cols-3 gap-6" @submit.prevent="onSubmit">
      <div class="col-span-2 space-y-6">
        <!-- Header data -->
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
          <h2 class="text-base font-semibold text-gray-900 mb-4">Dati fattura</h2>
          <div class="grid grid-cols-2 gap-4">
            <div class="col-span-2">
              <label class="text-sm font-medium text-gray-700 block mb-1">Cliente *</label>
              <select
                v-model.number="form.client_id"
                required
                class="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option :value="0" disabled>— Seleziona cliente —</option>
                <option
                  v-for="client in clientsStore.clients"
                  :key="client.id"
                  :value="client.id"
                >
                  {{ client.last_name }} {{ client.first_name }}
                </option>
              </select>
            </div>
            <div>
              <label class="text-sm font-medium text-gray-700 block mb-1">Data emissione *</label>
              <input
                v-model="form.issue_date"
                type="date"
                required
                class="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label class="text-sm font-medium text-gray-700 block mb-1">Data scadenza</label>
              <input
                v-model="form.due_date"
                type="date"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label class="text-sm font-medium text-gray-700 block mb-1">Metodo di pagamento</label>
              <select
                v-model="form.payment_method"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option v-for="opt in PAYMENT_METHODS" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
            </div>
            <div>
              <label class="text-sm font-medium text-gray-700 block mb-1">Stato</label>
              <select
                v-model="form.status"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option v-for="opt in STATUS_OPTIONS" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
            </div>
            <div class="col-span-2">
              <label class="text-sm font-medium text-gray-700 block mb-1">Note</label>
              <textarea
                v-model="form.notes"
                rows="2"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
              />
            </div>
            <div class="col-span-2">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="form.apply_enpap"
                  type="checkbox"
                  class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <span class="text-sm text-gray-700">Applica contributo ENPAP (2%)</span>
              </label>
            </div>
          </div>
        </div>

        <!-- Invoice lines -->
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-base font-semibold text-gray-900">Righe fattura</h2>
            <button
              type="button"
              class="border border-gray-300 text-gray-700 hover:bg-gray-50 px-3 py-1.5 rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors"
              @click="addLine"
            >
              <Plus class="w-3.5 h-3.5" />
              Aggiungi riga
            </button>
          </div>

          <div class="space-y-3">
            <div
              v-for="(line, idx) in form.lines"
              :key="line._key"
              class="grid gap-2 items-end pb-3 border-b border-gray-50 last:border-0"
              style="grid-template-columns: 2fr 3fr 1fr 1fr 1fr auto"
            >
              <!-- Servizio -->
              <div>
                <label v-if="idx === 0" class="text-xs font-medium text-gray-500 block mb-1">Servizio</label>
                <select
                  v-model.number="line.service_id"
                  class="w-full border border-gray-300 rounded-lg px-2 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                  @change="onServiceSelect(line)"
                >
                  <option :value="undefined">— nessuno —</option>
                  <option v-for="s in servicesStore.services" :key="s.id" :value="s.id">
                    {{ s.name }}
                  </option>
                </select>
              </div>
              <!-- Descrizione -->
              <div>
                <label v-if="idx === 0" class="text-xs font-medium text-gray-500 block mb-1">Descrizione *</label>
                <input
                  v-model="line.description"
                  type="text"
                  required
                  class="w-full border border-gray-300 rounded-lg px-2 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <!-- Quantità -->
              <div>
                <label v-if="idx === 0" class="text-xs font-medium text-gray-500 block mb-1">Qtà</label>
                <input
                  v-model.number="line.quantity"
                  type="number"
                  min="1"
                  step="1"
                  required
                  class="w-full border border-gray-300 rounded-lg px-2 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <!-- Prezzo unitario -->
              <div>
                <label v-if="idx === 0" class="text-xs font-medium text-gray-500 block mb-1">Prezzo (€)</label>
                <input
                  v-model.number="line.unit_price"
                  type="number"
                  min="0"
                  step="0.01"
                  required
                  class="w-full border border-gray-300 rounded-lg px-2 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <!-- IVA -->
              <div>
                <label v-if="idx === 0" class="text-xs font-medium text-gray-500 block mb-1">IVA %</label>
                <input
                  v-model.number="line.vat_rate"
                  type="number"
                  min="0"
                  max="100"
                  step="1"
                  class="w-full border border-gray-300 rounded-lg px-2 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <!-- Remove -->
              <div>
                <div v-if="idx === 0" class="h-5 mb-1" />
                <button
                  type="button"
                  class="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                  :disabled="form.lines.length === 1"
                  @click="removeLine(line._key)"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Totals panel -->
      <div class="col-span-1">
        <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-5 sticky top-6">
          <h2 class="text-sm font-semibold text-gray-900 mb-4">Riepilogo</h2>
          <div class="space-y-2 text-sm">
            <div class="flex justify-between text-gray-500">
              <span>Totale netto</span>
              <span>{{ formatCurrency(totals.total_net) }}</span>
            </div>
            <div class="flex justify-between text-gray-500">
              <span>IVA totale</span>
              <span>{{ formatCurrency(totals.total_tax) }}</span>
            </div>
            <div v-if="form.apply_enpap" class="flex justify-between text-gray-500">
              <span>Contributo ENPAP (2%)</span>
              <span>+ {{ formatCurrency(totals.contributo_enpap) }}</span>
            </div>
            <div v-if="totals.ritenuta_acconto > 0" class="flex justify-between text-gray-500">
              <span>Ritenuta d'acconto (20%)</span>
              <span class="text-red-600">- {{ formatCurrency(totals.ritenuta_acconto) }}</span>
            </div>
            <div v-if="totals.marca_da_bollo > 0" class="flex justify-between text-gray-500">
              <span>Marca da bollo</span>
              <span>+ {{ formatCurrency(totals.marca_da_bollo) }}</span>
            </div>
            <div class="border-t border-gray-100 pt-2 mt-2 flex justify-between font-semibold text-gray-900">
              <span>Totale dovuto</span>
              <span>{{ formatCurrency(totals.total_due) }}</span>
            </div>
          </div>

          <div v-if="error" class="mt-4 rounded-lg bg-red-50 border border-red-200 px-3 py-2 text-xs text-red-700">
            {{ error }}
          </div>

          <div class="mt-5 space-y-2">
            <button
              type="submit"
              :disabled="saving"
              class="w-full bg-blue-600 text-white hover:bg-blue-700 px-4 py-2 rounded-lg text-sm font-medium transition-colors disabled:opacity-60"
            >
              {{ saving ? 'Salvataggio...' : isEdit ? 'Aggiorna fattura' : 'Crea fattura' }}
            </button>
            <button
              type="button"
              class="w-full border border-gray-300 text-gray-700 hover:bg-gray-50 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
              @click="router.push('/invoices')"
            >
              Annulla
            </button>
          </div>
        </div>
      </div>
    </form>
  </div>
</template>

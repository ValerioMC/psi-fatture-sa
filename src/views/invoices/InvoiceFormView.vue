<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Plus, Trash2, FileText, User, Calendar, CreditCard, List, Check, X, ChevronLeft } from 'lucide-vue-next'
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

const STATUS_OPTIONS: Array<{ value: InvoiceStatus; label: string; dot: string }> = [
  { value: 'draft',     label: 'Bozza',     dot: 'bg-warm-400' },
  { value: 'issued',    label: 'Emessa',    dot: 'bg-ocean-500' },
  { value: 'paid',      label: 'Pagata',    dot: 'bg-sage-500' },
  { value: 'cancelled', label: 'Annullata', dot: 'bg-sage-300' },
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
    <div class="max-w-4xl mx-auto">
      <PageHeader
        :title="isEdit ? 'Modifica Fattura' : 'Nuova Fattura'"
        :subtitle="isEdit ? 'Aggiorna i dati della fattura.' : 'Crea una nuova fattura.'"
      >
        <button
          type="button"
          class="flex items-center gap-1.5 text-sm text-sage-500 hover:text-sage-700 transition-colors cursor-pointer"
          @click="router.push('/invoices')"
        >
          <ChevronLeft class="w-4 h-4" />
          Torna alla lista
        </button>
      </PageHeader>

      <!-- Loading -->
      <div v-if="loading" class="flex flex-col items-center justify-center py-20 gap-3">
        <div class="w-8 h-8 rounded-full border-2 border-sage-200 border-t-sage-500 animate-spin" />
        <p class="text-sm text-sage-400">Caricamento fattura…</p>
      </div>

      <form v-else class="space-y-5" @submit.prevent="onSubmit">

        <!-- Dati fattura -->
        <div class="glass-card rounded-2xl p-5 animate-in">
          <div class="flex items-center gap-2.5 mb-5">
            <div class="w-7 h-7 rounded-lg flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #059669, #047857)">
              <FileText class="w-3.5 h-3.5 text-white" />
            </div>
            <h2 class="text-xs font-semibold text-sage-500 uppercase tracking-wider">Dati fattura</h2>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <!-- Cliente -->
            <div class="col-span-2">
              <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">
                <span class="flex items-center gap-1.5">
                  <User class="w-3 h-3" />
                  Cliente *
                </span>
              </label>
              <select
                v-model.number="form.client_id"
                required
                class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow cursor-pointer appearance-none"
              >
                <option :value="0" disabled>— Seleziona paziente —</option>
                <option
                  v-for="client in clientsStore.clients"
                  :key="client.id"
                  :value="client.id"
                >
                  {{ client.last_name }} {{ client.first_name }}
                </option>
              </select>
            </div>

            <!-- Date -->
            <div>
              <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">
                <span class="flex items-center gap-1.5">
                  <Calendar class="w-3 h-3" />
                  Data emissione *
                </span>
              </label>
              <input
                v-model="form.issue_date"
                type="date"
                required
                class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
              />
            </div>
            <div>
              <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Data scadenza</label>
              <input
                v-model="form.due_date"
                type="date"
                class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
              />
            </div>

            <!-- Payment method -->
            <div>
              <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">
                <span class="flex items-center gap-1.5">
                  <CreditCard class="w-3 h-3" />
                  Metodo di pagamento
                </span>
              </label>
              <select
                v-model="form.payment_method"
                class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow cursor-pointer appearance-none"
              >
                <option v-for="opt in PAYMENT_METHODS" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
            </div>

            <!-- Status -->
            <div>
              <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Stato</label>
              <div class="flex rounded-xl border border-sage-200 p-1 bg-sage-50/50 gap-1">
                <button
                  v-for="opt in STATUS_OPTIONS"
                  :key="opt.value"
                  type="button"
                  class="flex-1 flex items-center justify-center gap-1.5 py-1.5 text-xs font-medium rounded-lg transition-all duration-200 cursor-pointer"
                  :class="form.status === opt.value
                    ? 'bg-white shadow-sm text-sage-900'
                    : 'text-sage-400 hover:text-sage-600'"
                  @click="form.status = opt.value"
                >
                  <span class="w-1.5 h-1.5 rounded-full shrink-0" :class="opt.dot" />
                  {{ opt.label }}
                </button>
              </div>
            </div>

            <!-- Notes -->
            <div class="col-span-2">
              <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Note</label>
              <textarea
                v-model="form.notes"
                rows="2"
                placeholder="Note aggiuntive sulla fattura…"
                class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 placeholder-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow resize-none"
              />
            </div>

            <!-- ENPAP toggle -->
            <div class="col-span-2">
              <label class="flex items-center gap-3 cursor-pointer p-3.5 rounded-xl border border-sage-100/60 hover:bg-sage-50/60 transition-colors">
                <div
                  class="w-9 h-5 rounded-full transition-colors duration-200 relative shrink-0"
                  :class="form.apply_enpap ? 'bg-sage-500' : 'bg-sage-200'"
                >
                  <div
                    class="absolute top-[3px] w-3.5 h-3.5 rounded-full bg-white shadow-sm transition-transform duration-200"
                    :class="form.apply_enpap ? 'translate-x-[17px]' : 'translate-x-[3px]'"
                  />
                </div>
                <input v-model="form.apply_enpap" type="checkbox" class="sr-only" />
                <div>
                  <p class="text-sm font-medium text-sage-800">Applica contributo ENPAP (2%)</p>
                  <p class="text-xs text-sage-400">Contributo previdenziale sulla quota netta</p>
                </div>
              </label>
            </div>
          </div>
        </div>

        <!-- Righe fattura -->
        <div class="glass-card rounded-2xl p-5 animate-in-d1">
          <div class="flex items-center justify-between mb-5">
            <div class="flex items-center gap-2.5">
              <div class="w-7 h-7 rounded-lg flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #4f46e5, #4338ca)">
                <List class="w-3.5 h-3.5 text-white" />
              </div>
              <h2 class="text-xs font-semibold text-sage-500 uppercase tracking-wider">Righe fattura</h2>
            </div>
            <button
              type="button"
              class="flex items-center gap-1.5 text-xs font-medium text-sage-600 hover:text-sage-800 border border-sage-200 hover:bg-sage-50 px-3 py-1.5 rounded-lg transition-all cursor-pointer"
              @click="addLine"
            >
              <Plus class="w-3.5 h-3.5" />
              Aggiungi riga
            </button>
          </div>

          <!-- Column headers -->
          <div
            class="grid gap-2 mb-2 px-3"
            style="grid-template-columns: 2fr 3fr 1fr 1fr 1fr auto"
          >
            <p class="text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Servizio</p>
            <p class="text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Descrizione *</p>
            <p class="text-[10px] font-semibold text-sage-400 uppercase tracking-wider text-center">Qtà</p>
            <p class="text-[10px] font-semibold text-sage-400 uppercase tracking-wider text-right">Prezzo (€)</p>
            <p class="text-[10px] font-semibold text-sage-400 uppercase tracking-wider text-right">IVA %</p>
            <p class="w-7" />
          </div>

          <TransitionGroup name="line" tag="div" class="space-y-2">
            <div
              v-for="line in form.lines"
              :key="line._key"
              class="grid gap-2 items-center rounded-xl border border-sage-100 bg-white/50 px-3 py-2.5"
              style="grid-template-columns: 2fr 3fr 1fr 1fr 1fr auto"
            >
              <!-- Servizio -->
              <select
                v-model.number="line.service_id"
                class="w-full border border-sage-200 rounded-lg px-2.5 py-1.5 text-xs text-sage-800 focus:outline-none focus:ring-2 focus:ring-sage-400/40 bg-white/80 cursor-pointer appearance-none"
                @change="onServiceSelect(line)"
              >
                <option :value="undefined">— nessuno —</option>
                <option v-for="s in servicesStore.services" :key="s.id" :value="s.id">
                  {{ s.name }}
                </option>
              </select>

              <!-- Descrizione -->
              <input
                v-model="line.description"
                type="text"
                required
                placeholder="Descrizione prestazione"
                class="w-full border border-sage-200 rounded-lg px-2.5 py-1.5 text-xs text-sage-900 placeholder-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 bg-white/80"
              />

              <!-- Quantità -->
              <input
                v-model.number="line.quantity"
                type="number"
                min="1"
                step="1"
                required
                class="w-full border border-sage-200 rounded-lg px-2.5 py-1.5 text-xs text-sage-900 text-center focus:outline-none focus:ring-2 focus:ring-sage-400/40 bg-white/80 tabular-nums"
              />

              <!-- Prezzo -->
              <input
                v-model.number="line.unit_price"
                type="number"
                min="0"
                step="0.01"
                required
                class="w-full border border-sage-200 rounded-lg px-2.5 py-1.5 text-xs text-sage-900 text-right focus:outline-none focus:ring-2 focus:ring-sage-400/40 bg-white/80 tabular-nums"
              />

              <!-- IVA -->
              <input
                v-model.number="line.vat_rate"
                type="number"
                min="0"
                max="100"
                step="1"
                class="w-full border border-sage-200 rounded-lg px-2.5 py-1.5 text-xs text-sage-900 text-right focus:outline-none focus:ring-2 focus:ring-sage-400/40 bg-white/80 tabular-nums"
              />

              <!-- Remove -->
              <button
                type="button"
                class="w-7 h-7 flex items-center justify-center rounded-lg transition-all duration-150 cursor-pointer"
                :class="form.lines.length === 1
                  ? 'text-sage-200 cursor-not-allowed'
                  : 'text-sage-300 hover:text-red-500 hover:bg-red-50'"
                :disabled="form.lines.length === 1"
                @click="removeLine(line._key)"
              >
                <Trash2 class="w-3.5 h-3.5" />
              </button>
            </div>
          </TransitionGroup>
        </div>

        <!-- Riepilogo + azioni -->
        <div class="glass-card rounded-2xl p-5 animate-in-d2">
          <!-- Totali in riga -->
          <div class="flex flex-wrap items-end gap-x-6 gap-y-3 pb-4 mb-4 border-b border-sage-100">
            <div>
              <p class="text-[10px] text-sage-400 uppercase tracking-wider mb-0.5">Netto</p>
              <p class="text-sm font-semibold text-sage-800 tabular-nums">{{ formatCurrency(totals.total_net) }}</p>
            </div>
            <div>
              <p class="text-[10px] text-sage-400 uppercase tracking-wider mb-0.5">IVA</p>
              <p class="text-sm font-semibold text-sage-800 tabular-nums">{{ formatCurrency(totals.total_tax) }}</p>
            </div>
            <div v-if="form.apply_enpap">
              <p class="text-[10px] text-sage-400 uppercase tracking-wider mb-0.5">ENPAP 2%</p>
              <p class="text-sm font-semibold text-sage-800 tabular-nums">+ {{ formatCurrency(totals.contributo_enpap) }}</p>
            </div>
            <div v-if="totals.ritenuta_acconto > 0">
              <p class="text-[10px] text-sage-400 uppercase tracking-wider mb-0.5">Ritenuta 20%</p>
              <p class="text-sm font-semibold text-red-500 tabular-nums">− {{ formatCurrency(totals.ritenuta_acconto) }}</p>
            </div>
            <div v-if="totals.marca_da_bollo > 0">
              <p class="text-[10px] text-sage-400 uppercase tracking-wider mb-0.5">Bollo</p>
              <p class="text-sm font-semibold text-sage-800 tabular-nums">+ {{ formatCurrency(totals.marca_da_bollo) }}</p>
            </div>
            <div class="ml-auto text-right">
              <p class="text-[10px] text-sage-400 uppercase tracking-wider mb-0.5">Totale dovuto</p>
              <p class="text-2xl font-bold text-sage-900 tabular-nums">{{ formatCurrency(totals.total_due) }}</p>
            </div>
          </div>

          <!-- Error -->
          <div v-if="error" class="mb-4 rounded-xl bg-red-50 border border-red-200 px-4 py-3 text-sm text-red-700 flex items-center gap-2">
            <X class="w-4 h-4 shrink-0 text-red-400" />
            {{ error }}
          </div>

          <!-- Submit -->
          <div class="flex items-center justify-between">
            <button
              type="button"
              class="flex items-center gap-1.5 text-sm text-sage-500 hover:text-sage-700 transition-colors cursor-pointer"
              @click="router.push('/invoices')"
            >
              <ChevronLeft class="w-4 h-4" />
              Annulla
            </button>
            <button
              type="submit"
              :disabled="saving"
              class="group relative overflow-hidden text-white font-semibold px-6 py-2.5 rounded-xl text-sm flex items-center gap-2 transition-all duration-200 disabled:opacity-60 cursor-pointer focus:outline-none"
              style="background: linear-gradient(135deg, #1e1b4b, #4338ca); box-shadow: 0 4px 20px rgba(67, 56, 202, 0.4);"
            >
              <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 transform -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700" aria-hidden="true" />
              <Check class="w-4 h-4 relative z-10" />
              <span class="relative z-10">{{ saving ? 'Salvataggio…' : isEdit ? 'Aggiorna fattura' : 'Crea fattura' }}</span>
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
/* Line item add/remove animation */
.line-enter-active { transition: all 0.25s ease; }
.line-leave-active { transition: all 0.2s ease; }
.line-enter-from   { opacity: 0; transform: translateY(-8px); }
.line-leave-to     { opacity: 0; transform: translateY(-4px); height: 0; margin: 0; padding: 0; }

@media (prefers-reduced-motion: reduce) {
  .line-enter-active,
  .line-leave-active { transition: opacity 0.1s ease; }
  .line-enter-from,
  .line-leave-to { transform: none; }
}
</style>

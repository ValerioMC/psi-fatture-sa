<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import {
  CalendarRange,
  FileText,
  Check,
  Loader2,
  AlertCircle,
  ChevronLeft,
  ChevronRight,
  Sparkles,
} from 'lucide-vue-next'
import { previewMonthlyInvoices, generateMonthlyInvoices } from '@/api'
import type { MonthlyInvoicePreview, PaymentMethod } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import { formatCurrency } from '@/utils/format'

const router = useRouter()

const MONTH_NAMES = [
  'Gennaio', 'Febbraio', 'Marzo', 'Aprile', 'Maggio', 'Giugno',
  'Luglio', 'Agosto', 'Settembre', 'Ottobre', 'Novembre', 'Dicembre',
]

const PAYMENT_OPTIONS: { value: PaymentMethod; label: string }[] = [
  { value: 'bonifico', label: 'Bonifico bancario' },
  { value: 'contanti', label: 'Contanti' },
  { value: 'pos', label: 'POS / Carta' },
  { value: 'altro', label: 'Altro' },
]

const now = new Date()
const selectedYear = ref(now.getFullYear())
const selectedMonth = ref(now.getMonth() + 1)
const paymentMethod = ref<PaymentMethod>('bonifico')
const applyEnpap = ref(true)

const previews = ref<MonthlyInvoicePreview[]>([])
const selectedClients = ref<Set<number>>(new Set())
const loading = ref(false)
const generating = ref(false)
const error = ref<string | null>(null)
const generatedCount = ref(0)
const step = ref<'select' | 'preview' | 'done'>('select')

const monthLabel = computed(() =>
  `${MONTH_NAMES[selectedMonth.value - 1]} ${selectedYear.value}`,
)

const selectedPreviews = computed(() =>
  previews.value.filter(p => selectedClients.value.has(p.client_id)),
)

const totalDue = computed(() =>
  selectedPreviews.value.reduce((sum, p) => sum + p.estimated_due, 0),
)

const allSelected = computed(() =>
  previews.value.length > 0 && selectedClients.value.size === previews.value.length,
)

/** Navigate between months. */
function shiftMonth(delta: number): void {
  let m = selectedMonth.value + delta
  let y = selectedYear.value
  if (m < 1) { m = 12; y-- }
  if (m > 12) { m = 1; y++ }
  selectedMonth.value = m
  selectedYear.value = y
  step.value = 'select'
  previews.value = []
}

/** Toggle all clients. */
function toggleAll(): void {
  if (allSelected.value) {
    selectedClients.value = new Set()
  } else {
    selectedClients.value = new Set(previews.value.map(p => p.client_id))
  }
}

/** Toggle a single client. */
function toggleClient(clientId: number): void {
  const next = new Set(selectedClients.value)
  if (next.has(clientId)) {
    next.delete(clientId)
  } else {
    next.add(clientId)
  }
  selectedClients.value = next
}

/** Load the preview for the selected month. */
async function loadPreview(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    previews.value = await previewMonthlyInvoices(selectedYear.value, selectedMonth.value)
    selectedClients.value = new Set(previews.value.map(p => p.client_id))
    step.value = 'preview'
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

/** Generate invoices for selected clients. */
async function generate(): Promise<void> {
  if (selectedClients.value.size === 0) return
  generating.value = true
  error.value = null
  try {
    const result = await generateMonthlyInvoices({
      year: selectedYear.value,
      month: selectedMonth.value,
      client_ids: [...selectedClients.value],
      payment_method: paymentMethod.value,
      apply_enpap: applyEnpap.value,
    })
    generatedCount.value = result.length
    step.value = 'done'
  } catch (e) {
    error.value = String(e)
  } finally {
    generating.value = false
  }
}
</script>

<template>
  <div class="p-8">
    <div class="max-w-3xl mx-auto">
      <PageHeader
        title="Genera fatture mensili"
        subtitle="Crea fatture automatiche dagli appuntamenti completati del mese."
      />

      <!-- ── Month selector ── -->
      <div class="glass-card rounded-2xl p-5 mb-5 animate-in">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl flex items-center justify-center bg-sage-50">
              <CalendarRange class="w-5 h-5 text-sage-500" />
            </div>
            <div>
              <div class="text-xs font-semibold text-sage-400 uppercase tracking-wider">Periodo</div>
              <div class="text-lg font-semibold text-sage-800">{{ monthLabel }}</div>
            </div>
          </div>

          <div class="flex items-center gap-2">
            <button
              type="button"
              class="p-2 rounded-lg text-sage-400 hover:text-sage-700 hover:bg-sage-50 transition-all"
              @click="shiftMonth(-1)"
            >
              <ChevronLeft class="w-5 h-5" />
            </button>
            <button
              type="button"
              class="p-2 rounded-lg text-sage-400 hover:text-sage-700 hover:bg-sage-50 transition-all"
              @click="shiftMonth(1)"
            >
              <ChevronRight class="w-5 h-5" />
            </button>
          </div>
        </div>

        <!-- Settings row -->
        <div class="flex items-end gap-4 mt-4 pt-4 border-t border-sage-100/60">
          <div class="flex-1 max-w-xs">
            <label class="text-xs font-medium text-sage-500 block mb-1">Metodo di pagamento</label>
            <select
              v-model="paymentMethod"
              class="w-full bg-white/60 border border-sage-200/70 rounded-xl px-3 py-2 text-sm text-sage-800 focus:outline-none focus:ring-2 focus:ring-sage-400/40"
            >
              <option v-for="opt in PAYMENT_OPTIONS" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </div>
          <label class="flex items-center gap-2 cursor-pointer pb-2">
            <input
              v-model="applyEnpap"
              type="checkbox"
              class="rounded border-sage-300 text-sage-600 focus:ring-sage-400"
            />
            <span class="text-sm text-sage-700">Applica ENPAP 2%</span>
          </label>
          <button
            v-if="step === 'select'"
            type="button"
            :disabled="loading"
            class="ml-auto group relative overflow-hidden text-white font-semibold px-5 py-2.5 rounded-xl text-sm flex items-center gap-2 transition-all disabled:opacity-60"
            style="background: linear-gradient(135deg, #1e1b4b, #4338ca); box-shadow: 0 4px 20px rgba(67, 56, 202, 0.4);"
            @click="loadPreview"
          >
            <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700 pointer-events-none" />
            <Loader2 v-if="loading" class="w-4 h-4 animate-spin relative z-10" />
            <CalendarRange v-else class="w-4 h-4 relative z-10" />
            <span class="relative z-10">{{ loading ? 'Caricamento...' : 'Carica anteprima' }}</span>
          </button>
        </div>
      </div>

      <!-- ── Error ── -->
      <div
        v-if="error"
        class="flex items-center gap-3 rounded-xl bg-red-50 border border-red-200 px-4 py-3 text-sm text-red-700 mb-5"
      >
        <AlertCircle class="w-4 h-4 shrink-0" />
        {{ error }}
      </div>

      <!-- ── Preview step ── -->
      <template v-if="step === 'preview'">
        <!-- Empty state -->
        <div
          v-if="previews.length === 0"
          class="glass-card rounded-2xl p-12 text-center animate-in-d1"
        >
          <div class="w-14 h-14 rounded-2xl bg-sage-50 flex items-center justify-center mx-auto mb-3">
            <CalendarRange class="w-7 h-7 text-sage-300" />
          </div>
          <p class="text-sm font-semibold text-sage-600">Nessun appuntamento da fatturare</p>
          <p class="text-xs text-sage-400 mt-1">
            Non ci sono appuntamenti completati e non ancora fatturati per {{ monthLabel }}.
          </p>
        </div>

        <!-- Client cards -->
        <div v-else class="space-y-3 animate-in-d1">
          <!-- Select all -->
          <div class="flex items-center justify-between px-1 mb-2">
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                :checked="allSelected"
                class="rounded border-sage-300 text-sage-600 focus:ring-sage-400"
                @change="toggleAll"
              />
              <span class="text-sm font-medium text-sage-600">
                Seleziona tutti ({{ previews.length }} pazienti)
              </span>
            </label>
            <div class="text-sm text-sage-500">
              Totale stimato: <span class="font-semibold text-sage-800">{{ formatCurrency(totalDue) }}</span>
            </div>
          </div>

          <!-- Per-client card -->
          <div
            v-for="preview in previews"
            :key="preview.client_id"
            class="glass-card rounded-xl p-4 transition-all cursor-pointer"
            :class="selectedClients.has(preview.client_id)
              ? 'ring-2 ring-sage-400/50'
              : 'opacity-60'"
            @click="toggleClient(preview.client_id)"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <input
                  type="checkbox"
                  :checked="selectedClients.has(preview.client_id)"
                  class="rounded border-sage-300 text-sage-600 focus:ring-sage-400"
                  @click.stop
                  @change="toggleClient(preview.client_id)"
                />
                <div>
                  <div class="text-sm font-semibold text-sage-800">{{ preview.client_name }}</div>
                  <div class="text-xs text-sage-400">
                    {{ preview.appointment_count }} {{ preview.appointment_count === 1 ? 'seduta' : 'sedute' }}
                  </div>
                </div>
              </div>
              <div class="text-right">
                <div class="text-xs text-sage-400">Imponibile</div>
                <div class="text-sm font-semibold text-sage-800">{{ formatCurrency(preview.estimated_net) }}</div>
              </div>
            </div>

            <!-- Lines detail -->
            <div
              v-if="selectedClients.has(preview.client_id) && preview.lines.length > 0"
              class="mt-3 pt-3 border-t border-sage-100/60 space-y-1"
            >
              <div
                v-for="(line, i) in preview.lines"
                :key="i"
                class="flex items-center justify-between text-xs"
              >
                <span class="text-sage-500 truncate mr-4">{{ line.description }}</span>
                <span class="text-sage-700 font-medium shrink-0">
                  {{ line.quantity }} &times; {{ formatCurrency(line.unit_price) }}
                </span>
              </div>
            </div>
          </div>

          <!-- Generate button -->
          <div class="flex justify-end pt-3">
            <button
              type="button"
              :disabled="generating || selectedClients.size === 0"
              class="group relative overflow-hidden text-white font-semibold px-6 py-2.5 rounded-xl text-sm flex items-center gap-2 transition-all disabled:opacity-60"
              style="background: linear-gradient(135deg, #1e1b4b, #4338ca); box-shadow: 0 4px 20px rgba(67, 56, 202, 0.4);"
              @click="generate"
            >
              <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700 pointer-events-none" />
              <Loader2 v-if="generating" class="w-4 h-4 animate-spin relative z-10" />
              <Sparkles v-else class="w-4 h-4 relative z-10" />
              <span class="relative z-10">
                {{ generating ? 'Generazione...' : `Genera ${selectedClients.size} fattur${selectedClients.size === 1 ? 'a' : 'e'}` }}
              </span>
            </button>
          </div>
        </div>
      </template>

      <!-- ── Done step ── -->
      <div
        v-if="step === 'done'"
        class="glass-card rounded-2xl p-10 text-center animate-in"
      >
        <div
          class="w-16 h-16 rounded-2xl flex items-center justify-center mx-auto mb-4"
          style="background: linear-gradient(135deg, #dcfce7, #d1fae5)"
        >
          <Check class="w-8 h-8 text-green-600" />
        </div>
        <p class="text-lg font-semibold text-sage-800">
          {{ generatedCount }} fattur{{ generatedCount === 1 ? 'a creata' : 'e create' }}
        </p>
        <p class="text-sm text-sage-400 mt-1">
          Le fatture per {{ monthLabel }} sono state generate con successo.
        </p>
        <div class="flex items-center justify-center gap-3 mt-6">
          <button
            type="button"
            class="px-4 py-2 rounded-xl text-sm font-medium text-sage-600 border border-sage-200 hover:bg-sage-50 transition-all"
            @click="router.push('/invoices')"
          >
            <FileText class="w-4 h-4 inline mr-1.5 -mt-0.5" />
            Vai alle fatture
          </button>
          <button
            type="button"
            class="px-4 py-2 rounded-xl text-sm font-medium text-sage-600 border border-sage-200 hover:bg-sage-50 transition-all"
            @click="step = 'select'; previews = []"
          >
            Genera altro mese
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

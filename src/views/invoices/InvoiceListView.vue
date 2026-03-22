<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { Plus, Search, Eye, Pencil, Trash2, FileText, CalendarRange, CheckSquare, X, TrendingUp, Euro, Clock } from 'lucide-vue-next'
import { useInvoicesStore } from '@/stores/invoices'
import type { Invoice, InvoiceStatus } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import ConfirmModal from '@/components/ui/ConfirmModal.vue'
import { formatCurrency, formatDate } from '@/utils/format'

const router = useRouter()
const invoicesStore = useInvoicesStore()
const currentYear = new Date().getFullYear()
const invoiceToDelete = ref<Invoice | null>(null)

const STATUS_OPTIONS = [
  { value: '', label: 'Tutti gli stati' },
  { value: 'draft', label: 'Bozza' },
  { value: 'issued', label: 'Emessa' },
  { value: 'paid', label: 'Pagata' },
  { value: 'overdue', label: 'Scaduta' },
  { value: 'cancelled', label: 'Annullata' },
]

const TARGET_STATUS_OPTIONS: { value: InvoiceStatus; label: string }[] = [
  { value: 'draft', label: 'Bozza' },
  { value: 'issued', label: 'Emessa' },
  { value: 'paid', label: 'Pagata' },
  { value: 'overdue', label: 'Scaduta' },
  { value: 'cancelled', label: 'Annullata' },
]

const YEAR_OPTIONS = [
  { value: null, label: 'Tutti gli anni' },
  { value: currentYear, label: String(currentYear) },
  { value: currentYear - 1, label: String(currentYear - 1) },
  { value: currentYear - 2, label: String(currentYear - 2) },
]

const filters = reactive<{ year: number | null; status: string; search: string }>({ year: currentYear, status: '', search: '' })
let debounceTimer: ReturnType<typeof setTimeout> | null = null

// ─── Financial summary ────────────────────────────────────────────────────────

const financialSummary = computed(() => {
  const invoices = invoicesStore.invoices
  const total    = invoices.reduce((s, i) => s + i.total_due, 0)
  const paid     = invoices.filter(i => i.status === 'paid').reduce((s, i) => s + i.total_due, 0)
  const pending  = invoices.filter(i => i.status === 'issued' || i.status === 'overdue').reduce((s, i) => s + i.total_due, 0)
  return { count: invoices.length, total, paid, pending }
})

// ─── Selection state ──────────────────────────────────────────────────────────

const selectedIds    = ref<Set<number>>(new Set())
const bulkTargetStatus = ref<InvoiceStatus>('paid')
const bulkUpdating   = ref(false)
const showBulkConfirm = ref(false)

const selectedCount = computed(() => selectedIds.value.size)
const allSelected   = computed(
  () => invoicesStore.invoices.length > 0 && selectedIds.value.size === invoicesStore.invoices.length,
)

function toggleSelectAll() {
  if (allSelected.value) {
    selectedIds.value = new Set()
  } else {
    selectedIds.value = new Set(invoicesStore.invoices.map(i => i.id))
  }
}

function toggleSelect(id: number) {
  const next = new Set(selectedIds.value)
  next.has(id) ? next.delete(id) : next.add(id)
  selectedIds.value = next
}

function clearSelection() { selectedIds.value = new Set() }

function requestBulkUpdate() { showBulkConfirm.value = true }

async function executeBulkUpdate() {
  showBulkConfirm.value = false
  bulkUpdating.value = true
  try {
    const ids = [...selectedIds.value]
    const paidDate = bulkTargetStatus.value === 'paid' ? new Date().toISOString().slice(0, 10) : undefined
    await invoicesStore.bulkUpdateStatus(ids, bulkTargetStatus.value, paidDate)
    clearSelection()
  } catch (e) {
    console.error('[bulk] FAILED:', e)
  } finally {
    bulkUpdating.value = false
  }
}

function bulkConfirmMessage(): string {
  const label = TARGET_STATUS_OPTIONS.find(o => o.value === bulkTargetStatus.value)?.label ?? bulkTargetStatus.value
  return `Cambiare lo stato di ${selectedCount.value} fattura/e in "${label}"?`
}

// ─── Data loading ─────────────────────────────────────────────────────────────

function loadInvoices() {
  clearSelection()
  invoicesStore.fetchInvoices({
    year:   filters.year ?? undefined,
    status: filters.status || undefined,
    search: filters.search || undefined,
  })
}

function onSearchInput() {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(loadInvoices, 300)
}

function clearSearch() {
  filters.search = ''
  loadInvoices()
}

watch(() => [filters.year, filters.status], loadInvoices)
onMounted(loadInvoices)

function confirmDelete(invoice: Invoice) { invoiceToDelete.value = invoice }

async function handleDelete() {
  if (!invoiceToDelete.value) return
  try {
    await invoicesStore.removeInvoice(invoiceToDelete.value.id)
  } finally {
    invoiceToDelete.value = null
  }
}

function rowDelay(idx: number): number {
  return Math.min(idx, 12)
}
</script>

<template>
  <div class="p-8">
    <div class="max-w-5xl mx-auto">
      <PageHeader title="Fatture" subtitle="Gestisci tutte le tue fatture">
        <button
          type="button"
          class="flex items-center gap-2 border border-sage-200 text-sage-600 hover:text-sage-800 hover:bg-sage-50 font-medium px-4 py-2 rounded-xl text-sm transition-all cursor-pointer"
          @click="router.push('/invoices/monthly')"
        >
          <CalendarRange class="w-4 h-4" />
          Genera mensili
        </button>
        <button
          type="button"
          class="group relative overflow-hidden text-white font-semibold px-4 py-2 rounded-xl text-sm flex items-center gap-2 transition-all duration-200 cursor-pointer focus:outline-none"
          style="background: linear-gradient(135deg, #1e1b4b, #4338ca); box-shadow: 0 4px 20px rgba(67, 56, 202, 0.4);"
          @click="router.push('/invoices/new')"
        >
          <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 transform -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700" aria-hidden="true" />
          <Plus class="w-4 h-4 relative z-10" />
          <span class="relative z-10">Nuova Fattura</span>
        </button>
      </PageHeader>

      <!-- Financial summary strip -->
      <div v-if="!invoicesStore.loading && invoicesStore.invoices.length > 0" class="grid grid-cols-3 gap-4 mb-5 animate-in">
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #059669, #047857)">
            <TrendingUp class="w-4 h-4 text-white" />
          </div>
          <div class="min-w-0">
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">{{ financialSummary.count }} fatture</p>
            <p class="text-lg font-bold text-sage-900 leading-tight truncate">{{ formatCurrency(financialSummary.total) }}</p>
          </div>
        </div>
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #4f46e5, #4338ca)">
            <Euro class="w-4 h-4 text-white" />
          </div>
          <div class="min-w-0">
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">Incassato</p>
            <p class="text-lg font-bold text-sage-900 leading-tight truncate">{{ formatCurrency(financialSummary.paid) }}</p>
          </div>
        </div>
        <div class="glass-card rounded-xl p-4 shadow-sm flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #d97706, #b45309)">
            <Clock class="w-4 h-4 text-white" />
          </div>
          <div class="min-w-0">
            <p class="text-[10px] text-sage-400 uppercase tracking-wider">In sospeso</p>
            <p class="text-lg font-bold text-sage-900 leading-tight truncate">{{ formatCurrency(financialSummary.pending) }}</p>
          </div>
        </div>
      </div>

      <!-- Filters bar -->
      <div class="glass-card rounded-2xl px-4 py-3 shadow-sm mb-4 flex items-center gap-3 animate-in">
        <select
          v-model="filters.year"
          class="bg-white/70 border border-sage-200/70 rounded-xl px-3 py-2 text-sm text-sage-800 focus:outline-none focus:ring-2 focus:ring-sage-400/40 transition-all cursor-pointer"
        >
          <option v-for="opt in YEAR_OPTIONS" :key="String(opt.value)" :value="opt.value">{{ opt.label }}</option>
        </select>

        <select
          v-model="filters.status"
          class="bg-white/70 border border-sage-200/70 rounded-xl px-3 py-2 text-sm text-sage-800 focus:outline-none focus:ring-2 focus:ring-sage-400/40 transition-all cursor-pointer"
        >
          <option v-for="opt in STATUS_OPTIONS" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
        </select>

        <div class="relative flex-1">
          <Search class="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-sage-400 pointer-events-none" />
          <input
            v-model="filters.search"
            type="text"
            placeholder="Cerca per numero, cliente…"
            class="w-full bg-white/70 border border-sage-200/70 rounded-xl pl-10 pr-9 py-2 text-sm text-sage-800 placeholder:text-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 transition-all"
            @input="onSearchInput"
          />
          <button
            v-if="filters.search"
            type="button"
            class="absolute right-3 top-1/2 -translate-y-1/2 p-0.5 text-sage-400 hover:text-sage-600 transition-colors cursor-pointer rounded"
            @click="clearSearch"
          >
            <X class="w-3.5 h-3.5" />
          </button>
        </div>

        <span class="text-xs text-sage-400 whitespace-nowrap shrink-0">
          {{ invoicesStore.invoices.length }} {{ invoicesStore.invoices.length === 1 ? 'fattura' : 'fatture' }}
        </span>
      </div>

      <!-- Bulk action bar -->
      <Transition
        enter-active-class="transition-all duration-300 ease-out"
        enter-from-class="opacity-0 -translate-y-2"
        enter-to-class="opacity-100 translate-y-0"
        leave-active-class="transition-all duration-200 ease-in"
        leave-from-class="opacity-100 translate-y-0"
        leave-to-class="opacity-0 -translate-y-2"
      >
        <div
          v-if="selectedCount > 0"
          class="glass-card rounded-2xl px-5 py-3 shadow-sm mb-4 flex items-center gap-4 border border-ocean-200/60 bg-ocean-50/30"
        >
          <div class="flex items-center gap-2">
            <CheckSquare class="w-4 h-4 text-ocean-600" />
            <span class="text-sm font-semibold text-ocean-700">
              {{ selectedCount }} selezionat{{ selectedCount === 1 ? 'a' : 'e' }}
            </span>
          </div>
          <div class="h-5 w-px bg-ocean-200/60" />
          <div class="flex items-center gap-2 flex-1">
            <label class="text-sm text-sage-600 whitespace-nowrap">Cambia stato in:</label>
            <select
              v-model="bulkTargetStatus"
              class="bg-white/80 border border-sage-200/70 rounded-xl px-3 py-1.5 text-sm text-sage-800 focus:outline-none focus:ring-2 focus:ring-ocean-400/40 transition-all cursor-pointer"
            >
              <option v-for="opt in TARGET_STATUS_OPTIONS" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
            </select>
            <button
              type="button"
              class="text-white font-medium px-4 py-1.5 rounded-xl text-sm transition-all duration-200 disabled:opacity-50 cursor-pointer"
              style="background: linear-gradient(135deg, #1e1b4b, #4338ca);"
              :disabled="bulkUpdating"
              @click="requestBulkUpdate"
            >
              {{ bulkUpdating ? 'Aggiornamento…' : 'Applica' }}
            </button>
          </div>
          <button
            type="button"
            class="p-1.5 text-sage-400 hover:text-sage-600 hover:bg-sage-100 rounded-lg transition-all cursor-pointer"
            @click="clearSelection"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </Transition>

      <!-- Table card -->
      <div class="glass-card rounded-2xl shadow-sm overflow-hidden animate-in-d1">
        <!-- Loading -->
        <div v-if="invoicesStore.loading" class="flex flex-col items-center justify-center py-16 gap-3">
          <div class="w-8 h-8 rounded-full border-2 border-sage-200 border-t-sage-500 animate-spin" />
          <p class="text-sm text-sage-400">Caricamento fatture…</p>
        </div>

        <!-- Empty state -->
        <div
          v-else-if="invoicesStore.invoices.length === 0"
          class="flex flex-col items-center justify-center py-16 text-center px-6"
        >
          <div class="w-14 h-14 rounded-2xl bg-sage-50 flex items-center justify-center mb-3">
            <FileText class="w-7 h-7 text-sage-300" />
          </div>
          <p class="text-sm font-semibold text-sage-600">Nessuna fattura trovata</p>
          <p class="text-xs text-sage-400 mt-1">Modifica i filtri o crea una nuova fattura.</p>
          <button
            type="button"
            class="mt-4 flex items-center gap-1.5 text-sm font-medium text-sage-600 hover:text-sage-800 transition-colors cursor-pointer"
            @click="router.push('/invoices/new')"
          >
            <Plus class="w-4 h-4" />
            Nuova fattura
          </button>
        </div>

        <!-- Table -->
        <table v-else class="w-full text-sm">
          <thead>
            <tr class="border-b border-sage-100/60 bg-sage-50/30">
              <th class="px-4 py-3 text-left w-10">
                <input
                  type="checkbox"
                  :checked="allSelected"
                  class="w-4 h-4 rounded border-sage-300 text-ocean-600 cursor-pointer accent-ocean-600"
                  @change="toggleSelectAll"
                />
              </th>
              <th class="px-4 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Numero</th>
              <th class="px-4 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Data</th>
              <th class="px-4 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Cliente</th>
              <th class="px-4 py-3 text-right text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Totale</th>
              <th class="px-4 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Stato</th>
              <th class="px-4 py-3 text-right text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Azioni</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(invoice, idx) in invoicesStore.invoices"
              :key="invoice.id"
              :style="{ '--i': rowDelay(idx) }"
              class="invoice-row border-b border-sage-50/70 transition-all duration-150 cursor-pointer group"
              :class="selectedIds.has(invoice.id) ? 'bg-ocean-50/40 hover:bg-ocean-50/60' : 'hover:bg-sage-50/50'"
              @click="router.push(`/invoices/${invoice.id}`)"
            >
              <!-- Checkbox -->
              <td class="px-4 py-3.5" @click.stop>
                <input
                  type="checkbox"
                  :checked="selectedIds.has(invoice.id)"
                  class="w-4 h-4 rounded border-sage-300 text-ocean-600 cursor-pointer accent-ocean-600"
                  @change="toggleSelect(invoice.id)"
                />
              </td>

              <!-- Number -->
              <td class="px-4 py-3.5">
                <span class="font-mono text-xs font-semibold text-sage-800 bg-sage-50 border border-sage-100 px-2 py-1 rounded-lg">
                  {{ invoice.invoice_number }}
                </span>
              </td>

              <!-- Date -->
              <td class="px-4 py-3.5 text-sage-500 text-xs">{{ formatDate(invoice.issue_date) }}</td>

              <!-- Client -->
              <td class="px-4 py-3.5">
                <span class="font-medium text-sage-800 truncate">{{ invoice.client_name }}</span>
              </td>

              <!-- Total -->
              <td class="px-4 py-3.5 text-right font-bold text-sage-900 tabular-nums">
                {{ formatCurrency(invoice.total_due) }}
              </td>

              <!-- Status -->
              <td class="px-4 py-3.5">
                <StatusBadge :status="invoice.status" type="invoice" />
              </td>

              <!-- Actions: always visible, subtle -->
              <td class="px-4 py-3.5" @click.stop>
                <div class="flex items-center justify-end gap-1">
                  <button
                    type="button"
                    class="p-1.5 text-sage-300 hover:text-ocean-600 hover:bg-ocean-50 rounded-lg transition-all duration-150 cursor-pointer"
                    title="Visualizza"
                    @click="router.push(`/invoices/${invoice.id}`)"
                  >
                    <Eye class="w-3.5 h-3.5" />
                  </button>
                  <button
                    type="button"
                    class="p-1.5 text-sage-300 hover:text-sage-600 hover:bg-sage-50 rounded-lg transition-all duration-150 cursor-pointer"
                    title="Modifica"
                    @click="router.push(`/invoices/${invoice.id}/edit`)"
                  >
                    <Pencil class="w-3.5 h-3.5" />
                  </button>
                  <button
                    type="button"
                    class="p-1.5 text-sage-300 hover:text-red-500 hover:bg-red-50 rounded-lg transition-all duration-150 cursor-pointer"
                    title="Elimina"
                    @click="confirmDelete(invoice)"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <ConfirmModal
        :open="!!invoiceToDelete"
        title="Elimina fattura"
        :message="`Eliminare la fattura ${invoiceToDelete?.invoice_number}?`"
        @confirm="handleDelete"
        @cancel="invoiceToDelete = null"
      />
      <ConfirmModal
        :open="showBulkConfirm"
        title="Aggiornamento stato"
        :message="bulkConfirmMessage()"
        confirm-label="Conferma"
        variant="primary"
        @confirm="executeBulkUpdate"
        @cancel="showBulkConfirm = false"
      />
    </div>
  </div>
</template>

<style scoped>
.invoice-row {
  animation: row-in 0.28s ease both;
  animation-delay: calc(var(--i, 0) * 30ms);
}

@keyframes row-in {
  from { opacity: 0; transform: translateX(-5px); }
  to   { opacity: 1; transform: translateX(0); }
}

@media (prefers-reduced-motion: reduce) {
  .invoice-row { animation: none; }
}
</style>

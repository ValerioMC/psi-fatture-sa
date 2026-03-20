<script setup lang="ts">
import { ref, reactive, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { Plus, Search, Eye, Pencil, Trash2, FileText } from 'lucide-vue-next'
import { useInvoicesStore } from '@/stores/invoices'
import type { Invoice } from '@/types'
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

const YEAR_OPTIONS = [currentYear, currentYear - 1, currentYear - 2]

const filters = reactive({ year: currentYear, status: '', search: '' })
let debounceTimer: ReturnType<typeof setTimeout> | null = null

function loadInvoices() {
  invoicesStore.fetchInvoices({
    year: filters.year,
    status: filters.status || undefined,
    search: filters.search || undefined,
  })
}

function onSearchInput() {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(loadInvoices, 300)
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
</script>

<template>
  <div class="p-8">
    <div class="max-w-5xl mx-auto">
    <PageHeader title="Fatture" subtitle="Gestisci tutte le tue fatture">
      <button
        type="button"
        class="group relative overflow-hidden text-white font-semibold px-4 py-2 rounded-xl text-sm flex items-center gap-2 transition-all duration-200 focus:outline-none"
        style="background: linear-gradient(135deg, #5d8062, #0c8aeb); box-shadow: 0 4px 14px rgba(93,128,98,0.3);"
        @click="router.push('/invoices/new')"
      >
        <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 transform -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700" aria-hidden="true" />
        <Plus class="w-4 h-4 relative z-10" />
        <span class="relative z-10">Nuova Fattura</span>
      </button>
    </PageHeader>

    <!-- Filters -->
    <div class="glass-card rounded-2xl px-5 py-4 shadow-sm mb-5 flex items-center gap-3 animate-in">
      <select
        v-model.number="filters.year"
        class="bg-white/60 border border-sage-200/70 rounded-xl px-3 py-2 text-sm text-sage-800 focus:outline-none focus:ring-2 focus:ring-sage-400/40 transition-all"
      >
        <option v-for="y in YEAR_OPTIONS" :key="y" :value="y">{{ y }}</option>
      </select>

      <select
        v-model="filters.status"
        class="bg-white/60 border border-sage-200/70 rounded-xl px-3 py-2 text-sm text-sage-800 focus:outline-none focus:ring-2 focus:ring-sage-400/40 transition-all"
      >
        <option v-for="opt in STATUS_OPTIONS" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>

      <div class="relative flex-1 max-w-sm">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-sage-400" />
        <input
          v-model="filters.search"
          type="text"
          placeholder="Cerca per numero, cliente..."
          class="w-full bg-white/60 border border-sage-200/70 rounded-xl pl-9 pr-3 py-2 text-sm text-sage-800 placeholder:text-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 transition-all"
          @input="onSearchInput"
        />
      </div>
    </div>

    <!-- Table -->
    <div class="glass-card rounded-2xl shadow-sm animate-in-d1">
      <!-- Loading -->
      <div v-if="invoicesStore.loading" class="flex flex-col items-center justify-center py-16 gap-3">
        <div class="w-8 h-8 rounded-full border-2 border-sage-200 border-t-sage-500 animate-spin" />
        <p class="text-sm text-sage-400">Caricamento...</p>
      </div>

      <!-- Empty state -->
      <div
        v-else-if="invoicesStore.invoices.length === 0"
        class="flex flex-col items-center justify-center py-16 text-center"
      >
        <div class="w-14 h-14 rounded-2xl bg-sage-50 flex items-center justify-center mb-3">
          <FileText class="w-7 h-7 text-sage-300" />
        </div>
        <p class="text-sm font-semibold text-sage-600">Nessuna fattura trovata</p>
        <p class="text-xs text-sage-400 mt-1">Modifica i filtri o crea una nuova fattura.</p>
      </div>

      <!-- Table -->
      <table v-else class="w-full text-sm">
        <thead>
          <tr class="border-b border-sage-100/60">
            <th class="px-6 py-3.5 text-left text-xs font-semibold text-sage-400 uppercase tracking-wider">Numero</th>
            <th class="px-6 py-3.5 text-left text-xs font-semibold text-sage-400 uppercase tracking-wider">Data</th>
            <th class="px-6 py-3.5 text-left text-xs font-semibold text-sage-400 uppercase tracking-wider">Cliente</th>
            <th class="px-6 py-3.5 text-right text-xs font-semibold text-sage-400 uppercase tracking-wider">Totale</th>
            <th class="px-6 py-3.5 text-left text-xs font-semibold text-sage-400 uppercase tracking-wider">Stato</th>
            <th class="px-6 py-3.5 text-right text-xs font-semibold text-sage-400 uppercase tracking-wider">Azioni</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="invoice in invoicesStore.invoices"
            :key="invoice.id"
            class="border-b border-sage-50/70 hover:bg-sage-50/40 transition-colors"
          >
            <td class="px-6 py-3.5 font-semibold text-sage-800">{{ invoice.invoice_number }}</td>
            <td class="px-6 py-3.5 text-sage-400">{{ formatDate(invoice.issue_date) }}</td>
            <td class="px-6 py-3.5 text-sage-600">{{ invoice.client_name }}</td>
            <td class="px-6 py-3.5 text-right font-semibold text-sage-800">{{ formatCurrency(invoice.total_due) }}</td>
            <td class="px-6 py-3.5"><StatusBadge :status="invoice.status" type="invoice" /></td>
            <td class="px-6 py-3.5">
              <div class="flex items-center justify-end gap-1">
                <button
                  type="button"
                  class="p-1.5 text-sage-300 hover:text-ocean-600 hover:bg-ocean-50 rounded-lg transition-all"
                  title="Visualizza"
                  @click="router.push(`/invoices/${invoice.id}`)"
                >
                  <Eye class="w-4 h-4" />
                </button>
                <button
                  type="button"
                  class="p-1.5 text-sage-300 hover:text-sage-600 hover:bg-sage-50 rounded-lg transition-all"
                  title="Modifica"
                  @click="router.push(`/invoices/${invoice.id}/edit`)"
                >
                  <Pencil class="w-4 h-4" />
                </button>
                <button
                  type="button"
                  class="p-1.5 text-sage-300 hover:text-red-600 hover:bg-red-50 rounded-lg transition-all"
                  title="Elimina"
                  @click="confirmDelete(invoice)"
                >
                  <Trash2 class="w-4 h-4" />
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
      :message="`Sei sicuro di voler eliminare la fattura ${invoiceToDelete?.invoice_number}?`"
      @confirm="handleDelete"
      @cancel="invoiceToDelete = null"
    />
    </div>
  </div>
</template>

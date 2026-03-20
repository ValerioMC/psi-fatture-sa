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

const STATUS_OPTIONS: Array<{ value: string; label: string }> = [
  { value: '', label: 'Tutti gli stati' },
  { value: 'draft', label: 'Bozza' },
  { value: 'issued', label: 'Emessa' },
  { value: 'paid', label: 'Pagata' },
  { value: 'overdue', label: 'Scaduta' },
  { value: 'cancelled', label: 'Annullata' },
]

const YEAR_OPTIONS = [currentYear, currentYear - 1, currentYear - 2]

const filters = reactive({
  year: currentYear,
  status: '',
  search: '',
})

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

function confirmDelete(invoice: Invoice) {
  invoiceToDelete.value = invoice
}

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
    <PageHeader title="Fatture" subtitle="Gestisci tutte le tue fatture.">
      <button
        type="button"
        class="bg-blue-600 text-white hover:bg-blue-700 px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-colors"
        @click="router.push('/invoices/new')"
      >
        <Plus class="w-4 h-4" />
        Nuova Fattura
      </button>
    </PageHeader>

    <!-- Filters -->
    <div class="flex items-center gap-3 mb-4">
      <select
        v-model.number="filters.year"
        class="border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        <option v-for="y in YEAR_OPTIONS" :key="y" :value="y">{{ y }}</option>
      </select>

      <select
        v-model="filters.status"
        class="border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        <option v-for="opt in STATUS_OPTIONS" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>

      <div class="relative flex-1 max-w-sm">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
        <input
          v-model="filters.search"
          type="text"
          placeholder="Cerca per numero, cliente..."
          class="w-full border border-gray-300 rounded-lg pl-9 pr-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          @input="onSearchInput"
        />
      </div>
    </div>

    <!-- Table -->
    <div class="bg-white rounded-xl border border-gray-100 shadow-sm">
      <div v-if="invoicesStore.loading" class="px-6 py-12 text-center text-sm text-gray-400">
        Caricamento...
      </div>

      <div
        v-else-if="invoicesStore.invoices.length === 0"
        class="flex flex-col items-center justify-center py-16 text-center"
      >
        <FileText class="w-12 h-12 text-gray-200 mb-3" />
        <p class="text-sm font-medium text-gray-500">Nessuna fattura trovata</p>
        <p class="text-xs text-gray-400 mt-1">Modifica i filtri o crea una nuova fattura.</p>
      </div>

      <table v-else class="w-full text-sm">
        <thead>
          <tr class="border-b border-gray-100">
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Numero</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Data</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Cliente</th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Totale</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Stato</th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Azioni</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-50">
          <tr
            v-for="invoice in invoicesStore.invoices"
            :key="invoice.id"
            class="hover:bg-gray-50"
          >
            <td class="px-6 py-3 font-medium text-gray-900">{{ invoice.invoice_number }}</td>
            <td class="px-6 py-3 text-gray-500">{{ formatDate(invoice.issue_date) }}</td>
            <td class="px-6 py-3 text-gray-700">{{ invoice.client_name }}</td>
            <td class="px-6 py-3 text-right font-medium text-gray-900">
              {{ formatCurrency(invoice.total_due) }}
            </td>
            <td class="px-6 py-3">
              <StatusBadge :status="invoice.status" type="invoice" />
            </td>
            <td class="px-6 py-3">
              <div class="flex items-center justify-end gap-2">
                <button
                  type="button"
                  class="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                  title="Visualizza"
                  @click="router.push(`/invoices/${invoice.id}`)"
                >
                  <Eye class="w-4 h-4" />
                </button>
                <button
                  type="button"
                  class="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                  title="Modifica"
                  @click="router.push(`/invoices/${invoice.id}/edit`)"
                >
                  <Pencil class="w-4 h-4" />
                </button>
                <button
                  type="button"
                  class="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
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
</template>

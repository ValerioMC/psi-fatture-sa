<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Pencil, Trash2, CheckCircle, ArrowLeft, Printer } from 'lucide-vue-next'
import { useInvoicesStore } from '@/stores/invoices'
import type { Invoice } from '@/types'
import PageHeader from '@/components/ui/PageHeader.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import ConfirmModal from '@/components/ui/ConfirmModal.vue'
import { formatCurrency, formatDate } from '@/utils/format'

const route = useRoute()
const router = useRouter()
const invoicesStore = useInvoicesStore()

const invoiceId = Number(route.params.id)
const invoice = ref<Invoice | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const showDeleteModal = ref(false)
const markingPaid = ref(false)

const PAYMENT_METHOD_LABELS: Record<string, string> = {
  bonifico: 'Bonifico bancario',
  contanti: 'Contanti',
  pos: 'POS / Carta',
  altro: 'Altro',
}

onMounted(async () => {
  loading.value = true
  try {
    invoice.value = await invoicesStore.fetchInvoice(invoiceId)
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})

async function handleDelete() {
  showDeleteModal.value = false
  await invoicesStore.removeInvoice(invoiceId)
  router.push('/invoices')
}

async function markAsPaid() {
  if (!invoice.value) return
  markingPaid.value = true
  try {
    const today = new Date().toISOString().slice(0, 10)
    const updated = await invoicesStore.editInvoice({
      id: invoice.value.id,
      client_id: invoice.value.client_id,
      issue_date: invoice.value.issue_date,
      due_date: invoice.value.due_date,
      status: 'paid',
      payment_method: invoice.value.payment_method,
      notes: invoice.value.notes,
      apply_enpap: invoice.value.apply_enpap,
      paid_date: today,
      lines: invoice.value.lines.map((l) => ({
        service_id: l.service_id,
        description: l.description,
        quantity: l.quantity,
        unit_price: l.unit_price,
        vat_rate: l.vat_rate,
      })),
    })
    invoice.value = updated
  } finally {
    markingPaid.value = false
  }
}

function canMarkAsPaid(): boolean {
  return invoice.value?.status === 'issued' || invoice.value?.status === 'overdue'
}
</script>

<template>
  <div class="p-8">
    <div class="max-w-4xl mx-auto">
    <div class="flex items-center gap-3 mb-6 animate-in">
      <button
        type="button"
        class="p-1.5 text-sage-400 hover:text-sage-700 hover:bg-sage-100 rounded-lg transition-colors"
        @click="router.push('/invoices')"
      >
        <ArrowLeft class="w-5 h-5" />
      </button>
      <PageHeader
        v-if="invoice"
        :title="`Fattura ${invoice.invoice_number}`"
        class="mb-0 flex-1"
      >
        <StatusBadge :status="invoice.status" type="invoice" />
        <button
          v-if="canMarkAsPaid()"
          type="button"
          :disabled="markingPaid"
          class="bg-sage-600 text-white hover:bg-sage-700 px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-colors disabled:opacity-60"
          @click="markAsPaid"
        >
          <CheckCircle class="w-4 h-4" />
          Segna come Pagata
        </button>
        <button
          type="button"
          class="border border-sage-200 text-sage-700 hover:bg-sage-50 px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-colors"
          @click="router.push(`/invoices/${invoiceId}/print`)"
        >
          <Printer class="w-4 h-4" />
          Stampa PDF
        </button>
        <button
          type="button"
          class="border border-sage-200 text-sage-700 hover:bg-sage-50 px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-colors"
          @click="router.push(`/invoices/${invoiceId}/edit`)"
        >
          <Pencil class="w-4 h-4" />
          Modifica
        </button>
        <button
          type="button"
          class="bg-red-600 text-white hover:bg-red-700 px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-colors"
          @click="showDeleteModal = true"
        >
          <Trash2 class="w-4 h-4" />
          Elimina
        </button>
      </PageHeader>
    </div>

    <div v-if="loading" class="text-sm text-sage-400">Caricamento...</div>
    <div v-else-if="error" class="text-sm text-red-600">{{ error }}</div>

    <template v-else-if="invoice">
      <div class="space-y-5">
          <!-- Client info -->
          <div class="glass-card rounded-xl p-6 animate-in-d1">
            <h2 class="text-xs font-semibold text-sage-500 uppercase tracking-wider mb-3">Cliente</h2>
            <p class="font-medium text-sage-900 text-base">{{ invoice.client_name }}</p>
          </div>

          <!-- Invoice details -->
          <div class="glass-card rounded-xl p-6 animate-in-d2">
            <h2 class="text-xs font-semibold text-sage-500 uppercase tracking-wider mb-3">Dettagli</h2>
            <div class="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span class="text-sage-500">Data emissione</span>
                <p class="font-medium text-sage-900 mt-0.5">{{ formatDate(invoice.issue_date) }}</p>
              </div>
              <div v-if="invoice.due_date">
                <span class="text-sage-500">Data scadenza</span>
                <p class="font-medium text-sage-900 mt-0.5">{{ formatDate(invoice.due_date) }}</p>
              </div>
              <div v-if="invoice.paid_date">
                <span class="text-sage-500">Data pagamento</span>
                <p class="font-medium text-sage-600 mt-0.5">{{ formatDate(invoice.paid_date) }}</p>
              </div>
              <div>
                <span class="text-sage-500">Metodo di pagamento</span>
                <p class="font-medium text-sage-900 mt-0.5">
                  {{ PAYMENT_METHOD_LABELS[invoice.payment_method] ?? invoice.payment_method }}
                </p>
              </div>
            </div>
            <div v-if="invoice.notes" class="mt-4 pt-4 border-t border-sage-100">
              <span class="text-sm text-sage-500">Note</span>
              <p class="text-sm text-sage-700 mt-0.5 whitespace-pre-wrap">{{ invoice.notes }}</p>
            </div>
          </div>

          <!-- Lines -->
          <div class="glass-card rounded-xl animate-in-d3">
            <div class="px-6 py-4 border-b border-sage-100">
              <h2 class="text-sm font-semibold text-sage-800">Righe fattura</h2>
            </div>
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-sage-100">
                  <th class="px-6 py-3 text-left text-xs font-medium text-sage-500 uppercase">Descrizione</th>
                  <th class="px-6 py-3 text-right text-xs font-medium text-sage-500 uppercase">Qtà</th>
                  <th class="px-6 py-3 text-right text-xs font-medium text-sage-500 uppercase">Prezzo unitario</th>
                  <th class="px-6 py-3 text-right text-xs font-medium text-sage-500 uppercase">IVA %</th>
                  <th class="px-6 py-3 text-right text-xs font-medium text-sage-500 uppercase">Totale</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-sage-100/50">
                <tr v-for="line in invoice.lines" :key="line.id" class="hover:bg-sage-50/40 transition-colors">
                  <td class="px-6 py-3 text-sage-900">{{ line.description }}</td>
                  <td class="px-6 py-3 text-right text-sage-500">{{ line.quantity }}</td>
                  <td class="px-6 py-3 text-right text-sage-500">{{ formatCurrency(line.unit_price) }}</td>
                  <td class="px-6 py-3 text-right text-sage-500">{{ line.vat_rate }}%</td>
                  <td class="px-6 py-3 text-right font-medium text-sage-900">
                    {{ formatCurrency(line.line_total) }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Riepilogo importi -->
          <div class="glass-card rounded-2xl p-5 animate-in-d4">
            <div class="flex flex-wrap items-end gap-x-7 gap-y-3">
              <div>
                <p class="text-xs text-sage-400 uppercase tracking-wide mb-0.5">Netto</p>
                <p class="text-sm font-semibold text-sage-700">{{ formatCurrency(invoice.total_net) }}</p>
              </div>
              <div>
                <p class="text-xs text-sage-400 uppercase tracking-wide mb-0.5">IVA</p>
                <p class="text-sm font-semibold text-sage-700">{{ formatCurrency(invoice.total_tax) }}</p>
              </div>
              <div>
                <p class="text-xs text-sage-400 uppercase tracking-wide mb-0.5">Lordo</p>
                <p class="text-sm font-semibold text-sage-700">{{ formatCurrency(invoice.total_gross) }}</p>
              </div>
              <div v-if="invoice.apply_enpap && invoice.contributo_enpap > 0">
                <p class="text-xs text-sage-400 uppercase tracking-wide mb-0.5">ENPAP 2%</p>
                <p class="text-sm font-semibold text-sage-700">+ {{ formatCurrency(invoice.contributo_enpap) }}</p>
              </div>
              <div v-if="invoice.ritenuta_acconto > 0">
                <p class="text-xs text-sage-400 uppercase tracking-wide mb-0.5">Ritenuta 20%</p>
                <p class="text-sm font-semibold text-red-600">− {{ formatCurrency(invoice.ritenuta_acconto) }}</p>
              </div>
              <div v-if="invoice.marca_da_bollo">
                <p class="text-xs text-sage-400 uppercase tracking-wide mb-0.5">Bollo</p>
                <p class="text-sm font-semibold text-sage-700">+ {{ formatCurrency(2) }}</p>
              </div>
              <div class="ml-auto text-right">
                <p class="text-xs text-sage-400 uppercase tracking-wide mb-0.5">Totale dovuto</p>
                <p class="text-2xl font-bold text-sage-900">{{ formatCurrency(invoice.total_due) }}</p>
              </div>
            </div>
          </div>
      </div>
    </template>

    <ConfirmModal
      :open="showDeleteModal"
      title="Elimina fattura"
      :message="`Sei sicuro di voler eliminare la fattura ${invoice?.invoice_number}?`"
      @confirm="handleDelete"
      @cancel="showDeleteModal = false"
    />
    </div>
  </div>
</template>

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
    <div class="flex items-center gap-3 mb-6">
      <button
        type="button"
        class="p-1.5 text-gray-400 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
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
          class="bg-green-600 text-white hover:bg-green-700 px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-colors disabled:opacity-60"
          @click="markAsPaid"
        >
          <CheckCircle class="w-4 h-4" />
          Segna come Pagata
        </button>
        <button
          type="button"
          class="border border-gray-300 text-gray-700 hover:bg-gray-50 px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-colors"
          @click="router.push(`/invoices/${invoiceId}/print`)"
        >
          <Printer class="w-4 h-4" />
          Stampa PDF
        </button>
        <button
          type="button"
          class="border border-gray-300 text-gray-700 hover:bg-gray-50 px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-colors"
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

    <div v-if="loading" class="text-sm text-gray-400">Caricamento...</div>
    <div v-else-if="error" class="text-sm text-red-600">{{ error }}</div>

    <template v-else-if="invoice">
      <div class="grid grid-cols-3 gap-6">
        <div class="col-span-2 space-y-6">
          <!-- Client info -->
          <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
            <h2 class="text-sm font-semibold text-gray-500 uppercase tracking-wider mb-3">Cliente</h2>
            <p class="font-medium text-gray-900 text-base">{{ invoice.client_name }}</p>
          </div>

          <!-- Invoice details -->
          <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
            <h2 class="text-sm font-semibold text-gray-500 uppercase tracking-wider mb-3">Dettagli</h2>
            <div class="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span class="text-gray-500">Data emissione</span>
                <p class="font-medium text-gray-900 mt-0.5">{{ formatDate(invoice.issue_date) }}</p>
              </div>
              <div v-if="invoice.due_date">
                <span class="text-gray-500">Data scadenza</span>
                <p class="font-medium text-gray-900 mt-0.5">{{ formatDate(invoice.due_date) }}</p>
              </div>
              <div v-if="invoice.paid_date">
                <span class="text-gray-500">Data pagamento</span>
                <p class="font-medium text-green-700 mt-0.5">{{ formatDate(invoice.paid_date) }}</p>
              </div>
              <div>
                <span class="text-gray-500">Metodo di pagamento</span>
                <p class="font-medium text-gray-900 mt-0.5">
                  {{ PAYMENT_METHOD_LABELS[invoice.payment_method] ?? invoice.payment_method }}
                </p>
              </div>
            </div>
            <div v-if="invoice.notes" class="mt-4 pt-4 border-t border-gray-50">
              <span class="text-sm text-gray-500">Note</span>
              <p class="text-sm text-gray-700 mt-0.5 whitespace-pre-wrap">{{ invoice.notes }}</p>
            </div>
          </div>

          <!-- Lines -->
          <div class="bg-white rounded-xl border border-gray-100 shadow-sm">
            <div class="px-6 py-4 border-b border-gray-100">
              <h2 class="text-sm font-semibold text-gray-900">Righe fattura</h2>
            </div>
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-gray-50">
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Descrizione</th>
                  <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">Qtà</th>
                  <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">Prezzo unitario</th>
                  <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">IVA %</th>
                  <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">Totale</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-50">
                <tr v-for="line in invoice.lines" :key="line.id" class="hover:bg-gray-50">
                  <td class="px-6 py-3 text-gray-900">
                    {{ line.description }}
                    <span v-if="line.service_id" class="text-xs text-gray-400 ml-1">({{ line.service_id }})</span>
                  </td>
                  <td class="px-6 py-3 text-right text-gray-500">{{ line.quantity }}</td>
                  <td class="px-6 py-3 text-right text-gray-500">{{ formatCurrency(line.unit_price) }}</td>
                  <td class="px-6 py-3 text-right text-gray-500">{{ line.vat_rate }}%</td>
                  <td class="px-6 py-3 text-right font-medium text-gray-900">
                    {{ formatCurrency(line.line_total) }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- Tax summary -->
        <div class="col-span-1">
          <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-5 sticky top-6">
            <h2 class="text-sm font-semibold text-gray-900 mb-4">Riepilogo importi</h2>
            <div class="space-y-2 text-sm">
              <div class="flex justify-between text-gray-500">
                <span>Totale netto</span>
                <span>{{ formatCurrency(invoice.total_net) }}</span>
              </div>
              <div class="flex justify-between text-gray-500">
                <span>IVA totale</span>
                <span>{{ formatCurrency(invoice.total_tax) }}</span>
              </div>
              <div class="flex justify-between text-gray-500">
                <span>Totale lordo</span>
                <span>{{ formatCurrency(invoice.total_gross) }}</span>
              </div>
              <div v-if="invoice.apply_enpap && invoice.contributo_enpap > 0" class="flex justify-between text-gray-500">
                <span>Contributo ENPAP (2%)</span>
                <span>+ {{ formatCurrency(invoice.contributo_enpap) }}</span>
              </div>
              <div v-if="invoice.ritenuta_acconto > 0" class="flex justify-between text-gray-500">
                <span>Ritenuta d'acconto (20%)</span>
                <span class="text-red-600">- {{ formatCurrency(invoice.ritenuta_acconto) }}</span>
              </div>
              <div v-if="invoice.marca_da_bollo" class="flex justify-between text-gray-500">
                <span>Marca da bollo</span>
                <span>+ {{ formatCurrency(2) }}</span>
              </div>
              <div class="border-t border-gray-100 pt-2 mt-2 flex justify-between font-semibold text-gray-900 text-base">
                <span>Totale dovuto</span>
                <span>{{ formatCurrency(invoice.total_due) }}</span>
              </div>
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
</template>

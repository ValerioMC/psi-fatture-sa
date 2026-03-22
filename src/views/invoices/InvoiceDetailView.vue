<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Pencil, Trash2, CheckCircle, ArrowLeft, Printer, User, Calendar, CreditCard, X } from 'lucide-vue-next'
import { useInvoicesStore } from '@/stores/invoices'
import type { Invoice } from '@/types'
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

const STATUS_CONFIG: Record<string, { label: string; bg: string; text: string; dot: string; gradient: string }> = {
  draft:     { label: 'Bozza',     bg: 'bg-warm-100/80',   text: 'text-warm-600',  dot: 'bg-warm-400',  gradient: 'linear-gradient(90deg, #9ca3af, #6b7280)' },
  issued:    { label: 'Emessa',    bg: 'bg-ocean-100/80',  text: 'text-ocean-700', dot: 'bg-ocean-500', gradient: 'linear-gradient(90deg, #4f46e5, #4338ca)' },
  paid:      { label: 'Pagata',    bg: 'bg-emerald-100/80', text: 'text-emerald-700', dot: 'bg-emerald-500', gradient: 'linear-gradient(90deg, #059669, #047857)' },
  overdue:   { label: 'Scaduta',   bg: 'bg-red-100/80',    text: 'text-red-700',   dot: 'bg-red-500',   gradient: 'linear-gradient(90deg, #ef4444, #b91c1c)' },
  cancelled: { label: 'Annullata', bg: 'bg-warm-100/60',   text: 'text-warm-500',  dot: 'bg-warm-400',  gradient: 'linear-gradient(90deg, #d6d3d1, #a8a29e)' },
}

const statusConfig = computed(() =>
  invoice.value ? (STATUS_CONFIG[invoice.value.status] ?? STATUS_CONFIG.draft) : STATUS_CONFIG.draft,
)

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

      <!-- Back + action bar -->
      <div class="flex items-center gap-3 mb-6 animate-in">
        <!-- Breadcrumb -->
        <button
          type="button"
          class="flex items-center gap-1.5 text-sm text-sage-400 hover:text-sage-700 transition-colors cursor-pointer"
          @click="router.push('/invoices')"
        >
          <ArrowLeft class="w-4 h-4" />
          Fatture
        </button>
        <span class="text-sage-200">/</span>
        <span v-if="invoice" class="text-sm font-semibold text-sage-700 font-mono">{{ invoice.invoice_number }}</span>

        <!-- Actions -->
        <div class="ml-auto flex items-center gap-2">
          <button
            v-if="canMarkAsPaid()"
            type="button"
            :disabled="markingPaid"
            class="group relative overflow-hidden text-white font-medium px-4 py-2 rounded-xl text-sm flex items-center gap-2 transition-all duration-200 disabled:opacity-60 cursor-pointer focus:outline-none"
            style="background: linear-gradient(135deg, #1e1b4b, #4338ca); box-shadow: 0 4px 20px rgba(67, 56, 202, 0.4);"
            @click="markAsPaid"
          >
            <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 transform -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700" aria-hidden="true" />
            <CheckCircle class="w-4 h-4 relative z-10" />
            <span class="relative z-10">{{ markingPaid ? 'Aggiornamento…' : 'Segna come Pagata' }}</span>
          </button>
          <button
            type="button"
            class="border border-sage-200 text-sage-700 hover:bg-sage-50 px-3.5 py-2 rounded-xl text-sm font-medium flex items-center gap-2 transition-colors cursor-pointer"
            @click="router.push(`/invoices/${invoiceId}/print`)"
          >
            <Printer class="w-4 h-4" />
            Stampa
          </button>
          <button
            type="button"
            class="border border-sage-200 text-sage-700 hover:bg-sage-50 px-3.5 py-2 rounded-xl text-sm font-medium flex items-center gap-2 transition-colors cursor-pointer"
            @click="router.push(`/invoices/${invoiceId}/edit`)"
          >
            <Pencil class="w-4 h-4" />
            Modifica
          </button>
          <button
            type="button"
            class="p-2 text-sage-400 hover:text-red-500 hover:bg-red-50 rounded-xl text-sm flex items-center gap-2 transition-colors cursor-pointer"
            @click="showDeleteModal = true"
          >
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Loading / error -->
      <div v-if="loading" class="flex flex-col items-center justify-center py-20 gap-3">
        <div class="w-8 h-8 rounded-full border-2 border-sage-200 border-t-sage-500 animate-spin" />
        <p class="text-sm text-sage-400">Caricamento fattura…</p>
      </div>
      <div v-else-if="error" class="rounded-2xl bg-red-50 border border-red-200 px-5 py-4 text-sm text-red-700 flex items-center gap-2">
        <X class="w-4 h-4 shrink-0 text-red-400" />
        {{ error }}
      </div>

      <template v-else-if="invoice">

        <!-- Document header card -->
        <div class="glass-card rounded-2xl overflow-hidden shadow-sm mb-5 animate-in">
          <!-- Status accent bar -->
          <div class="h-1.5" :style="{ background: statusConfig.gradient }" />

          <div class="p-6">
            <div class="flex items-start justify-between gap-6">
              <!-- Left: invoice identity -->
              <div>
                <p class="text-[10px] text-sage-400 uppercase tracking-wider mb-1">Numero fattura</p>
                <h1 class="text-3xl font-bold text-sage-900 tracking-tight leading-none font-mono mb-3">
                  {{ invoice.invoice_number }}
                </h1>
                <span
                  class="inline-flex items-center gap-1.5 text-xs font-semibold px-3 py-1 rounded-full"
                  :class="[statusConfig.bg, statusConfig.text]"
                >
                  <span class="w-1.5 h-1.5 rounded-full" :class="statusConfig.dot" />
                  {{ statusConfig.label }}
                </span>
              </div>

              <!-- Right: client -->
              <div class="text-right">
                <p class="text-[10px] text-sage-400 uppercase tracking-wider mb-2">Paziente</p>
                <div class="flex items-center gap-2.5 justify-end">
                  <p class="text-base font-semibold text-sage-900 leading-tight">{{ invoice.client_name }}</p>
                  <div class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #4f46e5, #4338ca)">
                    <User class="w-4 h-4 text-white" />
                  </div>
                </div>
              </div>
            </div>

            <!-- Dates row -->
            <div class="grid grid-cols-4 gap-4 mt-5 pt-5 border-t border-sage-100/60">
              <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-lg bg-sage-50 flex items-center justify-center shrink-0">
                  <Calendar class="w-3.5 h-3.5 text-sage-400" />
                </div>
                <div>
                  <p class="text-[10px] text-sage-400 uppercase tracking-wider">Emessa il</p>
                  <p class="text-sm font-semibold text-sage-800">{{ formatDate(invoice.issue_date) }}</p>
                </div>
              </div>
              <div v-if="invoice.due_date" class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-lg bg-sage-50 flex items-center justify-center shrink-0">
                  <Calendar class="w-3.5 h-3.5 text-sage-400" />
                </div>
                <div>
                  <p class="text-[10px] text-sage-400 uppercase tracking-wider">Scadenza</p>
                  <p class="text-sm font-semibold" :class="invoice.status === 'overdue' ? 'text-red-600' : 'text-sage-800'">
                    {{ formatDate(invoice.due_date) }}
                  </p>
                </div>
              </div>
              <div v-if="invoice.paid_date" class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-lg bg-sage-50 flex items-center justify-center shrink-0">
                  <CheckCircle class="w-3.5 h-3.5 text-sage-500" />
                </div>
                <div>
                  <p class="text-[10px] text-sage-400 uppercase tracking-wider">Pagata il</p>
                  <p class="text-sm font-semibold text-sage-700">{{ formatDate(invoice.paid_date) }}</p>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-lg bg-sage-50 flex items-center justify-center shrink-0">
                  <CreditCard class="w-3.5 h-3.5 text-sage-400" />
                </div>
                <div>
                  <p class="text-[10px] text-sage-400 uppercase tracking-wider">Pagamento</p>
                  <p class="text-sm font-semibold text-sage-800">
                    {{ PAYMENT_METHOD_LABELS[invoice.payment_method] ?? invoice.payment_method }}
                  </p>
                </div>
              </div>
            </div>

            <!-- Notes -->
            <div v-if="invoice.notes" class="mt-4 pt-4 border-t border-sage-100/60">
              <p class="text-[10px] text-sage-400 uppercase tracking-wider mb-1.5">Note</p>
              <p class="text-sm text-sage-600 whitespace-pre-wrap leading-relaxed">{{ invoice.notes }}</p>
            </div>
          </div>
        </div>

        <!-- Line items table -->
        <div class="glass-card rounded-2xl shadow-sm overflow-hidden mb-5 animate-in-d1">
          <div class="px-6 py-4 border-b border-sage-100/60 flex items-center justify-between bg-sage-50/30">
            <h2 class="text-xs font-semibold text-sage-500 uppercase tracking-wider">Prestazioni</h2>
            <span class="text-[10px] text-sage-400 font-medium bg-sage-100 px-2 py-0.5 rounded-full">
              {{ invoice.lines.length }} {{ invoice.lines.length === 1 ? 'riga' : 'righe' }}
            </span>
          </div>
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b border-sage-100/40">
                <th class="px-6 py-3 text-left text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Descrizione</th>
                <th class="px-6 py-3 text-right text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Qtà</th>
                <th class="px-6 py-3 text-right text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Prezzo unit.</th>
                <th class="px-6 py-3 text-right text-[10px] font-semibold text-sage-400 uppercase tracking-wider">IVA</th>
                <th class="px-6 py-3 text-right text-[10px] font-semibold text-sage-400 uppercase tracking-wider">Totale</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-sage-50">
              <tr
                v-for="line in invoice.lines"
                :key="line.id"
                class="hover:bg-sage-50/40 transition-colors"
              >
                <td class="px-6 py-3.5 text-sage-900 font-medium">{{ line.description }}</td>
                <td class="px-6 py-3.5 text-right text-sage-500 tabular-nums">{{ line.quantity }}</td>
                <td class="px-6 py-3.5 text-right text-sage-500 tabular-nums">{{ formatCurrency(line.unit_price) }}</td>
                <td class="px-6 py-3.5 text-right">
                  <span class="text-xs px-1.5 py-0.5 rounded-full" :class="line.vat_rate > 0 ? 'bg-ocean-50 text-ocean-600' : 'bg-sage-50 text-sage-400'">
                    {{ line.vat_rate > 0 ? line.vat_rate + '%' : 'esente' }}
                  </span>
                </td>
                <td class="px-6 py-3.5 text-right font-semibold text-sage-900 tabular-nums">{{ formatCurrency(line.line_total) }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Totals panel -->
        <div class="glass-card rounded-2xl shadow-sm overflow-hidden animate-in-d2">
          <div class="flex justify-end">
            <div class="w-full max-w-sm p-6 space-y-2.5">
              <div class="flex items-center justify-between text-sm">
                <span class="text-sage-500">Imponibile netto</span>
                <span class="font-medium text-sage-800 tabular-nums">{{ formatCurrency(invoice.total_net) }}</span>
              </div>
              <div v-if="invoice.total_tax > 0" class="flex items-center justify-between text-sm">
                <span class="text-sage-500">IVA</span>
                <span class="font-medium text-sage-800 tabular-nums">+ {{ formatCurrency(invoice.total_tax) }}</span>
              </div>
              <div v-if="invoice.apply_enpap && invoice.contributo_enpap > 0" class="flex items-center justify-between text-sm">
                <span class="text-sage-500">Contributo ENPAP 2%</span>
                <span class="font-medium text-sage-800 tabular-nums">+ {{ formatCurrency(invoice.contributo_enpap) }}</span>
              </div>
              <div v-if="invoice.marca_da_bollo" class="flex items-center justify-between text-sm">
                <span class="text-sage-500">Marca da bollo</span>
                <span class="font-medium text-sage-800 tabular-nums">+ {{ formatCurrency(2) }}</span>
              </div>
              <div v-if="invoice.ritenuta_acconto > 0" class="flex items-center justify-between text-sm">
                <span class="text-sage-500">Ritenuta d'acconto 20%</span>
                <span class="font-medium text-red-500 tabular-nums">− {{ formatCurrency(invoice.ritenuta_acconto) }}</span>
              </div>

              <!-- Total due highlight -->
              <div class="pt-4 mt-2 border-t border-sage-100">
                <div class="rounded-2xl px-5 py-4 flex items-center justify-between" :style="{ background: statusConfig.gradient }">
                  <span class="text-sm font-semibold text-white/80">Totale dovuto</span>
                  <span class="text-2xl font-bold text-white tracking-tight tabular-nums">{{ formatCurrency(invoice.total_due) }}</span>
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
  </div>
</template>

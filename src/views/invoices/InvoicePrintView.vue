<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Printer, ArrowLeft } from 'lucide-vue-next'
import { save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import { getInvoice, getClient, getConfig } from '@/api'
import type { Invoice, Client, ProfessionalConfig } from '@/types'
import { formatCurrency, formatDateLong } from '@/utils/format'
import { buildInvoicePdf } from '@/utils/invoice-pdf'

const route = useRoute()
const router = useRouter()
const invoiceId = Number(route.params.id)

const invoice = ref<Invoice | null>(null)
const client = ref<Client | null>(null)
const config = ref<ProfessionalConfig | null>(null)
const loading = ref(true)
const saving = ref(false)
const error = ref<string | null>(null)

onMounted(async () => {
  try {
    invoice.value = await getInvoice(invoiceId)
    const [c, cfg] = await Promise.all([
      getClient(invoice.value.client_id),
      getConfig(),
    ])
    client.value = c
    config.value = cfg
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})

const isForfettario = computed(() => config.value?.tax_regime === 'forfettario')
const hasIva = computed(() => (invoice.value?.total_tax ?? 0) > 0)
const professionalFullName = computed(() => {
  if (!config.value) return ''
  return `${config.value.title} ${config.value.first_name} ${config.value.last_name}`.trim()
})

const notes = computed(() => {
  const lines: string[] = []
  if (!invoice.value || !config.value) return lines

  if (isForfettario.value) {
    lines.push(
      'Operazione effettuata ai sensi dell\'art. 1, c. 54-89 Legge n. 190/2014 – Regime Forfettario. Imposta non dovuta.',
    )
  }
  if (!hasIva.value && !isForfettario.value) {
    lines.push(
      'Operazione esente da IVA ai sensi dell\'art. 10, n. 18, DPR 633/72.',
    )
  }
  if (invoice.value.apply_enpap && invoice.value.contributo_enpap > 0) {
    lines.push(
      `Contributo integrativo ENPAP 2% (${formatCurrency(invoice.value.contributo_enpap)}) addebitato al cliente ai sensi dell'art. 8, L. 21/86.`,
    )
  }
  if (invoice.value.ritenuta_acconto > 0) {
    lines.push(
      `Si richiede di operare una ritenuta d'acconto del 20% pari a ${formatCurrency(invoice.value.ritenuta_acconto)}.`,
    )
  }
  if (invoice.value.marca_da_bollo) {
    lines.push(
      'Marca da bollo virtuale di € 2,00 assolta ai sensi del D.M. 17/06/2014 (importo > € 77,47 e operazione esente IVA).',
    )
  }
  return lines
})

async function handlePrint() {
  if (!invoice.value || !client.value || !config.value) return
  saving.value = true
  error.value = null
  try {
    const defaultFilename = `fattura-${invoice.value.invoice_number}-${invoice.value.year}.pdf`
    const savePath = await save({
      defaultPath: defaultFilename,
      filters: [{ name: 'PDF', extensions: ['pdf'] }],
    })
    if (!savePath) return
    const pdf = buildInvoicePdf(invoice.value, client.value, config.value, notes.value)
    await writeFile(savePath, new Uint8Array(pdf.output('arraybuffer')))
  } catch (e) {
    error.value = `Errore durante il salvataggio: ${String(e)}`
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div>
    <!-- Toolbar — hidden when printing -->
    <div class="print:hidden fixed top-4 right-4 z-50 flex gap-2">
      <button
        type="button"
        class="flex items-center gap-2 bg-white border border-gray-300 text-gray-700 hover:bg-gray-50 px-4 py-2 rounded-lg text-sm font-medium shadow-lg transition-colors"
        @click="router.push(`/invoices/${invoiceId}`)"
      >
        <ArrowLeft class="w-4 h-4" />
        Torna alla fattura
      </button>
      <button
        v-if="!loading && invoice"
        type="button"
        :disabled="saving"
        class="flex items-center gap-2 bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-60 px-4 py-2 rounded-lg text-sm font-medium shadow-lg transition-colors"
        @click="handlePrint"
      >
        <Printer class="w-4 h-4" />
        {{ saving ? 'Salvataggio...' : 'Salva PDF' }}
      </button>
    </div>

    <div v-if="loading" class="flex items-center justify-center h-screen text-gray-400 text-sm">
      Caricamento...
    </div>
    <div v-else-if="error" class="flex items-center justify-center h-screen text-red-500 text-sm">
      {{ error }}
    </div>

    <div v-if="error" class="fixed bottom-4 right-4 bg-red-50 border border-red-200 text-red-700 text-sm px-4 py-3 rounded-lg shadow-lg print:hidden">
      {{ error }}
    </div>

    <!-- Invoice document -->
    <div
      v-else-if="invoice && client && config"
      class="invoice-page bg-white mx-auto my-8 print:my-0 p-16 print:p-12 max-w-3xl shadow-lg print:shadow-none"
    >
      <!-- ─── Header: professional + invoice number ─── -->
      <div class="flex justify-between items-start mb-10">
        <!-- Professional details -->
        <div class="text-sm leading-relaxed">
          <p class="text-xl font-bold text-gray-900 mb-1">{{ professionalFullName }}</p>
          <p class="text-gray-600">P.IVA: {{ config.vat_number }}</p>
          <p class="text-gray-600">C.F.: {{ config.fiscal_code }}</p>
          <p class="text-gray-600">
            {{ config.address }}, {{ config.zip_code }} {{ config.city }} ({{ config.province }})
          </p>
          <p v-if="config.phone" class="text-gray-600">Tel: {{ config.phone }}</p>
          <p v-if="config.pec_email" class="text-gray-600">PEC: {{ config.pec_email }}</p>
          <p v-if="config.albo_number" class="text-gray-600">
            Albo: n. {{ config.albo_number }} – {{ config.albo_region }}
          </p>
        </div>

        <!-- Invoice number box -->
        <div class="text-right">
          <div class="border-2 border-gray-900 rounded-lg px-5 py-4 inline-block">
            <p class="text-xs font-semibold text-gray-500 uppercase tracking-widest mb-0.5">Fattura</p>
            <p class="text-2xl font-bold text-gray-900">
              {{ invoice.invoice_number }}/{{ invoice.year }}
            </p>
          </div>
          <p class="text-sm text-gray-500 mt-2">
            Data: <span class="font-medium text-gray-800">{{ formatDateLong(invoice.issue_date) }}</span>
          </p>
          <p v-if="invoice.due_date" class="text-sm text-gray-500">
            Scadenza: <span class="font-medium text-gray-800">{{ formatDateLong(invoice.due_date) }}</span>
          </p>
        </div>
      </div>

      <hr class="border-gray-200 mb-8" />

      <!-- ─── Client block ─── -->
      <div class="mb-8">
        <p class="text-xs font-semibold text-gray-400 uppercase tracking-widest mb-2">Spett.le</p>
        <div class="bg-gray-50 rounded-lg px-5 py-4 text-sm leading-relaxed">
          <p class="font-semibold text-gray-900 text-base">
            {{ client.first_name }} {{ client.last_name }}
          </p>
          <p v-if="client.fiscal_code" class="text-gray-600">C.F.: {{ client.fiscal_code }}</p>
          <p v-if="client.vat_number" class="text-gray-600">P.IVA: {{ client.vat_number }}</p>
          <p class="text-gray-600">
            {{ client.address }}, {{ client.zip_code }} {{ client.city }} ({{ client.province }})
          </p>
          <p v-if="client.email" class="text-gray-600">{{ client.email }}</p>
        </div>
      </div>

      <!-- ─── Lines table ─── -->
      <table class="w-full text-sm mb-6 border-collapse">
        <thead>
          <tr class="border-b-2 border-gray-900">
            <th class="py-2 text-left font-semibold text-gray-900 pr-4">Descrizione</th>
            <th class="py-2 text-center font-semibold text-gray-900 w-12">Qtà</th>
            <th class="py-2 text-right font-semibold text-gray-900 w-28">Prezzo unit.</th>
            <th class="py-2 text-right font-semibold text-gray-900 w-16">IVA%</th>
            <th class="py-2 text-right font-semibold text-gray-900 w-28">Totale</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="line in invoice.lines"
            :key="line.id"
            class="border-b border-gray-100"
          >
            <td class="py-3 text-gray-900 pr-4">{{ line.description }}</td>
            <td class="py-3 text-center text-gray-600">{{ line.quantity }}</td>
            <td class="py-3 text-right text-gray-600">{{ formatCurrency(line.unit_price) }}</td>
            <td class="py-3 text-right text-gray-600">{{ line.vat_rate }}%</td>
            <td class="py-3 text-right font-medium text-gray-900">{{ formatCurrency(line.line_total) }}</td>
          </tr>
        </tbody>
      </table>

      <!-- ─── Totals ─── -->
      <div class="flex justify-end mb-8">
        <div class="w-72 text-sm space-y-1.5">
          <div class="flex justify-between text-gray-600">
            <span>Totale netto</span>
            <span>{{ formatCurrency(invoice.total_net) }}</span>
          </div>
          <div v-if="hasIva" class="flex justify-between text-gray-600">
            <span>IVA</span>
            <span>{{ formatCurrency(invoice.total_tax) }}</span>
          </div>
          <div v-if="invoice.apply_enpap && invoice.contributo_enpap > 0" class="flex justify-between text-gray-600">
            <span>Contributo ENPAP (2%)</span>
            <span>{{ formatCurrency(invoice.contributo_enpap) }}</span>
          </div>
          <div class="flex justify-between text-gray-600 border-t border-gray-200 pt-1.5">
            <span>Totale lordo</span>
            <span>{{ formatCurrency(invoice.total_gross) }}</span>
          </div>
          <div v-if="invoice.ritenuta_acconto > 0" class="flex justify-between text-gray-600">
            <span>Ritenuta d'acconto (20%)</span>
            <span>– {{ formatCurrency(invoice.ritenuta_acconto) }}</span>
          </div>
          <div v-if="invoice.marca_da_bollo" class="flex justify-between text-gray-600">
            <span>Marca da bollo</span>
            <span>+ {{ formatCurrency(2) }}</span>
          </div>
          <div class="flex justify-between font-bold text-gray-900 text-base border-t-2 border-gray-900 pt-2 mt-1">
            <span>Totale dovuto</span>
            <span>{{ formatCurrency(invoice.total_due) }}</span>
          </div>
        </div>
      </div>

      <!-- ─── Payment info ─── -->
      <div v-if="config.iban" class="bg-gray-50 rounded-lg px-5 py-3 text-sm mb-6">
        <span class="font-semibold text-gray-700">Pagamento:</span>
        <span class="text-gray-600 ml-2">{{ config.iban }}</span>
        <span v-if="invoice.payment_method === 'bonifico'" class="text-gray-500 ml-1">(Bonifico bancario)</span>
      </div>

      <!-- ─── Legal notes ─── -->
      <div v-if="notes.length > 0" class="text-xs text-gray-500 space-y-1 border-t border-gray-100 pt-4">
        <p v-for="(note, i) in notes" :key="i">{{ note }}</p>
      </div>

      <!-- ─── Invoice notes ─── -->
      <div v-if="invoice.notes" class="mt-4 text-sm text-gray-600">
        <span class="font-medium">Note:</span> {{ invoice.notes }}
      </div>
    </div>
  </div>
</template>

<style>
@media print {
  body {
    margin: 0;
    padding: 0;
  }
  .invoice-page {
    max-width: 100% !important;
    margin: 0 !important;
    padding: 2cm !important;
    box-shadow: none !important;
  }
  @page {
    margin: 0;
    size: A4;
  }
}
</style>

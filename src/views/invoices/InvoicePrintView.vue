<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Download, ArrowLeft, Loader2 } from 'lucide-vue-next'
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
  return [config.value.title, config.value.first_name, config.value.last_name]
    .filter(Boolean).join(' ')
})

const notes = computed(() => {
  const lines: string[] = []
  if (!invoice.value || !config.value) return lines
  if (isForfettario.value)
    lines.push("Operazione effettuata ai sensi dell'art. 1, c. 54-89 Legge n. 190/2014 – Regime Forfettario. Imposta non dovuta.")
  if (!hasIva.value && !isForfettario.value)
    lines.push("Operazione esente da IVA ai sensi dell'art. 10, n. 18, DPR 633/72.")
  if (invoice.value.apply_enpap && invoice.value.contributo_enpap > 0)
    lines.push(`Contributo integrativo ENPAP 2% (${formatCurrency(invoice.value.contributo_enpap)}) addebitato al cliente ai sensi dell'art. 8, L. 21/86.`)
  if (invoice.value.ritenuta_acconto > 0)
    lines.push(`Si richiede di operare una ritenuta d'acconto del 20% pari a ${formatCurrency(invoice.value.ritenuta_acconto)}.`)
  if (invoice.value.marca_da_bollo)
    lines.push('Marca da bollo virtuale di € 2,00 assolta ai sensi del D.M. 17/06/2014 (importo > € 77,47 e operazione esente IVA).')
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
  <div style="background: linear-gradient(135deg, #f6f8f6 0%, #e8dfd3 100%); min-height: 100vh;">
    <!-- ── Toolbar ── -->
    <div class="print:hidden fixed top-5 right-5 z-50 flex items-center gap-2">
      <button
        type="button"
        class="flex items-center gap-2 glass-card text-sage-700 hover:text-sage-900 px-4 py-2 rounded-xl text-sm font-medium shadow-sm transition-all"
        @click="router.push(`/invoices/${invoiceId}`)"
      >
        <ArrowLeft class="w-4 h-4" />
        Indietro
      </button>
      <button
        v-if="!loading && invoice"
        type="button"
        :disabled="saving"
        class="group relative overflow-hidden flex items-center gap-2 text-white font-semibold px-4 py-2 rounded-xl text-sm shadow-sm transition-all disabled:opacity-60"
        style="background: linear-gradient(135deg, #5d8062, #0c8aeb);"
        @click="handlePrint"
      >
        <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 transform -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700" aria-hidden="true" />
        <Loader2 v-if="saving" class="w-4 h-4 animate-spin relative z-10" />
        <Download v-else class="w-4 h-4 relative z-10" />
        <span class="relative z-10">{{ saving ? 'Salvataggio...' : 'Salva PDF' }}</span>
      </button>
    </div>

    <!-- ── Loading ── -->
    <div v-if="loading" class="flex items-center justify-center min-h-screen">
      <div class="flex flex-col items-center gap-3">
        <div class="w-8 h-8 rounded-full border-2 border-sage-200 border-t-sage-500 animate-spin" />
        <p class="text-sm text-sage-400">Caricamento fattura...</p>
      </div>
    </div>

    <!-- ── Error toast ── -->
    <div
      v-if="error"
      class="print:hidden fixed bottom-5 right-5 glass-card border border-red-200 text-red-700 text-sm px-4 py-3 rounded-xl shadow-lg"
    >
      {{ error }}
    </div>

    <!-- ── Invoice document ── -->
    <div
      v-if="!loading && invoice && client && config"
      class="invoice-page bg-white mx-auto my-10 print:my-0 max-w-[780px] shadow-xl print:shadow-none overflow-hidden"
      style="border-radius: 16px;"
    >
      <!-- Accent bar -->
      <div
        class="h-1.5 w-full print:h-1"
        style="background: linear-gradient(to right, #5d8062, #0c8aeb, #d4a017)"
      />

      <div class="p-14 print:p-12">
        <!-- ── Header ── -->
        <div class="flex justify-between items-start mb-10">
          <!-- Professional details -->
          <div>
            <p class="text-xl font-bold text-sage-900 mb-2 tracking-tight">{{ professionalFullName }}</p>
            <div class="space-y-0.5 text-sm text-sage-500 leading-relaxed">
              <p>P.IVA: <span class="text-sage-700">{{ config.vat_number }}</span></p>
              <p>C.F.: <span class="text-sage-700">{{ config.fiscal_code }}</span></p>
              <p>{{ config.address }}, {{ config.zip_code }} {{ config.city }} ({{ config.province }})</p>
              <p v-if="config.phone">Tel: {{ config.phone }}</p>
              <p v-if="config.pec_email">PEC: {{ config.pec_email }}</p>
              <p v-if="config.albo_number">Albo: n. {{ config.albo_number }} – {{ config.albo_region }}</p>
            </div>
          </div>

          <!-- Invoice number box -->
          <div class="text-right">
            <div
              class="inline-block rounded-2xl px-6 py-4 text-center"
              style="background: linear-gradient(135deg, #f6f8f6, #e3ebe3); border: 1.5px solid rgba(93,128,98,0.2);"
            >
              <p class="text-[10px] font-bold text-sage-400 uppercase tracking-widest mb-1">Fattura</p>
              <p class="text-3xl font-bold text-sage-800 tracking-tight leading-none">
                {{ invoice.invoice_number }}<span class="text-sage-400 text-xl">/{{ invoice.year }}</span>
              </p>
            </div>
            <div class="mt-3 space-y-1 text-sm text-sage-500">
              <p>Data emissione: <span class="font-semibold text-sage-700">{{ formatDateLong(invoice.issue_date) }}</span></p>
              <p v-if="invoice.due_date">Scadenza: <span class="font-semibold text-sage-700">{{ formatDateLong(invoice.due_date) }}</span></p>
            </div>
          </div>
        </div>

        <!-- Divider -->
        <div class="h-px mb-8" style="background: linear-gradient(to right, #a1baa3, rgba(163,186,163,0.1))" />

        <!-- ── Client block ── -->
        <div class="mb-8">
          <p class="text-[10px] font-bold text-sage-400 uppercase tracking-widest mb-2.5">Spett.le</p>
          <div
            class="rounded-xl px-5 py-4 text-sm leading-relaxed"
            style="background: linear-gradient(135deg, #f6f8f6, #f0f4f0); border: 1px solid rgba(163,186,163,0.25);"
          >
            <p class="font-bold text-sage-900 text-base mb-1">{{ client.first_name }} {{ client.last_name }}</p>
            <div class="space-y-0.5 text-sage-500">
              <p v-if="client.fiscal_code">C.F.: <span class="text-sage-700">{{ client.fiscal_code }}</span></p>
              <p v-if="client.vat_number">P.IVA: <span class="text-sage-700">{{ client.vat_number }}</span></p>
              <p>{{ client.address }}, {{ client.zip_code }} {{ client.city }} ({{ client.province }})</p>
              <p v-if="client.email">{{ client.email }}</p>
            </div>
          </div>
        </div>

        <!-- ── Lines table ── -->
        <table class="w-full text-sm mb-8">
          <thead>
            <tr style="background: linear-gradient(135deg, #2f4233, #3a513e);">
              <th class="py-3 px-4 text-left text-xs font-semibold text-sage-100 uppercase tracking-wider rounded-tl-lg">Descrizione</th>
              <th class="py-3 px-3 text-center text-xs font-semibold text-sage-100 uppercase tracking-wider w-12">Qtà</th>
              <th class="py-3 px-3 text-right text-xs font-semibold text-sage-100 uppercase tracking-wider w-28">Prezzo unit.</th>
              <th class="py-3 px-3 text-right text-xs font-semibold text-sage-100 uppercase tracking-wider w-16">IVA%</th>
              <th class="py-3 px-4 text-right text-xs font-semibold text-sage-100 uppercase tracking-wider w-28 rounded-tr-lg">Totale</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(line, i) in invoice.lines"
              :key="line.id"
              :style="{ background: i % 2 === 0 ? 'rgba(246,248,246,0.6)' : 'white' }"
            >
              <td class="py-3.5 px-4 text-sage-800">{{ line.description }}</td>
              <td class="py-3.5 px-3 text-center text-sage-500">{{ line.quantity }}</td>
              <td class="py-3.5 px-3 text-right text-sage-500">{{ formatCurrency(line.unit_price) }}</td>
              <td class="py-3.5 px-3 text-right text-sage-500">{{ line.vat_rate }}%</td>
              <td class="py-3.5 px-4 text-right font-semibold text-sage-800">{{ formatCurrency(line.line_total) }}</td>
            </tr>
          </tbody>
        </table>

        <!-- ── Totals ── -->
        <div class="flex justify-end mb-8">
          <div class="w-72 text-sm space-y-2">
            <div class="flex justify-between text-sage-500">
              <span>Totale netto</span>
              <span>{{ formatCurrency(invoice.total_net) }}</span>
            </div>
            <div v-if="hasIva" class="flex justify-between text-sage-500">
              <span>IVA</span>
              <span>{{ formatCurrency(invoice.total_tax) }}</span>
            </div>
            <div v-if="invoice.apply_enpap && invoice.contributo_enpap > 0" class="flex justify-between text-sage-500">
              <span>Contributo ENPAP (2%)</span>
              <span>{{ formatCurrency(invoice.contributo_enpap) }}</span>
            </div>
            <div class="flex justify-between text-sage-600 border-t border-sage-100 pt-2">
              <span>Totale lordo</span>
              <span>{{ formatCurrency(invoice.total_gross) }}</span>
            </div>
            <div v-if="invoice.ritenuta_acconto > 0" class="flex justify-between text-sage-500">
              <span>Ritenuta d'acconto (20%)</span>
              <span class="text-red-500">– {{ formatCurrency(invoice.ritenuta_acconto) }}</span>
            </div>
            <div v-if="invoice.marca_da_bollo" class="flex justify-between text-sage-500">
              <span>Marca da bollo</span>
              <span>+ {{ formatCurrency(2) }}</span>
            </div>
            <!-- Total due -->
            <div
              class="flex justify-between items-center font-bold text-base rounded-xl px-4 py-3 mt-1"
              style="background: linear-gradient(135deg, #2f4233, #3a513e); color: white;"
            >
              <span>Totale dovuto</span>
              <span>{{ formatCurrency(invoice.total_due) }}</span>
            </div>
          </div>
        </div>

        <!-- ── Payment info ── -->
        <div
          v-if="config.iban"
          class="rounded-xl px-5 py-3.5 text-sm mb-6 flex items-center gap-3"
          style="background: linear-gradient(135deg, #f6f8f6, #e3ebe3); border: 1px solid rgba(163,186,163,0.25);"
        >
          <div
            class="w-7 h-7 rounded-lg flex items-center justify-center shrink-0"
            style="background: linear-gradient(135deg, #5d8062, #0c8aeb);"
          >
            <svg class="w-3.5 h-3.5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
            </svg>
          </div>
          <div>
            <span class="font-semibold text-sage-700">Pagamento: </span>
            <span class="text-sage-600 font-mono text-xs">{{ config.iban }}</span>
            <span v-if="invoice.payment_method === 'bonifico'" class="text-sage-400 ml-1">(Bonifico bancario)</span>
          </div>
        </div>

        <!-- ── Legal notes ── -->
        <div v-if="notes.length > 0" class="border-t border-sage-100 pt-5">
          <p class="text-[10px] font-bold text-sage-400 uppercase tracking-widest mb-3">Note legali</p>
          <div class="space-y-1.5">
            <p v-for="(note, i) in notes" :key="i" class="text-xs text-sage-400 leading-relaxed">
              {{ note }}
            </p>
          </div>
        </div>

        <!-- ── Invoice notes ── -->
        <div v-if="invoice.notes" class="mt-4 rounded-xl px-4 py-3 text-sm" style="background: rgba(163,186,163,0.12);">
          <span class="font-semibold text-sage-600">Note: </span>
          <span class="text-sage-500">{{ invoice.notes }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
@media print {
  .invoice-page {
    max-width: 100% !important;
    margin: 0 !important;
    border-radius: 0 !important;
    box-shadow: none !important;
  }
  @page {
    margin: 0;
    size: A4;
  }
}
</style>

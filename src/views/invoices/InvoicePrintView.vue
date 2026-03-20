<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, Printer } from 'lucide-vue-next'
import { getInvoice, getClient, getConfig } from '@/api'
import type { Invoice, Client, ProfessionalConfig } from '@/types'
import { formatCurrency, formatDateLong } from '@/utils/format'

const route = useRoute()
const router = useRouter()
const invoiceId = Number(route.params.id)

const invoice = ref<Invoice | null>(null)
const client = ref<Client | null>(null)
const config = ref<ProfessionalConfig | null>(null)
const loading = ref(true)
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

const clientDisplayName = computed(() => {
  if (!client.value) return ''
  if (client.value.client_type === 'azienda') return client.value.last_name
  return `${client.value.first_name} ${client.value.last_name}`.trim()
})

const PAYMENT_LABELS: Record<string, string> = {
  bonifico: 'Bonifico bancario',
  contanti: 'Contanti',
  pos: 'POS / Carta di credito',
  altro: 'Altro',
}

const PAYMENT_CONDITIONS: Record<string, string> = {
  bonifico: 'Pagamento immediato',
  contanti: 'Contestuale alla prestazione',
  pos: 'Contestuale alla prestazione',
}

const STATUS_LABELS: Record<string, string> = {
  draft: 'Bozza',
  issued: 'Emessa',
  paid: 'Pagata',
  overdue: 'Scaduta',
  cancelled: 'Annullata',
}

const legalNotes = computed((): string[] => {
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

function handlePrint() {
  window.print()
}
</script>

<template>
  <div class="print-root">

    <!-- ── Toolbar (screen only) ── -->
    <div class="print:hidden fixed top-5 right-5 z-50 flex items-center gap-2">
      <button
        type="button"
        class="flex items-center gap-2 bg-white/95 backdrop-blur-sm border border-slate-200 text-slate-600 hover:text-slate-900 px-4 py-2 rounded-xl text-sm font-medium shadow-sm transition-all"
        @click="router.push(`/invoices/${invoiceId}`)"
      >
        <ArrowLeft class="w-4 h-4" />
        Indietro
      </button>
      <button
        v-if="!loading && invoice"
        type="button"
        class="flex items-center gap-2 text-white font-semibold px-4 py-2 rounded-xl text-sm shadow-sm transition-all hover:opacity-90"
        style="background: linear-gradient(135deg, #3d6142, #0c8aeb);"
        @click="handlePrint"
      >
        <Printer class="w-4 h-4" />
        Stampa / Salva PDF
      </button>
    </div>

    <!-- ── Loading ── -->
    <div v-if="loading" class="print:hidden flex items-center justify-center min-h-screen">
      <div class="flex flex-col items-center gap-3">
        <div class="w-8 h-8 rounded-full border-2 border-sage-200 border-t-sage-500 animate-spin" />
        <p class="text-sm text-sage-400">Caricamento fattura...</p>
      </div>
    </div>

    <!-- ── Error ── -->
    <div
      v-else-if="error"
      class="print:hidden max-w-md mx-auto mt-24 bg-red-50 border border-red-200 rounded-xl p-6 text-center"
    >
      <p class="text-sm font-semibold text-red-700 mb-1">Errore nel caricamento</p>
      <p class="text-xs text-red-500 font-mono break-all">{{ error }}</p>
    </div>

    <!-- ── Invoice document ── -->
    <div
      v-else-if="invoice && client && config"
      class="invoice-doc"
    >
      <!-- Gradient top bar -->
      <div class="header-band" />

      <div class="doc-body">

        <!-- ══════════════════════════════
             HEADER
        ══════════════════════════════ -->
        <div class="doc-header">
          <!-- Left: professional info -->
          <div class="header-left">
            <div class="company-name">{{ professionalFullName }}</div>
            <div class="company-profession">
              Psicologo<template v-if="config.is_psicoanalista"> &nbsp;—&nbsp; Psicoanalista</template>
            </div>
            <div v-if="config.albo_number || config.albo_region" class="albo-info">
              <template v-if="config.albo_number">Iscriz. Albo n.&nbsp;<strong>{{ config.albo_number }}</strong></template>
              <template v-if="config.albo_number && config.albo_region">&nbsp;—&nbsp;</template>
              <template v-if="config.albo_region">Regione&nbsp;<strong>{{ config.albo_region }}</strong></template>
            </div>
            <div class="company-details">
              <template v-if="config.address">{{ config.address }}<br></template>
              {{ config.zip_code }} {{ config.city }} ({{ config.province }})<br>
              P.IVA&nbsp;{{ config.vat_number }}&nbsp;·&nbsp;C.F.&nbsp;{{ config.fiscal_code }}
              <template v-if="config.pec_email"><br>PEC:&nbsp;{{ config.pec_email }}</template>
              <template v-if="config.phone">&nbsp;·&nbsp;Tel:&nbsp;{{ config.phone }}</template>
            </div>
          </div>

          <!-- Right: invoice identity -->
          <div class="header-right">
            <div class="invoice-label">Fattura</div>
            <div class="invoice-number">
              <span class="invoice-prefix">N.</span>{{ invoice.invoice_number }}
            </div>
            <div class="invoice-meta">
              <div class="meta-row">
                <span class="meta-label">Anno</span>
                <span class="meta-value">{{ invoice.year }}</span>
              </div>
              <div class="meta-row">
                <span class="meta-label">Data emissione</span>
                <span class="meta-value">{{ formatDateLong(invoice.issue_date) }}</span>
              </div>
              <div v-if="invoice.due_date" class="meta-row">
                <span class="meta-label">Scadenza</span>
                <span class="meta-value">{{ formatDateLong(invoice.due_date) }}</span>
              </div>
            </div>
            <span :class="`status-badge status-${invoice.status}`">{{ STATUS_LABELS[invoice.status] }}</span>
          </div>
        </div>

        <hr class="doc-divider">

        <!-- ══════════════════════════════
             INFO CARD: Client + Payment
        ══════════════════════════════ -->
        <div class="info-card">
          <!-- Destinatario -->
          <div class="info-col info-col-left">
            <div class="section-label">Destinatario</div>
            <div class="client-name">{{ clientDisplayName }}</div>
            <div class="client-details">
              <template v-if="client.fiscal_code"><span class="cf-tag">C.F.</span>{{ client.fiscal_code }}<br></template>
              <template v-if="client.vat_number"><span class="cf-tag">P.IVA</span>{{ client.vat_number }}<br></template>
              <template v-if="client.address">{{ client.address }}<br></template>
              {{ client.zip_code }} {{ client.city }}<template v-if="client.province"> ({{ client.province }})</template>
              <template v-if="client.email"><br>{{ client.email }}</template>
            </div>
          </div>

          <!-- Pagamento -->
          <div class="info-col info-col-right">
            <div class="section-label">Pagamento</div>
            <div class="payment-tag">{{ PAYMENT_LABELS[invoice.payment_method] ?? invoice.payment_method }}</div>
            <div v-if="PAYMENT_CONDITIONS[invoice.payment_method]" class="pay-field">
              <span class="pay-label">Condizioni</span>
              <span class="pay-value">{{ PAYMENT_CONDITIONS[invoice.payment_method] }}</span>
            </div>
            <template v-if="invoice.payment_method === 'bonifico' && config.iban">
              <div class="pay-field">
                <span class="pay-label">IBAN</span>
                <span class="iban-value">{{ config.iban }}</span>
              </div>
              <div class="pay-field">
                <span class="pay-label">Intestato a</span>
                <span class="pay-value">{{ [config.first_name, config.last_name].filter(Boolean).join(' ') }}</span>
              </div>
            </template>
          </div>
        </div>

        <!-- ══════════════════════════════
             ITEMS TABLE
        ══════════════════════════════ -->
        <div class="items-label">Dettaglio prestazioni</div>
        <div class="items-table-wrap">
          <table class="items-table">
            <thead>
              <tr>
                <th class="th-desc">Descrizione</th>
                <th class="th-center">Qtà</th>
                <th class="th-right">Prezzo unit.</th>
                <th class="th-center">IVA</th>
                <th class="th-right th-last">Importo</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="(line, i) in invoice.lines"
                :key="line.id ?? i"
                :class="i % 2 === 1 ? 'tr-alt' : ''"
              >
                <td class="td-desc">{{ line.description }}</td>
                <td class="td-center td-secondary">{{ line.quantity }}</td>
                <td class="td-right td-secondary">{{ formatCurrency(line.unit_price) }}</td>
                <td class="td-center td-secondary">
                  <span v-if="line.vat_rate === 0" class="vat-exempt">Esente</span>
                  <template v-else>{{ line.vat_rate }}%</template>
                </td>
                <td class="td-amount">{{ formatCurrency(line.line_total) }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- ══════════════════════════════
             TOTALS
        ══════════════════════════════ -->
        <div class="totals-outer">
          <div class="totals-table-wrap">
            <table class="totals-table">
              <tr>
                <td class="totals-label">Imponibile</td>
                <td class="totals-value">{{ formatCurrency(invoice.total_net) }}</td>
              </tr>
              <tr v-if="hasIva">
                <td class="totals-label">IVA</td>
                <td class="totals-value">{{ formatCurrency(invoice.total_tax) }}</td>
              </tr>
              <tr v-if="invoice.apply_enpap && invoice.contributo_enpap > 0">
                <td class="totals-label">Contributo ENPAP 2%</td>
                <td class="totals-value">{{ formatCurrency(invoice.contributo_enpap) }}</td>
              </tr>
              <tr v-if="invoice.ritenuta_acconto > 0" class="totals-deduct">
                <td class="totals-label">Ritenuta d'acconto 20%</td>
                <td class="totals-value">− {{ formatCurrency(invoice.ritenuta_acconto) }}</td>
              </tr>
              <tr v-if="invoice.marca_da_bollo">
                <td class="totals-label">Marca da bollo</td>
                <td class="totals-value">{{ formatCurrency(2) }}</td>
              </tr>
              <tr class="totals-separator"><td colspan="2" /></tr>
              <tr class="totals-grand">
                <td class="totals-grand-label">Totale dovuto</td>
                <td class="totals-grand-value">{{ formatCurrency(invoice.total_due) }}</td>
              </tr>
            </table>
          </div>
        </div>

        <!-- ══════════════════════════════
             FOOTER
        ══════════════════════════════ -->
        <div class="footer-section">

          <!-- Fiscal notes -->
          <div v-if="legalNotes.length > 0" class="legal-notes">
            <div class="legal-notes-header">Note fiscali</div>
            <p v-for="(note, i) in legalNotes" :key="i">{{ note }}</p>
          </div>

          <!-- STS tessera sanitaria authorization -->
          <div class="sts-box">
            <div class="sts-section-label">Autorizzazione Sistema Tessera Sanitaria</div>
            <p>
              Ai sensi dell'art.&nbsp;3, commi&nbsp;1 e&nbsp;2, del D.Lgs. 21&nbsp;luglio&nbsp;2014, n.&nbsp;175
              (Dichiarazione dei redditi precompilata), l'intestatario della presente ricevuta sanitaria
              &nbsp;
              <span :class="['sts-option', client.sts_authorization ? 'sts-selected' : '']">
                <span class="sts-checkbox">{{ client.sts_authorization ? '✓' : '' }}</span>AUTORIZZA
              </span>
              <span :class="['sts-option', !client.sts_authorization ? 'sts-selected' : '']">
                <span class="sts-checkbox">{{ !client.sts_authorization ? '✓' : '' }}</span>NON AUTORIZZA
              </span>
              la trasmissione dei dati relativi alla presente spesa sanitaria al Sistema Tessera Sanitaria
              ai fini dell'elaborazione della dichiarazione dei redditi precompilata.
            </p>
          </div>

          <!-- Invoice notes -->
          <div v-if="invoice.notes" class="user-notes-box">
            <div class="user-notes-label">Note</div>
            <p>{{ invoice.notes }}</p>
          </div>

          <!-- Bottom bar -->
          <div class="bottom-bar">
            <span class="bar-name">{{ professionalFullName }}</span>
            <span class="bar-dot">·</span>
            P.IVA&nbsp;{{ config.vat_number }}
            <span class="bar-dot">·</span>
            C.F.&nbsp;{{ config.fiscal_code }}
            <template v-if="config.pec_email">
              <span class="bar-dot">·</span>PEC:&nbsp;{{ config.pec_email }}
            </template>
            <br>
            <template v-if="config.address">{{ config.address }},&nbsp;</template>
            {{ config.zip_code }} {{ config.city }} ({{ config.province }})
          </div>

        </div>
      </div>
    </div>

  </div>
</template>

<style scoped>
/* ── Root: screen background ── */
.print-root {
  min-height: 100vh;
  background: linear-gradient(135deg, #eef2ee 0%, #e4dbd2 100%);
  padding-top: 60px;
  padding-bottom: 80px;
}

/* ── Invoice document card ── */
.invoice-doc {
  max-width: 794px;
  margin: 0 auto;
  background: #ffffff;
  border-radius: 14px;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.13), 0 4px 16px rgba(0, 0, 0, 0.06);
  overflow: hidden;
  font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
  font-size: 9pt;
  color: #1e2d22;
  line-height: 1.55;
}

/* ── Gradient top bar ── */
.header-band {
  height: 6px;
  background: linear-gradient(90deg, #3d6142 0%, #5d8062 45%, #2b7ab3 100%);
}

/* ── Body padding ── */
.doc-body {
  padding: 36px 48px 40px;
}

/* ══════════════════════════════
   HEADER
══════════════════════════════ */
.doc-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 24px;
  margin-bottom: 24px;
}

.header-left { flex: 1; min-width: 0; }
.header-right { flex-shrink: 0; text-align: right; }

.company-name {
  font-family: Georgia, 'Times New Roman', 'Book Antiqua', serif;
  font-size: 19pt;
  font-weight: 700;
  color: #1a2b1d;
  letter-spacing: -0.5px;
  line-height: 1.15;
  margin-bottom: 4px;
}

.company-profession {
  font-size: 7.5pt;
  color: #5d8062;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1.4px;
  margin-bottom: 5px;
}

.albo-info {
  font-size: 7.5pt;
  color: #6b8f70;
  margin-bottom: 2px;
  line-height: 1.45;
}
.albo-info strong { color: #3d6142; font-weight: 600; }

.company-details {
  margin-top: 8px;
  font-size: 7.5pt;
  color: #7a9580;
  line-height: 1.7;
}

/* Invoice identity */
.invoice-label {
  font-size: 7pt;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 2px;
  color: #5d8062;
  margin-bottom: 4px;
}

.invoice-number {
  font-family: Georgia, 'Times New Roman', serif;
  font-size: 30pt;
  font-weight: 700;
  color: #1a2b1d;
  letter-spacing: -1px;
  line-height: 1;
  margin-bottom: 10px;
}

.invoice-prefix {
  font-size: 15pt;
  font-weight: 400;
  color: #9cb8a0;
  margin-right: 2px;
}

.invoice-meta {
  border-top: 1px solid #e0ebe1;
  padding-top: 8px;
}

.meta-row {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 8px;
  margin-bottom: 3px;
}

.meta-label {
  font-size: 7pt;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: #9cb8a0;
}

.meta-value {
  font-size: 8pt;
  font-weight: 600;
  color: #1a2b1d;
}

/* Status badges */
.status-badge {
  display: inline-block;
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 7pt;
  font-weight: 700;
  letter-spacing: 0.8px;
  text-transform: uppercase;
  margin-top: 8px;
}
.status-paid     { background: #dcfce7; color: #166534; }
.status-issued   { background: #dbeafe; color: #1e40af; }
.status-draft    { background: #f3f4f6; color: #4b5563; }
.status-overdue  { background: #fee2e2; color: #991b1b; }
.status-cancelled { background: #fef9c3; color: #854d0e; }

/* ── Divider ── */
.doc-divider {
  border: none;
  border-top: 1px solid #e0ebe1;
  margin: 0 0 20px;
}

/* ══════════════════════════════
   INFO CARD
══════════════════════════════ */
.info-card {
  display: flex;
  margin-bottom: 20px;
  border: 1px solid #dde8dd;
  border-radius: 8px;
  overflow: hidden;
}

.info-col { padding: 14px 18px; }

.info-col-left {
  flex: 0 0 48%;
  background: #f7faf7;
  border-right: 1px solid #dde8dd;
}

.info-col-right { flex: 1; background: #ffffff; }

.section-label {
  font-size: 6.5pt;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  color: #5d8062;
  font-weight: 700;
  margin-bottom: 8px;
}

.client-name {
  font-family: Georgia, 'Times New Roman', serif;
  font-size: 13pt;
  font-weight: 700;
  color: #1a2b1d;
  margin-bottom: 5px;
  letter-spacing: -0.2px;
  line-height: 1.2;
}

.client-details {
  font-size: 7.5pt;
  color: #6b8f70;
  line-height: 1.65;
}

.cf-tag {
  font-size: 6.5pt;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: #5d8062;
  text-transform: uppercase;
  margin-right: 3px;
}

.payment-tag {
  display: inline-block;
  background: #f0f7f1;
  border: 1px solid #ccdece;
  border-radius: 4px;
  padding: 3px 10px;
  font-size: 8pt;
  font-weight: 600;
  color: #3d6142;
  margin-bottom: 9px;
}

.pay-field { margin-bottom: 6px; }

.pay-label {
  font-size: 6.5pt;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  color: #9cb8a0;
  font-weight: 600;
  display: block;
  margin-bottom: 1px;
}

.pay-value {
  font-size: 8pt;
  font-weight: 600;
  color: #1a2b1d;
}

.iban-value {
  font-family: 'Courier New', 'Courier', monospace;
  font-size: 7.5pt;
  letter-spacing: 0.8px;
  color: #1a2b1d;
  font-weight: 600;
  word-break: break-all;
}

/* ══════════════════════════════
   ITEMS TABLE
══════════════════════════════ */
.items-label {
  font-size: 6.5pt;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  color: #5d8062;
  font-weight: 700;
  margin-bottom: 7px;
}

.items-table-wrap {
  border: 1px solid #dde8dd;
  border-radius: 7px;
  overflow: hidden;
}

.items-table {
  width: 100%;
  border-collapse: collapse;
}

.items-table thead tr {
  background: #f0f5f0;
  border-bottom: 1.5px solid #d0dfd0;
}

.items-table thead th {
  padding: 9px 12px;
  font-size: 6.5pt;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: #3d6142;
  border: none;
}

.th-desc  { text-align: left; padding-left: 15px !important; width: 44%; }
.th-center { text-align: center; width: 9%; }
.th-right { text-align: right; width: 18%; }
.th-last  { padding-right: 15px !important; }

.items-table tbody td {
  padding: 9px 12px;
  font-size: 8.5pt;
  border: none;
  border-bottom: 1px solid #f0f5f0;
  vertical-align: top;
}
.items-table tbody td:first-child { padding-left: 15px; }
.items-table tbody td:last-child  { padding-right: 15px; }
.items-table tbody tr:last-child td { border-bottom: none; }
.tr-alt { background: #fafcfa; }

.td-desc      { font-weight: 500; color: #1a2b1d; }
.td-center    { text-align: center; }
.td-right     { text-align: right; }
.td-secondary { color: #5e7862; font-size: 8pt; }
.td-amount    { font-weight: 700; color: #1a2b1d; text-align: right; }

.vat-exempt {
  display: inline-block;
  background: #f0f5f0;
  border-radius: 3px;
  padding: 1px 5px;
  font-size: 7pt;
  font-weight: 600;
  color: #5d8062;
}

/* ══════════════════════════════
   TOTALS
══════════════════════════════ */
.totals-outer {
  display: flex;
  justify-content: flex-end;
  margin-top: 14px;
}

.totals-table-wrap {
  width: 295px;
  border: 1px solid #dde8dd;
  border-radius: 7px;
  overflow: hidden;
}

.totals-table {
  width: 100%;
  border-collapse: collapse;
}

.totals-table td {
  padding: 7px 16px;
  font-size: 8.5pt;
  border: none;
  border-bottom: 1px solid #f0f5f0;
}

.totals-label { color: #7a9580; font-weight: 400; }
.totals-value { text-align: right; font-weight: 600; color: #1a2b1d; }

.totals-deduct .totals-label { color: #c53030; }
.totals-deduct .totals-value { color: #c53030; }

.totals-separator td { padding: 0; border-bottom: 2px solid #dde8dd; }

.totals-grand { background: #1a2b1d !important; }
.totals-grand td { border-bottom: none !important; }
.totals-grand-label {
  padding: 12px 16px !important;
  color: #c8deca;
  font-size: 8pt;
  font-weight: 600;
}
.totals-grand-value {
  padding: 12px 16px !important;
  text-align: right;
  color: #ffffff;
  font-family: Georgia, 'Times New Roman', serif;
  font-size: 15pt;
  font-weight: 700;
  letter-spacing: -0.3px;
}

/* ══════════════════════════════
   FOOTER
══════════════════════════════ */
.footer-section { margin-top: 26px; }

/* Legal notes */
.legal-notes {
  background: #fffbf0;
  border: 1px solid #f0d890;
  border-radius: 6px;
  padding: 11px 15px;
  margin-bottom: 10px;
}

.legal-notes-header {
  font-size: 6.5pt;
  text-transform: uppercase;
  letter-spacing: 1.2px;
  color: #9a7210;
  font-weight: 700;
  margin-bottom: 6px;
}

.legal-notes p {
  font-size: 7pt;
  color: #6b4f0a;
  margin: 3px 0;
  line-height: 1.55;
}

/* STS authorization */
.sts-box {
  background: #fafafa;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  padding: 11px 15px;
  margin-bottom: 10px;
}

.sts-section-label {
  font-size: 6.5pt;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  color: #888;
  font-weight: 700;
  margin-bottom: 7px;
}

.sts-box p {
  font-size: 7.5pt;
  color: #333;
  margin: 0;
  line-height: 1.65;
}

.sts-option {
  display: inline-block;
  margin: 0 10px 0 0;
  vertical-align: middle;
}

.sts-selected {
  font-weight: 700;
  color: #1a2b1d;
}

.sts-checkbox {
  display: inline-block;
  width: 11px;
  height: 11px;
  border: 1.5px solid #7a7a7a;
  margin-right: 4px;
  vertical-align: middle;
  border-radius: 2px;
  text-align: center;
  line-height: 10px;
  font-size: 8pt;
  font-weight: 700;
  color: #3d6142;
}

/* Invoice notes */
.user-notes-box {
  background: #f7faf7;
  border: 1px solid #dde8dd;
  border-radius: 6px;
  padding: 10px 15px;
  margin-bottom: 10px;
}

.user-notes-label {
  font-size: 6.5pt;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  color: #5d8062;
  font-weight: 700;
  margin-bottom: 5px;
}

.user-notes-box p {
  font-size: 8pt;
  color: #1a2b1d;
  margin: 0;
  line-height: 1.65;
}

/* Bottom bar */
.bottom-bar {
  margin-top: 20px;
  padding-top: 12px;
  border-top: 1px solid #e0ebe1;
  text-align: center;
  font-size: 7pt;
  color: #9cb8a0;
  line-height: 1.7;
}

.bar-name { color: #3d6142; font-weight: 600; }
.bar-dot  { color: #c8deca; margin: 0 4px; }

/* ══════════════════════════════
   PRINT MEDIA QUERY
══════════════════════════════ */
@media print {
  .print-root {
    background: none !important;
    padding: 0 !important;
    min-height: auto !important;
  }

  .invoice-doc {
    max-width: 100% !important;
    margin: 0 !important;
    border-radius: 0 !important;
    box-shadow: none !important;
  }

  @page {
    size: A4;
    margin: 0;
  }
}
</style>

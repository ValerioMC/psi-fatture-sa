<script setup lang="ts">
/** The printable invoice: loads the invoice, its patient and the profile, then lays the page out section by section. */
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowLeft, Printer } from 'lucide-vue-next'
import AppButton from '@/components/ui/AppButton.vue'
import AppSpinner from '@/components/ui/AppSpinner.vue'
import PrintLetterhead from '@/features/invoice/components/print/PrintLetterhead.vue'
import PrintParties from '@/features/invoice/components/print/PrintParties.vue'
import PrintLines from '@/features/invoice/components/print/PrintLines.vue'
import PrintTotals from '@/features/invoice/components/print/PrintTotals.vue'
import PrintFooter from '@/features/invoice/components/print/PrintFooter.vue'
import { getClient } from '@/features/client/api/clientApi'
import { getInvoice } from '@/features/invoice/api/invoiceApi'
import { printCurrentPage } from '@/features/invoice/api/pdfApi'
import { getConfig } from '@/features/settings/api/configApi'
import type { Client } from '@/features/client/model/client'
import type { Invoice } from '@/features/invoice/model/invoice'
import type { ProfessionalConfig } from '@/features/settings/model/config'
import { useSmoothScroll } from '@/common/composables/useSmoothScroll'

const route = useRoute()
useSmoothScroll()
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

</script>

<template>
  <div class="print-root">

    <!-- ── Toolbar (screen only) ── -->
    <div class="glass print:hidden fixed inset-x-0 top-0 z-(--z-sticky) border-b border-border/70" data-tauri-drag-region>
      <div class="toolbar-inner mx-auto flex max-w-[794px] items-center gap-2 py-3" data-tauri-drag-region>
        <AppButton variant="ghost" :icon="ArrowLeft" :to="`/invoices/${invoiceId}`">Fattura</AppButton>
        <p class="flex-1 text-center text-sm text-text-subtle" data-tauri-drag-region>
          Per il paziente il PDF lo prepara l’app: dalla fattura, <span class="font-medium text-text-muted">Salva PDF</span> o <span class="font-medium text-text-muted">Invia al paziente</span>.
        </p>
        <AppButton v-if="!loading && invoice" variant="primary" :icon="Printer" @click="printCurrentPage()">Stampa</AppButton>
      </div>
    </div>

    <!-- ── Loading ── -->
    <div v-if="loading" class="print:hidden flex min-h-[70vh] items-center justify-center text-text-subtle" role="status">
      <AppSpinner :size="20" />
      <span class="sr-only">Caricamento della fattura</span>
    </div>

    <!-- ── Error ── -->
    <div
      v-else-if="error"
      class="print:hidden mx-auto mt-24 max-w-md rounded-card border border-danger-line bg-danger-soft p-6 text-center"
      role="alert"
    >
      <p class="text-base font-medium text-danger">Non è stato possibile preparare la fattura</p>
      <p class="mt-1 text-sm text-text-muted">{{ error }}</p>
    </div>

    <!-- ── Invoice document ── -->
    <div
      v-else-if="invoice && client && config"
      class="invoice-doc"
    >
      <div class="doc-body">
        <PrintLetterhead :invoice="invoice" :config="config" />
        <PrintParties :invoice="invoice" :client="client" :config="config" />
        <PrintLines :invoice="invoice" />
        <PrintTotals :invoice="invoice" />
        <PrintFooter :invoice="invoice" :client="client" :config="config" />
      </div>
    </div>

  </div>
</template>

<style scoped>
/* ── Root: screen background ── */
.print-root {
  min-height: 100vh;
  padding-top: 92px;
  padding-bottom: 80px;
}

/* ── Invoice document card ── */
.invoice-doc {
  max-width: 794px;
  margin: 0 auto;
  background: #ffffff;
  border-radius: 14px;
  box-shadow: var(--shadow-modal);
  overflow: hidden;
  font-family: var(--font-sans);
  font-size: 9pt;
  color: #1d1b24;
  line-height: 1.55;
}

/* ── Body padding ── */
.doc-body {
  padding: 36px 48px 40px;
}

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
    -webkit-print-color-adjust: exact !important;
    print-color-adjust: exact !important;
  }

  .invoice-doc * {
    -webkit-print-color-adjust: exact !important;
    print-color-adjust: exact !important;
  }

  @page {
    size: A4;
    margin: 0;
  }
}
</style>

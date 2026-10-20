<script setup lang="ts">
/** The totals box at the right: one row per amount that applies, then the total due. */
import { computed } from 'vue'
import type { Invoice } from '@/features/invoice/model/invoice'
import { formatCurrency } from '@/common/lib/format'
import { hasVat } from '@/features/invoice/lib/invoicePrint'

const props = defineProps<{ invoice: Invoice }>()

const hasIva = computed(() => hasVat(props.invoice))
</script>

<template>
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
        <tr v-if="invoice.marca_da_bollo">
          <td class="totals-label">Marca da bollo</td>
          <td class="totals-value">{{ formatCurrency(2) }}</td>
        </tr>
        <tr v-if="invoice.apply_enpap && invoice.contributo_enpap > 0">
          <td class="totals-label">Contributo ENPAP 2%</td>
          <td class="totals-value">{{ formatCurrency(invoice.contributo_enpap) }}</td>
        </tr>
        <tr v-if="invoice.ritenuta_acconto > 0" class="totals-deduct">
          <td class="totals-label">Ritenuta d'acconto 20%</td>
          <td class="totals-value">− {{ formatCurrency(invoice.ritenuta_acconto) }}</td>
        </tr>
        <tr class="totals-separator"><td colspan="2" /></tr>
        <tr class="totals-grand">
          <td class="totals-grand-label">Totale dovuto</td>
          <td class="totals-grand-value">{{ formatCurrency(invoice.total_due) }}</td>
        </tr>
      </table>
    </div>
  </div>
</template>

<style scoped>
.totals-outer {
  display: flex;
  justify-content: flex-end;
  margin-top: 14px;
}

.totals-table-wrap {
  width: 295px;
  border: 1px solid #e6e2d9;
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
  border-bottom: 1px solid #f2efe9;
}

.totals-label { color: #5c5866; font-weight: 400; }
.totals-value { text-align: right; font-weight: 600; color: #1d1b24; }

.totals-deduct .totals-label { color: #a83a2c; }
.totals-deduct .totals-value { color: #a83a2c; }

.totals-separator td { padding: 0; border-bottom: 2px solid #e6e2d9; }

.totals-grand { background: #ecedf9 !important; border: 1px solid #c9cbee; }
.totals-grand td { border-bottom: none !important; }
.totals-grand-label {
  padding: 12px 16px !important;
  color: #362ea0;
  font-size: 8pt;
  font-weight: 600;
}
.totals-grand-value {
  padding: 12px 16px !important;
  text-align: right;
  color: #2c2f80;
  font-family: var(--font-display);
  font-size: 15pt;
  font-weight: 600;
  letter-spacing: -0.3px;
}

@media print {
  .totals-outer {
    page-break-inside: avoid;
  }
}
</style>

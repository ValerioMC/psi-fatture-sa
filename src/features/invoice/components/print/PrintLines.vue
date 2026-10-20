<script setup lang="ts">
/** The table of the invoice's lines; quantity and unit price are left out when the invoice hides them. */
import type { Invoice } from '@/features/invoice/model/invoice'
import { formatCurrency } from '@/common/lib/format'
import { hasAmountOverride } from '@/features/invoice/lib/tax'

defineProps<{ invoice: Invoice }>()

</script>

<template>
  <div class="items-label">Dettaglio prestazioni</div>
  <div class="items-table-wrap">
    <table class="items-table">
      <thead>
        <tr>
          <th class="th-desc">Descrizione</th>
          <th v-if="!invoice.hide_quantity" class="th-center">Qtà</th>
          <th v-if="!invoice.hide_quantity" class="th-right">Prezzo unit.</th>
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
          <td v-if="!invoice.hide_quantity" class="td-center td-secondary">{{ line.quantity }}</td>
          <td v-if="!invoice.hide_quantity" class="td-right td-secondary">{{ hasAmountOverride(line) ? '' : formatCurrency(line.unit_price) }}</td>
          <td class="td-center td-secondary">
            <span v-if="line.vat_rate === 0" class="vat-exempt">Esente</span>
            <template v-else>{{ line.vat_rate }}%</template>
          </td>
          <td class="td-amount">{{ formatCurrency(line.line_total) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.items-label {
  font-size: 6.5pt;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  color: #362ea0;
  font-weight: 700;
  margin-bottom: 7px;
}

.items-table-wrap {
  border: 1px solid #e6e2d9;
  border-radius: 7px;
  overflow: hidden;
}

.items-table {
  width: 100%;
  border-collapse: collapse;
}

.items-table thead tr {
  background: #f2efe9;
  border-bottom: 1.5px solid #e6e2d9;
}

.items-table thead th {
  padding: 9px 12px;
  font-size: 6.5pt;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: #2c2f80;
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
  border-bottom: 1px solid #f2efe9;
  vertical-align: top;
}
.items-table tbody td:first-child { padding-left: 15px; }
.items-table tbody td:last-child  { padding-right: 15px; }
.items-table tbody tr:last-child td { border-bottom: none; }
.tr-alt { background: #faf8f4; }

.td-desc      { font-weight: 500; color: #1d1b24; }
.td-center    { text-align: center; }
.td-right     { text-align: right; }
.td-secondary { color: #5c5866; font-size: 8pt; }
.td-amount    { font-weight: 700; color: #1d1b24; text-align: right; }

.vat-exempt {
  display: inline-block;
  background: #f2efe9;
  border-radius: 3px;
  padding: 1px 5px;
  font-size: 7pt;
  font-weight: 600;
  color: #362ea0;
}

@media print {
  .items-table-wrap {
    page-break-inside: avoid;
  }
}
</style>

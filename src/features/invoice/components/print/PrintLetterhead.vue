<script setup lang="ts">
/** The printed invoice's header: the professional's letterhead on the left, the invoice number and dates on the right. */
import { computed } from 'vue'
import type { Invoice } from '@/features/invoice/model/invoice'
import type { ProfessionalConfig } from '@/features/settings/model/config'
import { formatDateLong } from '@/common/lib/format'
import { PROFESSION_LABEL } from '@/common/lib/labels'
import { professionalFullName } from '@/features/settings/lib/professional'

const props = defineProps<{ invoice: Invoice; config: ProfessionalConfig }>()

const professionalName = computed(() => professionalFullName(props.config))
const professionLabel = computed(() => PROFESSION_LABEL[props.config.profession])
</script>

<template>
  <div class="doc-header">
    <!-- Left: professional info -->
    <div class="header-left">
      <div class="company-name">{{ professionalName }}</div>
      <div class="company-profession">
        {{ professionLabel }}<template v-if="config.is_psicoanalista"> &nbsp;&mdash;&nbsp; Psicoanalista</template><template v-else-if="config.specialization"> &nbsp;&mdash;&nbsp; {{ config.specialization }}</template>
      </div>
      <div v-if="config.albo_number || config.albo_region" class="albo-info">
        <template v-if="config.albo_number">Iscriz. Albo n.&nbsp;<strong>{{ config.albo_number }}</strong></template>
        <template v-if="config.albo_number && config.albo_region">&nbsp;&mdash;&nbsp;</template>
        <template v-if="config.albo_region">Regione&nbsp;<strong>{{ config.albo_region }}</strong></template>
      </div>
      <div v-if="config.is_psicoanalista" class="albo-info">
        Membro della <strong>International Psychoanalytical Association (IPA)</strong>
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
          <span class="meta-label">Data emissione</span>
          <span class="meta-value">{{ formatDateLong(invoice.issue_date) }}</span>
        </div>
        <div v-if="invoice.due_date" class="meta-row">
          <span class="meta-label">Scadenza</span>
          <span class="meta-value">{{ formatDateLong(invoice.due_date) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
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
  font-family: var(--font-display);
  font-size: 19pt;
  font-weight: 600;
  color: #1d1b24;
  letter-spacing: -0.5px;
  line-height: 1.15;
  margin-bottom: 4px;
}

.company-profession {
  font-size: 7.5pt;
  color: #362ea0;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1.4px;
  margin-bottom: 5px;
}

.albo-info {
  font-size: 7.5pt;
  color: #5c5866;
  margin-bottom: 2px;
  line-height: 1.45;
}
.albo-info strong { color: #2c2f80; font-weight: 600; }

.company-details {
  margin-top: 8px;
  font-size: 7.5pt;
  color: #5c5866;
  line-height: 1.7;
}

/* Invoice identity */
.invoice-label {
  font-size: 7pt;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 2px;
  color: #362ea0;
  margin-bottom: 4px;
}

.invoice-number {
  font-family: var(--font-display);
  font-size: 30pt;
  font-weight: 600;
  color: #1d1b24;
  letter-spacing: -1px;
  line-height: 1;
  margin-bottom: 10px;
}

.invoice-prefix {
  font-size: 15pt;
  font-weight: 400;
  color: #8a8694;
  margin-right: 2px;
}

.invoice-meta {
  border-top: 1px solid #e6e2d9;
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
  color: #8a8694;
}

.meta-value {
  font-size: 8pt;
  font-weight: 600;
  color: #1d1b24;
}
</style>

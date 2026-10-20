<script setup lang="ts">
/** The card under the header: who the invoice is for on the left, how to pay on the right. */
import { computed } from 'vue'
import type { Client } from '@/features/client/model/client'
import type { Invoice } from '@/features/invoice/model/invoice'
import type { ProfessionalConfig } from '@/features/settings/model/config'
import { paymentCondition, printedClientName, printedPaymentLabel } from '@/features/invoice/lib/invoicePrint'

const props = defineProps<{ invoice: Invoice; client: Client; config: ProfessionalConfig }>()

const clientName = computed(() => printedClientName(props.client))
</script>

<template>
  <div class="info-card">
    <!-- Destinatario -->
    <div class="info-col info-col-left">
      <div class="section-label">Destinatario</div>
      <div class="client-name">{{ clientName }}</div>
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
      <div class="payment-tag">{{ printedPaymentLabel(invoice.payment_method) }}</div>
      <div v-if="paymentCondition(invoice.payment_method)" class="pay-field">
        <span class="pay-label">Condizioni</span>
        <span class="pay-value">{{ paymentCondition(invoice.payment_method) }}</span>
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
</template>

<style scoped>
.info-card {
  display: flex;
  margin-bottom: 20px;
  border: 1px solid #e6e2d9;
  border-radius: 8px;
  overflow: hidden;
}

.info-col { padding: 14px 18px; }

.info-col-left {
  flex: 0 0 48%;
  background: #faf8f4;
  border-right: 1px solid #e6e2d9;
}

.info-col-right { flex: 1; background: #ffffff; }

.section-label {
  font-size: 6.5pt;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  color: #362ea0;
  font-weight: 700;
  margin-bottom: 8px;
}

.client-name {
  font-family: var(--font-display);
  font-size: 13pt;
  font-weight: 600;
  color: #1d1b24;
  margin-bottom: 5px;
  letter-spacing: -0.2px;
  line-height: 1.2;
}

.client-details {
  font-size: 7.5pt;
  color: #5c5866;
  line-height: 1.65;
}

.cf-tag {
  font-size: 6.5pt;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: #362ea0;
  text-transform: uppercase;
  margin-right: 3px;
}

.payment-tag {
  display: inline-block;
  background: #ecedf9;
  border: 1px solid #c9cbee;
  border-radius: 4px;
  padding: 3px 10px;
  font-size: 8pt;
  font-weight: 600;
  color: #2c2f80;
  margin-bottom: 9px;
}

.pay-field { margin-bottom: 6px; }

.pay-label {
  font-size: 6.5pt;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  color: #8a8694;
  font-weight: 600;
  display: block;
  margin-bottom: 1px;
}

.pay-value {
  font-size: 8pt;
  font-weight: 600;
  color: #1d1b24;
}

.iban-value {
  font-family: var(--font-mono);
  font-size: 7.5pt;
  letter-spacing: 0.8px;
  color: #1d1b24;
  font-weight: 600;
  word-break: break-all;
}

@media print {
  .info-card {
    page-break-inside: avoid;
  }
}
</style>

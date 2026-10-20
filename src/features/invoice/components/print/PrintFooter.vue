<script setup lang="ts">
/** The end of the printed invoice: fiscal notes, the Sistema TS authorisation, free notes and the professional's footer. */
import { computed } from 'vue'
import type { Client } from '@/features/client/model/client'
import type { Invoice } from '@/features/invoice/model/invoice'
import type { ProfessionalConfig } from '@/features/settings/model/config'
import { legalNotes } from '@/features/invoice/lib/invoicePrint'
import { professionalFullName } from '@/features/settings/lib/professional'

const props = defineProps<{ invoice: Invoice; client: Client; config: ProfessionalConfig }>()

const professionalName = computed(() => professionalFullName(props.config))
const notes = computed(() => legalNotes(props.invoice, props.config))
</script>

<template>
  <div class="footer-section">

    <!-- Fiscal notes -->
    <div v-if="notes.length > 0" class="legal-notes">
      <div class="legal-notes-header">Note fiscali</div>
      <p v-for="(note, i) in notes" :key="i">{{ note }}</p>
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
      <span class="bar-name">{{ professionalName }}</span>
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
</template>

<style scoped>
.footer-section { margin-top: 26px; }

/* Legal notes */
.legal-notes {
  background: #fffbeb;
  border: 1px solid #fde68a;
  border-radius: 6px;
  padding: 11px 15px;
  margin-bottom: 10px;
}

.legal-notes-header {
  font-size: 6.5pt;
  text-transform: uppercase;
  letter-spacing: 1.2px;
  color: #b45309;
  font-weight: 700;
  margin-bottom: 6px;
}

.legal-notes p {
  font-size: 7pt;
  color: #78350f;
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
  color: #8a8694;
  font-weight: 700;
  margin-bottom: 7px;
}

.sts-box p {
  font-size: 7.5pt;
  color: #3d3a45;
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
  color: #1d1b24;
}

.sts-checkbox {
  display: inline-block;
  width: 11px;
  height: 11px;
  border: 1.5px solid #8a8694;
  margin-right: 4px;
  vertical-align: middle;
  border-radius: 2px;
  text-align: center;
  line-height: 10px;
  font-size: 8pt;
  font-weight: 700;
  color: #2c2f80;
}

/* Invoice notes */
.user-notes-box {
  background: #faf8f4;
  border: 1px solid #e6e2d9;
  border-radius: 6px;
  padding: 10px 15px;
  margin-bottom: 10px;
}

.user-notes-label {
  font-size: 6.5pt;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  color: #362ea0;
  font-weight: 700;
  margin-bottom: 5px;
}

.user-notes-box p {
  font-size: 8pt;
  color: #1d1b24;
  margin: 0;
  line-height: 1.65;
}

/* Bottom bar */
.bottom-bar {
  margin-top: 20px;
  padding-top: 12px;
  border-top: 1px solid #e6e2d9;
  text-align: center;
  font-size: 7pt;
  color: #8a8694;
  line-height: 1.7;
}

.bar-name { color: #2c2f80; font-weight: 600; }
.bar-dot  { color: #d4cfc4; margin: 0 4px; }

@media print {
  .footer-section {
    page-break-inside: avoid;
  }
}
</style>

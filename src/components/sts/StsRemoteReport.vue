<script setup lang="ts">
/**
 * What the Sistema TS holds for a month, read live from its monthly report:
 * by the day each document was sent or by the day it was paid. Every row is
 * traced back to its invoice; a document with no invoice here stands out.
 */
import { computed, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'
import { CloudOff, RefreshCw, Unlink } from 'lucide-vue-next'
import { getTsMonthlyReport } from '@/api'
import { errorMessage } from '@/stores/toast'
import type { TsReportBasis, TsReportRow } from '@/types'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import PeriodStepper from '@/components/ui/PeriodStepper.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import SkeletonRows from '@/components/ui/SkeletonRows.vue'
import { formatCurrency, formatDate, formatMonthYear } from '@/utils/format'
import { plural } from '@/utils/labels'
import { describeSendKind } from '@/utils/sts'

const BASIS_OPTIONS = [
  { value: 'invio', label: 'Per data di invio' },
  { value: 'pagamento', label: 'Per data di pagamento' },
] as const

const today = new Date()
const year = ref(today.getFullYear())
const month = ref(today.getMonth() + 1)
const basis = ref<TsReportBasis>('invio')
const rows = ref<TsReportRow[] | null>(null)
const loading = ref(false)
const failure = ref<string | null>(null)

const isCurrentMonth = computed(() => year.value === today.getFullYear() && month.value === today.getMonth() + 1)
const total = computed(() => (rows.value ?? []).reduce((sum, row) => sum + row.amount - row.refunded_amount, 0))
const orphans = computed(() => (rows.value ?? []).filter((row) => row.invoice_id === null).length)

async function load(): Promise<void> {
  loading.value = true
  failure.value = null
  try {
    rows.value = await getTsMonthlyReport(year.value, month.value, basis.value)
  } catch (error) {
    rows.value = null
    failure.value = errorMessage(error)
  } finally {
    loading.value = false
  }
}

function step(delta: number): void {
  const moved = new Date(year.value, month.value - 1 + delta, 1)
  year.value = moved.getFullYear()
  month.value = moved.getMonth() + 1
}

watch([year, month, basis], () => { void load() }, { immediate: true })
</script>

<template>
  <div class="flex flex-wrap items-center gap-3 border-b border-border px-5 py-3">
    <PeriodStepper
      :label="formatMonthYear(year, month)"
      previous-label="Mese precedente"
      next-label="Mese successivo"
      :can-next="!isCurrentMonth"
      min-width="9rem"
      @previous="step(-1)"
      @next="step(1)"
    />
    <SegmentedControl v-model="basis" :options="BASIS_OPTIONS" label="Criterio del report" size="sm" />
    <p v-if="rows && rows.length > 0" class="ml-auto text-sm text-text-muted">
      {{ plural(rows.length, 'documento', 'documenti') }} · <span class="tabular font-medium text-text">{{ formatCurrency(total) }}</span>
    </p>
    <AppButton :class="rows && rows.length > 0 ? '' : 'ml-auto'" variant="ghost" size="sm" :icon="RefreshCw" :loading="loading" label="Aggiorna il report" @click="load" />
  </div>

  <SkeletonRows v-if="loading && rows === null" variant="table" :count="5" label="Lettura del report dal Sistema TS" />

  <EmptyState v-else-if="failure" :icon="CloudOff" :bordered="false" title="Report non disponibile" :description="failure">
    <AppButton to="/settings">Controlla le credenziali</AppButton>
  </EmptyState>

  <EmptyState
    v-else-if="rows && rows.length === 0"
    :bordered="false"
    title="Nessun documento sul Sistema TS"
    :description="`Per ${formatMonthYear(year, month).toLowerCase()} il Sistema TS non ha documenti ${basis === 'invio' ? 'inviati' : 'pagati'} a tuo nome.`"
  />

  <template v-else-if="rows">
    <p v-if="orphans > 0" class="flex items-center gap-2 border-b border-border bg-warn-soft px-5 py-2 text-xs text-text-muted">
      <Unlink :size="13" class="text-warn" aria-hidden="true" />
      {{ plural(orphans, 'documento non corrisponde', 'documenti non corrispondono') }} a una fattura dell’archivio: forse inviati da un altro programma.
    </p>
    <table class="data-table text-base">
      <thead>
        <tr>
          <th class="w-32 pl-5 font-medium">Documento</th>
          <th class="w-28 font-medium">Emesso</th>
          <th class="w-28 font-medium">Pagato</th>
          <th class="w-28 font-medium">Ultima operazione</th>
          <th class="hidden font-medium xl:table-cell">Protocollo</th>
          <th class="w-28 font-medium">Inviato</th>
          <th class="w-32 pr-6 text-right font-medium">Importo</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in rows" :key="`${row.issue_date}-${row.document_number}-${row.protocol}`" class="h-row">
          <td class="pl-5">
            <RouterLink v-if="row.invoice_id" :to="`/invoices/${row.invoice_id}`" class="tabular rounded-sm text-text hover:underline focus-ring">
              N. {{ row.document_number }}
            </RouterLink>
            <span v-else class="inline-flex items-center gap-1.5">
              <span class="tabular text-text-muted">{{ row.document_number }}</span>
              <AppBadge tone="warn">Non in archivio</AppBadge>
            </span>
          </td>
          <td class="tabular text-sm text-text-muted">{{ formatDate(row.issue_date) }}</td>
          <td class="tabular text-sm text-text-muted">{{ formatDate(row.payment_date) }}</td>
          <td class="text-sm text-text-muted">{{ describeSendKind(row.send_kind) }}</td>
          <td class="hidden font-mono text-xs text-text-subtle xl:table-cell">{{ row.protocol }}</td>
          <td class="tabular text-sm text-text-subtle">{{ formatDate(row.sent_date) }}</td>
          <td class="tabular pr-6 text-right font-medium text-text">
            {{ formatCurrency(row.amount) }}
            <span v-if="row.refunded_amount > 0" class="block text-xs font-normal text-text-subtle">rimborsati {{ formatCurrency(row.refunded_amount) }}</span>
          </td>
        </tr>
      </tbody>
    </table>
  </template>
</template>

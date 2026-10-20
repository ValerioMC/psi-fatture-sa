<script setup lang="ts">
/** The patient's latest invoices beside their record. A courtesy: when they cannot load, the card says none. */
import { computed, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { ArrowRight, FileText } from 'lucide-vue-next'
import { listInvoices } from '@/features/invoice/api/invoiceApi'
import type { Invoice } from '@/features/invoice/model/invoice'
import { historyTotal, newestFirst } from '@/features/invoice/lib/patientInvoices'
import AppCard from '@/components/ui/AppCard.vue'
import CardHeader from '@/components/ui/CardHeader.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import { formatCurrency, formatDateShort } from '@/common/lib/format'
import { plural } from '@/common/lib/labels'

const props = defineProps<{ clientId: number; clientName: string }>()

const history = ref<Invoice[]>([])
const total = computed(() => historyTotal(history.value))

onMounted(() => {
  listInvoices({ client_id: props.clientId })
    .then((invoices) => { history.value = newestFirst(invoices) })
    .catch(() => { history.value = [] })
})
</script>

<template>
  <AppCard :padded="false">
    <CardHeader title="Fatture" :icon="FileText" divided>
      <template #subtitle>
        <template v-if="history.length > 0">{{ plural(history.length, 'fattura', 'fatture') }} · {{ formatCurrency(total) }}</template>
        <template v-else>Nessuna fattura ancora</template>
      </template>
    </CardHeader>
    <ul v-if="history.length > 0" class="p-1.5">
      <li v-for="invoice in history.slice(0, 5)" :key="invoice.id">
        <RouterLink
          :to="`/invoices/${invoice.id}`"
          class="group flex items-center gap-3 rounded-control px-3 py-2 transition-colors hover:bg-surface-hover focus-ring"
        >
          <span class="min-w-0 flex-1">
            <span class="tabular block text-sm font-medium text-text">N. {{ invoice.invoice_number }}/{{ invoice.year }}</span>
            <span class="block text-xs text-text-subtle">{{ formatDateShort(invoice.issue_date) }}</span>
          </span>
          <span class="tabular text-sm font-medium text-text">{{ formatCurrency(invoice.total_due) }}</span>
          <StatusBadge type="invoice" :status="invoice.status" />
        </RouterLink>
      </li>
    </ul>
    <div v-if="history.length > 5" class="border-t border-border px-5 py-2.5">
      <RouterLink :to="{ path: '/invoices', query: { year: 0, q: clientName } }" class="group inline-flex items-center gap-1 text-sm font-medium text-accent focus-ring">
        Tutte le fatture <ArrowRight :size="14" class="transition-transform group-hover:translate-x-0.5" aria-hidden="true" />
      </RouterLink>
    </div>
  </AppCard>
</template>

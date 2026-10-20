<script setup lang="ts">
/** The register's "In carico" figure: how many of the listed patients were billed in the last `CARE_WINDOW_DAYS`. */
import { computed, onMounted, ref } from 'vue'
import { Activity } from 'lucide-vue-next'
import { listInvoices } from '@/features/invoice/api/invoiceApi'
import type { Invoice } from '@/features/invoice/model/invoice'
import { patientsInCare } from '@/features/invoice/lib/patientInvoices'
import { useToastStore } from '@/common/store/toast'
import StatTile from '@/components/ui/StatTile.vue'

const CARE_WINDOW_DAYS = 90

const props = defineProps<{ patientIds: readonly number[] }>()

const toast = useToastStore()
const invoices = ref<Invoice[]>([])

const total = computed(() => props.patientIds.length)
const inCare = computed(() => {
  const billed = patientsInCare(invoices.value, CARE_WINDOW_DAYS)
  return props.patientIds.filter((id) => billed.has(id)).length
})
const share = computed(() => (total.value > 0 ? Math.round((inCare.value / total.value) * 100) : 0))

onMounted(async () => {
  try {
    invoices.value = await listInvoices({})
  } catch (error) {
    toast.notifyError(error, 'Fatture non caricate')
  }
})
</script>

<template>
  <StatTile label="In carico" :value="String(inCare)" :icon="Activity" tone="accent" :hint="`fatturati negli ultimi ${CARE_WINDOW_DAYS} giorni · ${total - inCare} inattivi`">
    <div class="h-1.5 overflow-hidden rounded-full bg-accent-soft">
      <div class="h-full rounded-full bg-gradient-to-r from-accent/70 to-accent transition-[width] duration-700 ease-out-expo" :style="{ width: `${share}%` }" />
    </div>
  </StatTile>
</template>

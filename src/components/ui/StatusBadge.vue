<script setup lang="ts">
import { computed } from 'vue'

/**
 * Colored pill badge for invoice and appointment statuses.
 */
const props = defineProps<{
  status: string
  type: 'invoice' | 'appointment'
}>()

const INVOICE_CLASSES: Record<string, string> = {
  draft: 'bg-gray-100 text-gray-600',
  issued: 'bg-blue-100 text-blue-700',
  paid: 'bg-green-100 text-green-700',
  overdue: 'bg-red-100 text-red-700',
  cancelled: 'bg-gray-100 text-gray-500',
}

const INVOICE_LABELS: Record<string, string> = {
  draft: 'Bozza',
  issued: 'Emessa',
  paid: 'Pagata',
  overdue: 'Scaduta',
  cancelled: 'Annullata',
}

const APPOINTMENT_CLASSES: Record<string, string> = {
  scheduled: 'bg-blue-100 text-blue-700',
  completed: 'bg-green-100 text-green-700',
  cancelled: 'bg-gray-100 text-gray-500',
}

const APPOINTMENT_LABELS: Record<string, string> = {
  scheduled: 'Programmato',
  completed: 'Completato',
  cancelled: 'Annullato',
}

const badgeClass = computed(() => {
  const map = props.type === 'invoice' ? INVOICE_CLASSES : APPOINTMENT_CLASSES
  return map[props.status] ?? 'bg-gray-100 text-gray-600'
})

const label = computed(() => {
  const map = props.type === 'invoice' ? INVOICE_LABELS : APPOINTMENT_LABELS
  return map[props.status] ?? props.status
})
</script>

<template>
  <span
    class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium"
    :class="badgeClass"
  >
    {{ label }}
  </span>
</template>

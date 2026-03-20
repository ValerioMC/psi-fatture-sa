<script setup lang="ts">
import { computed } from 'vue'

defineProps<{
  status: string
  type: 'invoice' | 'appointment'
}>()

const INVOICE_CLASSES: Record<string, string> = {
  draft:     'bg-warm-100 text-warm-700',
  issued:    'bg-ocean-100 text-ocean-700',
  paid:      'bg-sage-100 text-sage-700',
  overdue:   'bg-red-100 text-red-700',
  cancelled: 'bg-warm-100/60 text-warm-500',
}

const INVOICE_LABELS: Record<string, string> = {
  draft:     'Bozza',
  issued:    'Emessa',
  paid:      'Pagata',
  overdue:   'Scaduta',
  cancelled: 'Annullata',
}

const APPOINTMENT_CLASSES: Record<string, string> = {
  scheduled: 'bg-ocean-100 text-ocean-700',
  completed: 'bg-sage-100 text-sage-700',
  cancelled: 'bg-warm-100/60 text-warm-500',
}

const APPOINTMENT_LABELS: Record<string, string> = {
  scheduled: 'Programmato',
  completed: 'Completato',
  cancelled: 'Annullato',
}

const props = defineProps<{ status: string; type: 'invoice' | 'appointment' }>()

const badgeClass = computed(() => {
  const map = props.type === 'invoice' ? INVOICE_CLASSES : APPOINTMENT_CLASSES
  return map[props.status] ?? 'bg-warm-100 text-warm-600'
})

const label = computed(() => {
  const map = props.type === 'invoice' ? INVOICE_LABELS : APPOINTMENT_LABELS
  return map[props.status] ?? props.status
})
</script>

<template>
  <span
    class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium tracking-wide"
    :class="badgeClass"
  >
    {{ label }}
  </span>
</template>

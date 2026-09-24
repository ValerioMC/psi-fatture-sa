<script setup lang="ts">
import { computed } from 'vue'
import type { AppointmentStatus, InvoiceStatus } from '@/types'
import { APPOINTMENT_STATUS, INVOICE_STATUS, type StatusPresentation } from '@/utils/labels'
import AppBadge from './AppBadge.vue'

const props = defineProps<
  | { type: 'invoice'; status: InvoiceStatus }
  | { type: 'appointment'; status: AppointmentStatus }
>()

const presentation = computed<StatusPresentation>(() =>
  props.type === 'invoice' ? INVOICE_STATUS[props.status] : APPOINTMENT_STATUS[props.status],
)
</script>

<template>
  <AppBadge :tone="presentation.tone" dot :strike="status === 'cancelled'">{{ presentation.label }}</AppBadge>
</template>

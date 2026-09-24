<script setup lang="ts">
/**
 * Where a year of compensi goes: what stays with you, what goes to the
 * pension fund, what goes in tax. Emphasis, not categories: the part you keep
 * is in ink, the two deductions are neutral greys, and a legend names all three.
 */
import { computed } from 'vue'
import { formatCurrency } from '@/utils/format'

const props = defineProps<{
  revenue: number
  net: number
  contributions: number
  tax: number
  taxLabel: string
}>()

const segments = computed(() => {
  const total = Math.max(props.revenue, 1)
  return [
    { key: 'net', label: 'Ti resta', value: props.net, swatch: 'bg-accent', share: props.net / total },
    { key: 'contributions', label: 'Contributi previdenziali', value: props.contributions, swatch: 'bg-text-subtle/70', share: props.contributions / total },
    { key: 'tax', label: props.taxLabel, value: props.tax, swatch: 'bg-border-strong', share: props.tax / total },
  ]
})

const netPercent = computed(() => Math.round((props.net / Math.max(props.revenue, 1)) * 100))
</script>

<template>
  <div>
    <p class="label-quiet">Netto stimato</p>
    <p class="mt-0.5 text-2xl font-semibold tracking-[-0.01em] text-text">
      {{ formatCurrency(net) }}
      <span class="text-base font-normal text-text-subtle">{{ netPercent }}%</span>
    </p>

    <!-- Segments separated by a 2px surface gap, never by a stroke. -->
    <div class="mt-4 flex h-2.5 gap-0.5" role="img" :aria-label="segments.map((s) => `${s.label} ${formatCurrency(s.value)}`).join(', ')">
      <div
        v-for="segment in segments"
        :key="segment.key"
        class="h-full first:rounded-l-full last:rounded-r-full transition-[flex-grow] duration-700 ease-out-expo"
        :class="segment.swatch"
        :style="{ flexGrow: Math.max(segment.share, 0.001), flexBasis: 0 }"
      />
    </div>

    <dl class="mt-4 space-y-2">
      <div v-for="segment in segments" :key="segment.key" class="flex items-center gap-2.5 text-sm">
        <span class="size-2 shrink-0 rounded-[2px]" :class="segment.swatch" aria-hidden="true" />
        <dt class="flex-1 text-text-muted">{{ segment.label }}</dt>
        <dd class="tabular font-medium text-text">{{ formatCurrency(segment.value) }}</dd>
      </div>
    </dl>
  </div>
</template>

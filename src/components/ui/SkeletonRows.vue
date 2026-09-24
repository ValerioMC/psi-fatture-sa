<script setup lang="ts">
/**
 * Placeholder shaped like what it stands in for. Table and list rows use the
 * real row height, so nothing reflows when the data arrives.
 */
withDefaults(
  defineProps<{
    variant?: 'table' | 'list' | 'cards'
    count?: number
    label?: string
  }>(),
  { variant: 'table', count: 6, label: 'Caricamento in corso' },
)

/** Varied widths so the placeholder reads as text, not as a barcode. */
const WIDTHS = ['62%', '48%', '71%', '55%', '66%', '44%', '58%', '69%']
</script>

<template>
  <div role="status" aria-live="polite" aria-busy="true">
    <span class="sr-only">{{ label }}</span>

    <div v-if="variant === 'cards'" class="grid grid-cols-2 lg:grid-cols-3 gap-4">
      <div v-for="index in count" :key="index" class="rounded-card border border-border bg-surface-raised p-5">
        <div class="skeleton h-3 w-1/3 mb-4" />
        <div class="skeleton h-6 w-2/3 mb-3" />
        <div class="skeleton h-3 w-1/2" />
      </div>
    </div>

    <div v-else>
      <div
        v-for="index in count"
        :key="index"
        class="flex items-center gap-4 h-row px-5 border-b border-border last:border-b-0"
      >
        <div v-if="variant === 'table'" class="skeleton size-7 rounded-full shrink-0" />
        <div class="skeleton h-3" :style="{ width: WIDTHS[index % WIDTHS.length] }" />
        <div v-if="variant === 'table'" class="skeleton h-3 w-16 ml-auto" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/** ‹ 2026 › — steps through years or months; the label is announced when it changes. */
import { ChevronLeft, ChevronRight } from 'lucide-vue-next'

withDefaults(
  defineProps<{
    label: string
    previousLabel: string
    nextLabel: string
    canNext?: boolean
    minWidth?: string
  }>(),
  { canNext: true, minWidth: '4.5rem' },
)

defineEmits<{ previous: []; next: [] }>()
</script>

<template>
  <div class="inline-flex h-control items-center rounded-control border border-border-strong bg-surface-raised shadow-[0_1px_1px_rgb(40_34_20/0.04)]">
    <button
      type="button"
      class="grid h-full w-8 place-items-center rounded-l-control text-text-subtle transition-colors hover:bg-surface-hover hover:text-text focus-ring"
      :aria-label="previousLabel"
      @click="$emit('previous')"
    >
      <ChevronLeft :size="16" :stroke-width="1.9" aria-hidden="true" />
    </button>
    <span
      v-if="label"
      class="tabular px-2 text-center text-base font-medium text-text"
      :style="{ minWidth }"
      aria-live="polite"
    >{{ label }}</span>
    <button
      type="button"
      class="grid h-full w-8 place-items-center rounded-r-control text-text-subtle transition-colors hover:bg-surface-hover hover:text-text focus-ring disabled:opacity-35 disabled:hover:bg-transparent"
      :aria-label="nextLabel"
      :disabled="!canNext"
      @click="$emit('next')"
    >
      <ChevronRight :size="16" :stroke-width="1.9" aria-hidden="true" />
    </button>
  </div>
</template>

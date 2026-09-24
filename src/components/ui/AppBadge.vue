<script setup lang="ts">
/**
 * A short label with a tone. Each tone is a fixed wash / text / ring triple
 * derived from its status token, so a badge can never invent its own colour.
 */
import type { Tone } from '@/utils/labels'

withDefaults(
  defineProps<{
    tone?: Tone
    dot?: boolean
    mono?: boolean
    strike?: boolean
  }>(),
  { tone: 'neutral', dot: false, mono: false, strike: false },
)

const TONE_CLASS: Record<Tone, string> = {
  neutral: 'bg-surface-sunken text-text-muted ring-border-strong',
  accent: 'bg-accent-soft text-accent ring-accent-line',
  safe: 'bg-safe-soft text-safe ring-safe-line',
  warn: 'bg-warn-soft text-warn ring-warn-line',
  danger: 'bg-danger-soft text-danger ring-danger-line',
}

const DOT_CLASS: Record<Tone, string> = {
  neutral: 'bg-text-subtle',
  accent: 'bg-accent',
  safe: 'bg-safe',
  warn: 'bg-warn',
  danger: 'bg-danger',
}
</script>

<template>
  <span
    class="inline-flex items-center gap-1.5 h-5 px-2 rounded-full text-xs font-medium ring-1 ring-inset whitespace-nowrap"
    :class="[TONE_CLASS[tone], mono ? 'font-mono tabular' : '', strike ? 'line-through decoration-1' : '']"
  >
    <span v-if="dot" class="size-1.5 rounded-full shrink-0" :class="DOT_CLASS[tone]" aria-hidden="true" />
    <slot />
  </span>
</template>

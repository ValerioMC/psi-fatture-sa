<script setup lang="ts">
/**
 * One figure, named and explained: an icon tile in the figure's tone, a quiet
 * label, the value in large tabular digits, one line of context, and an
 * optional instrument at the foot (a sparkline, a meter) in the default slot.
 * A soft wash of the tone rises from the top corner, so a row of tiles reads
 * as a set of distinct instruments rather than a strip of numbers.
 */
import type { Component } from 'vue'
import type { Tone } from '@/utils/labels'

withDefaults(
  defineProps<{
    label: string
    value: string
    hint?: string
    icon?: Component
    tone?: Tone
  }>(),
  { tone: 'accent' },
)

const TONE_COLOR: Record<Tone, string> = {
  neutral: 'var(--text-muted)',
  accent: 'var(--accent)',
  safe: 'var(--safe)',
  warn: 'var(--warn)',
  danger: 'var(--danger)',
}
</script>

<template>
  <div class="stat-tile sheet relative flex min-w-0 flex-col overflow-hidden p-4.5" :style="{ '--tone': TONE_COLOR[tone] }">
    <div class="flex items-start justify-between gap-3">
      <div class="min-w-0">
        <p class="label-quiet truncate">{{ label }}</p>
        <p class="tabular mt-1 truncate text-2xl font-semibold tracking-[-0.02em] text-text" aria-live="polite">{{ value }}</p>
      </div>
      <span v-if="icon" class="icon-chip" :style="{ '--chip': 'var(--tone)' }" aria-hidden="true">
        <component :is="icon" :size="16" :stroke-width="1.8" />
      </span>
    </div>
    <p v-if="hint || $slots.hint" class="mt-1 truncate text-xs text-text-subtle"><slot name="hint">{{ hint }}</slot></p>
    <div v-if="$slots.default" class="mt-auto pt-3">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.stat-tile::before {
  content: '';
  position: absolute;
  inset: 0;
  pointer-events: none;
  background: radial-gradient(120% 90% at 100% 0%, color-mix(in srgb, var(--tone) 9%, transparent), transparent 55%);
}
.stat-tile > * { position: relative; }
</style>

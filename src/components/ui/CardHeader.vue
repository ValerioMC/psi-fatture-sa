<script setup lang="ts">
/**
 * The head of a card: what it is about (icon tile, title, one line of context)
 * and, on the right, whatever acts on it. Every card in the app opens the same
 * way, so the eye learns where to look once.
 */
import type { Component } from 'vue'
import type { Tone } from '@/utils/labels'

withDefaults(
  defineProps<{
    title: string
    subtitle?: string
    icon?: Component
    tone?: Tone
    /** Draws a hairline under the header, for cards whose body is a list. */
    divided?: boolean
  }>(),
  { tone: 'accent', divided: false },
)

const CHIP_COLOR: Record<Tone, string> = {
  neutral: 'var(--text-muted)',
  accent: 'var(--accent)',
  safe: 'var(--safe)',
  warn: 'var(--warn)',
  danger: 'var(--danger)',
}
</script>

<template>
  <div class="flex items-center gap-3 px-5 pt-4.5 pb-4" :class="divided ? 'border-b border-border' : ''">
    <span v-if="icon" class="icon-chip" :style="{ '--chip': CHIP_COLOR[tone] }" aria-hidden="true">
      <component :is="icon" :size="16" :stroke-width="1.8" />
    </span>
    <div class="min-w-0 flex-1">
      <h2 class="truncate text-md font-semibold tracking-[-0.005em] text-text">{{ title }}</h2>
      <p v-if="subtitle || $slots.subtitle" class="truncate text-sm text-text-subtle"><slot name="subtitle">{{ subtitle }}</slot></p>
    </div>
    <div v-if="$slots.default" class="flex shrink-0 items-center gap-2">
      <slot />
    </div>
  </div>
</template>

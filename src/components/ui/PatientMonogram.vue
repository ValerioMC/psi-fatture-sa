<script setup lang="ts">
/**
 * A patient's initials on their own muted tint: the same person always wears
 * the same colour, and no tint is ever louder than the ink accent.
 */
import { computed } from 'vue'
import { initialsOf, tintIndexOf } from '@/utils/monogram'

const props = withDefaults(defineProps<{ name: string; size?: 'sm' | 'md' | 'lg' }>(), { size: 'md' })

const initials = computed(() => initialsOf(props.name))
const tint = computed(() => `var(--tint-${tintIndexOf(props.name)})`)

const SIZE_CLASS = {
  sm: 'size-6 text-[10px]',
  md: 'size-8 text-xs',
  lg: 'size-11 text-md',
} as const
</script>

<template>
  <span
    class="inline-grid shrink-0 place-items-center rounded-full font-semibold tracking-[0.02em] ring-1 ring-inset"
    :class="SIZE_CLASS[size]"
    :style="{
      color: tint,
      backgroundColor: `color-mix(in srgb, ${tint} 13%, var(--surface-raised))`,
      '--tw-ring-color': `color-mix(in srgb, ${tint} 24%, transparent)`,
    }"
    aria-hidden="true"
  >
    {{ initials }}
  </span>
</template>

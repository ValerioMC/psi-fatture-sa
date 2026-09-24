<script setup lang="ts">
/**
 * A sheet on the desk. Resting cards are flat with a hairline; `interactive`
 * cards lift on hover (1px rise, border warms, shadow appears), so something
 * clickable is recognisable before the pointer reaches it.
 */
withDefaults(
  defineProps<{
    as?: string
    interactive?: boolean
    padded?: boolean
  }>(),
  { as: 'section', interactive: false, padded: true },
)
</script>

<template>
  <component
    :is="as"
    class="bg-surface-raised border border-border rounded-card"
    :class="[
      padded ? 'p-5' : '',
      interactive
        ? 'transition-[transform,box-shadow,border-color] duration-200 hover:-translate-y-px hover:shadow-lift hover:border-border-strong cursor-pointer focus-ring'
        : '',
    ]"
  >
    <slot />
  </component>
</template>

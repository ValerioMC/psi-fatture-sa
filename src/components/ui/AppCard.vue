<script setup lang="ts">
/**
 * A sheet on the desk. Resting cards sit just off the canvas, with a lit top
 * edge; `interactive` cards lift on hover (1px rise, border warms, shadow
 * deepens), so something clickable is recognisable before the pointer arrives.
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
    class="sheet"
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

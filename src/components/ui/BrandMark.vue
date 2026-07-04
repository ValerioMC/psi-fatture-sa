<script setup lang="ts">
/**
 * App brand mark: the Ψ glyph on an indigo gradient squircle,
 * matching the desktop app icon.
 */
withDefaults(defineProps<{ size?: number }>(), { size: 36 })
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 36 36"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
  >
    <defs>
      <linearGradient id="brand-bg" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="#1e1b4b" />
        <stop offset="1" stop-color="#4338ca" />
      </linearGradient>
    </defs>
    <rect width="36" height="36" rx="9" fill="url(#brand-bg)" />
    <g stroke="#ffffff" stroke-width="2.6" stroke-linecap="round" fill="none">
      <path d="M 11.5,10.5 v 4.2 a 6.5,6.5 0 0 0 13,0 v -4.2" />
      <path d="M 18,9.5 V 26.5" />
    </g>
  </svg>
</template>

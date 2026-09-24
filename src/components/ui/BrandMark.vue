<script setup lang="ts">
/**
 * The brand mark: the Ψ drawn in white on an ink squircle, echoing the
 * desktop app icon. Brand colours live here and in the icon files only;
 * the interface itself uses the ink accent token.
 */
withDefaults(defineProps<{ size?: number }>(), { size: 28 })
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 36 36" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
    <defs>
      <linearGradient id="brand-mark-bg" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#5b5fe6" />
        <stop offset="0.6" stop-color="#4146b8" />
        <stop offset="1" stop-color="#2c2f80" />
      </linearGradient>
      <radialGradient id="brand-mark-glow" cx="0.5" cy="0.2" r="0.8">
        <stop offset="0" stop-color="#ffffff" stop-opacity="0.28" />
        <stop offset="1" stop-color="#ffffff" stop-opacity="0" />
      </radialGradient>
    </defs>
    <rect width="36" height="36" rx="10" fill="url(#brand-mark-bg)" />
    <rect width="36" height="36" rx="10" fill="url(#brand-mark-glow)" />
    <rect x="0.5" y="0.5" width="35" height="35" rx="9.5" stroke="#ffffff" stroke-opacity="0.14" />
    <g stroke="#ffffff" stroke-width="2.5" stroke-linecap="round" fill="none">
      <path d="M11.5 10.5v4.2a6.5 6.5 0 0 0 13 0v-4.2" />
      <path d="M18 9.5v17" />
    </g>
  </svg>
</template>

<script setup lang="ts">
/**
 * The brand mark: the desktop app icon itself, cropped to its squircle, so
 * the sidebar and the Dock show the same Ψ and hawkmoth.
 */
import brandMarkUrl from '@/assets/brand-mark.png'

withDefaults(defineProps<{ size?: number }>(), { size: 28 })
</script>

<template>
  <img :src="brandMarkUrl" :width="size" :height="size" alt="" aria-hidden="true" draggable="false" class="select-none" />
</template>

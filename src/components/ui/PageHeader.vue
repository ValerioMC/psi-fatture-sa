<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
}>()
</script>

<template>
  <div class="flex items-center justify-between mb-7">
    <div>
      <h1 class="text-2xl font-semibold text-sage-900 tracking-tight">{{ title }}</h1>
      <p v-if="subtitle" class="text-sm text-sage-500 mt-0.5">{{ subtitle }}</p>
    </div>
    <div class="flex items-center gap-2">
      <slot />
    </div>
  </div>
</template>

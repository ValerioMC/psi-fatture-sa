<script setup lang="ts">
/**
 * One group of a long form: what it is and why it is asked on the left, the
 * fields on the right. The explanation sits beside the fields, where it is
 * read, rather than in a hint under each one.
 */
defineProps<{ title: string; description?: string }>()
</script>

<template>
  <section class="grid grid-cols-1 gap-x-10 gap-y-4 border-b border-border py-8 first:pt-2 last:border-b-0 md:grid-cols-[15rem_1fr]">
    <div>
      <h2 class="text-md font-medium text-text">{{ title }}</h2>
      <p v-if="description" class="mt-1 text-sm text-text-muted">{{ description }}</p>
      <slot name="aside" />
    </div>
    <div class="min-w-0">
      <slot />
    </div>
  </section>
</template>

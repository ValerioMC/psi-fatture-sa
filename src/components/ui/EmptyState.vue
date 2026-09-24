<script setup lang="ts">
/**
 * Calm blank space, not an error: a dashed outline, generous air, one line
 * of what is missing and, when there is one, the action that fills it.
 */
import type { Component } from 'vue'
import { Inbox } from 'lucide-vue-next'

withDefaults(
  defineProps<{
    title: string
    description?: string
    icon?: Component
    bordered?: boolean
  }>(),
  { icon: () => Inbox, bordered: true },
)
</script>

<template>
  <div
    class="flex flex-col items-center justify-center text-center px-6 py-14"
    :class="bordered ? 'border border-dashed border-border-strong rounded-card' : ''"
  >
    <div class="mb-4 grid size-11 place-items-center rounded-full bg-surface-sunken text-text-subtle ring-1 ring-border">
      <component :is="icon" :size="20" :stroke-width="1.6" aria-hidden="true" />
    </div>
    <p class="text-md font-medium text-text">{{ title }}</p>
    <p v-if="description" class="mt-1 max-w-sm text-sm text-text-muted">{{ description }}</p>
    <div v-if="$slots.default" class="mt-5 flex items-center gap-2">
      <slot />
    </div>
  </div>
</template>

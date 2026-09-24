<script setup lang="ts">
/**
 * The top of every page: sticky glass over the ambient room, so the title and
 * the page's actions stay reachable while the content scrolls beneath them.
 */
import { ChevronLeft } from 'lucide-vue-next'
import { RouterLink, type RouteLocationRaw } from 'vue-router'

defineProps<{
  title: string
  subtitle?: string
  back?: { to: RouteLocationRaw; label: string }
}>()
</script>

<template>
  <header class="glass sticky top-0 z-(--z-sticky) border-b border-border/70" data-tauri-drag-region>
    <div class="mx-auto flex max-w-[72rem] items-end justify-between gap-6 px-8 pt-7 pb-5" data-tauri-drag-region>
      <div class="min-w-0" data-tauri-drag-region>
        <RouterLink
          v-if="back"
          :to="back.to"
          class="group -ml-1 mb-1.5 inline-flex items-center gap-0.5 rounded-[6px] px-1 text-sm text-text-subtle hover:text-text focus-ring"
        >
          <ChevronLeft :size="15" :stroke-width="2" class="transition-transform group-hover:-translate-x-0.5" aria-hidden="true" />
          {{ back.label }}
        </RouterLink>
        <p v-else-if="$slots.eyebrow" class="mb-1 text-sm text-text-subtle" data-tauri-drag-region><slot name="eyebrow" /></p>
        <h1 class="display truncate text-3xl text-text" data-tauri-drag-region>
          <slot name="title">{{ title }}</slot>
        </h1>
        <p v-if="subtitle" class="mt-1 text-base text-text-muted" data-tauri-drag-region>{{ subtitle }}</p>
      </div>
      <div v-if="$slots.default" class="flex shrink-0 items-center gap-2">
        <slot />
      </div>
    </div>
  </header>
</template>

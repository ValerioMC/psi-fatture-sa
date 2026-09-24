<script setup lang="ts">
/**
 * The top of every page: sticky glass over the ambient room, so the title and
 * the page's actions stay reachable while the content scrolls beneath them.
 */
import type { Component } from 'vue'
import { ChevronLeft } from 'lucide-vue-next'
import { RouterLink, type RouteLocationRaw } from 'vue-router'

defineProps<{
  title: string
  subtitle?: string
  back?: { to: RouteLocationRaw; label: string }
  /** The section's icon, set in an ink tile beside the title. */
  icon?: Component
}>()
</script>

<template>
  <header class="page-header glass sticky top-0 z-(--z-sticky)" data-tauri-drag-region>
    <div class="page flex items-end justify-between gap-6 pt-7 pb-5" data-tauri-drag-region>
      <div class="flex min-w-0 items-end gap-4" data-tauri-drag-region>
        <span v-if="icon && !back" class="page-icon hidden sm:grid" aria-hidden="true">
          <component :is="icon" :size="20" :stroke-width="1.7" />
        </span>
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
      </div>
      <div v-if="$slots.default" class="flex shrink-0 items-center gap-2">
        <slot />
      </div>
    </div>
  </header>
</template>

<style scoped>
/* The hairline under the header fades out at both ends, like a ruled line on paper. */
.page-header::after {
  content: '';
  position: absolute;
  inset-inline: 0;
  bottom: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--border-strong) 12%, var(--border-strong) 88%, transparent);
  opacity: 0.8;
}

/* The section's ink tile: a small plate of the accent, lit from above. */
.page-icon {
  place-items: center;
  width: 2.75rem;
  height: 2.75rem;
  margin-bottom: 0.125rem;
  border-radius: 12px;
  color: var(--accent-ink);
  background: linear-gradient(160deg, color-mix(in srgb, var(--accent) 78%, white), var(--accent) 55%, var(--accent-strong));
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.25), 0 1px 2px rgb(20 18 60 / 0.2), 0 8px 18px -8px color-mix(in srgb, var(--accent) 70%, transparent);
}
:root[data-theme='dark'] .page-icon {
  background: linear-gradient(160deg, var(--accent-strong), var(--accent) 60%, color-mix(in srgb, var(--accent) 70%, black));
}
</style>

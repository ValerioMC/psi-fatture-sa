<script setup lang="ts">
/**
 * The settings grouped by what they are for, one place per context. Each place
 * says in one line what it holds and, where something is missing, a status dot
 * says so before it is opened. The active plate glides like the app sidebar's.
 */
import { nextTick, onMounted, ref, watch, type Component } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import type { Tone } from '@/utils/labels'

export interface SettingsNavItem {
  to: string
  label: string
  hint: string
  icon: Component
  status?: { tone: Tone; label: string }
}

defineProps<{ items: readonly SettingsNavItem[] }>()

const route = useRoute()
const navRef = ref<HTMLElement | null>(null)
const marker = ref<{ top: number; height: number } | null>(null)
const markerReady = ref(false)

function placeMarker(): void {
  const active = navRef.value?.querySelector<HTMLElement>('[aria-current="page"]')
  marker.value = active ? { top: active.offsetTop, height: active.offsetHeight } : null
}

watch(
  () => route.path,
  async () => {
    await nextTick()
    placeMarker()
  },
)
onMounted(async () => {
  await nextTick()
  placeMarker()
  requestAnimationFrame(() => {
    markerReady.value = true
  })
})

const DOT: Record<Tone, string> = {
  neutral: 'bg-text-subtle',
  accent: 'bg-accent',
  safe: 'bg-safe',
  warn: 'bg-warn',
  danger: 'bg-danger',
}
</script>

<template>
  <nav ref="navRef" class="relative" aria-label="Sezioni delle impostazioni">
    <span
      v-if="marker"
      class="settings-marker pointer-events-none absolute inset-x-0 rounded-control"
      :class="markerReady ? 'transition-[transform,height] duration-300 ease-out-expo' : ''"
      :style="{ transform: `translateY(${marker.top}px)`, height: `${marker.height}px`, top: 0 }"
      aria-hidden="true"
    >
      <span class="absolute -left-px top-1/2 h-5 w-[3px] -translate-y-1/2 rounded-r-full bg-accent shadow-glow" />
    </span>
    <ul class="space-y-1">
      <li v-for="item in items" :key="item.to">
        <RouterLink
          :to="item.to"
          class="group relative flex items-start gap-3 rounded-control px-3 py-2.5 transition-colors duration-150 focus-ring"
          :class="route.path === item.to ? '' : 'hover:bg-surface-hover'"
          :aria-current="route.path === item.to ? 'page' : undefined"
        >
          <component
            :is="item.icon"
            :size="17"
            :stroke-width="1.75"
            class="relative mt-0.5 shrink-0 transition-colors"
            :class="route.path === item.to ? 'text-accent' : 'text-text-subtle group-hover:text-text-muted'"
            aria-hidden="true"
          />
          <span class="relative min-w-0 flex-1">
            <span class="flex items-center gap-2">
              <span class="text-base" :class="route.path === item.to ? 'font-medium text-text' : 'text-text-muted group-hover:text-text'">{{ item.label }}</span>
              <span v-if="item.status" class="size-1.5 shrink-0 rounded-full" :class="DOT[item.status.tone]" :title="item.status.label" aria-hidden="true" />
              <span v-if="item.status" class="sr-only">{{ item.status.label }}</span>
            </span>
            <span class="block truncate text-xs text-text-subtle">{{ item.status?.label ?? item.hint }}</span>
          </span>
        </RouterLink>
      </li>
    </ul>
  </nav>
</template>

<style scoped>
.settings-marker {
  background: var(--surface-raised);
  box-shadow: 0 0 0 1px var(--border), var(--sheet-highlight), 0 1px 2px rgb(40 34 20 / 0.06);
}
</style>

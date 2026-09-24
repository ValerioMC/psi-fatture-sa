<script setup lang="ts">
/**
 * The sidebar: brand, the two things you do most (find something, write an
 * invoice), the five places, and who you are. Ordered by how often a
 * psychologist goes there: the week's agenda comes right after the overview.
 */
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { CalendarDays, ClipboardList, FileText, LayoutGrid, Plus, Search, Settings, Users } from 'lucide-vue-next'
import BrandMark from '@/components/ui/BrandMark.vue'
import AppButton from '@/components/ui/AppButton.vue'
import PatientMonogram from '@/components/ui/PatientMonogram.vue'
import { useConfigStore } from '@/stores/config'
import { TAX_REGIME_LABEL } from '@/utils/labels'
import { shortcutLabel } from '@/utils/platform'

defineEmits<{ openPalette: [] }>()

const route = useRoute()
const configStore = useConfigStore()

const NAV_ITEMS = [
  { to: '/dashboard', label: 'Panoramica', icon: LayoutGrid },
  { to: '/agenda', label: 'Agenda', icon: CalendarDays },
  { to: '/invoices', label: 'Fatture', icon: FileText },
  { to: '/clients', label: 'Pazienti', icon: Users },
  { to: '/services', label: 'Prestazioni', icon: ClipboardList },
] as const

function isActive(path: string): boolean {
  return route.path === path || route.path.startsWith(`${path}/`)
}

const profileName = computed(() => {
  const config = configStore.config
  return config ? `${config.first_name} ${config.last_name}`.trim() : 'Profilo'
})

const regimeLabel = computed(() => {
  const regime = configStore.config?.tax_regime
  return regime ? TAX_REGIME_LABEL[regime] : ''
})
</script>

<template>
  <aside class="sidebar flex h-full w-sidebar shrink-0 flex-col border-r border-border bg-surface">
    <div class="sidebar-top flex items-center gap-2.5 px-5 pt-6 pb-5" data-tauri-drag-region>
      <BrandMark :size="28" />
      <span class="display text-[1.1875rem] leading-none text-text" data-tauri-drag-region>PSI Fatture</span>
    </div>

    <div class="space-y-2 px-3">
      <button
        type="button"
        class="flex h-control w-full items-center gap-2.5 rounded-control border border-border bg-surface-raised px-3 text-base text-text-subtle transition-colors hover:border-border-strong hover:text-text-muted focus-ring"
        @click="$emit('openPalette')"
      >
        <Search :size="15" :stroke-width="1.8" aria-hidden="true" />
        <span class="flex-1 text-left">Cerca o vai a…</span>
        <kbd class="rounded-[5px] border border-border bg-surface-sunken px-1.5 font-sans text-2xs text-text-subtle">{{ shortcutLabel('K') }}</kbd>
      </button>
      <AppButton variant="primary" :icon="Plus" block to="/invoices/new">Nuova fattura</AppButton>
    </div>

    <nav class="mt-6 flex-1 space-y-0.5 px-3" aria-label="Sezioni">
      <RouterLink
        v-for="item in NAV_ITEMS"
        :key="item.to"
        :to="item.to"
        class="nav-item group relative flex h-9 items-center gap-3 rounded-control border px-3 text-base transition-[background-color,color,border-color] duration-150 focus-ring"
        :class="isActive(item.to)
          ? 'border-border bg-surface-raised font-medium text-text shadow-[0_1px_2px_rgb(40_34_20/0.05)]'
          : 'border-transparent text-text-muted hover:bg-surface-hover hover:text-text'"
        :aria-current="isActive(item.to) ? 'page' : undefined"
      >
        <!-- A placed marker, not a border: a short capsule of ink light in the gutter. -->
        <span
          v-if="isActive(item.to)"
          class="absolute -left-2 top-1/2 h-4 w-[3px] -translate-y-1/2 rounded-r-full bg-accent shadow-glow"
          aria-hidden="true"
        />
        <component
          :is="item.icon"
          :size="17"
          :stroke-width="1.75"
          class="shrink-0 transition-colors"
          :class="isActive(item.to) ? 'text-accent' : 'text-text-subtle group-hover:text-text-muted'"
          aria-hidden="true"
        />
        {{ item.label }}
      </RouterLink>
    </nav>

    <div class="border-t border-border p-3">
      <RouterLink
        to="/settings"
        class="group flex items-center gap-3 rounded-control px-2 py-2 transition-colors hover:bg-surface-hover focus-ring"
        :class="isActive('/settings') ? 'bg-surface-hover' : ''"
        :aria-current="isActive('/settings') ? 'page' : undefined"
      >
        <PatientMonogram :name="`${configStore.config?.first_name ?? ''} ${configStore.config?.last_name ?? ''}`" size="md" />
        <span class="min-w-0 flex-1">
          <span class="block truncate text-base font-medium text-text">{{ profileName }}</span>
          <span class="block truncate text-xs text-text-subtle">{{ regimeLabel }}</span>
        </span>
        <Settings :size="16" :stroke-width="1.75" class="shrink-0 text-text-subtle transition-transform duration-300 group-hover:rotate-45 group-hover:text-text-muted" aria-label="Impostazioni" />
      </RouterLink>
    </div>
  </aside>
</template>


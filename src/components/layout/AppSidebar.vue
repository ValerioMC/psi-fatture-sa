<script setup lang="ts">
/**
 * The sidebar: brand, the two things you do most (find something, write an
 * invoice), the places grouped by what they are for, and who you are.
 *
 * It also keeps a small pulse of the practice: today's sessions next to the
 * agenda, late invoices next to the invoices, and, on forfettario, how far
 * the year has gone towards the 85.000 € ceiling. The pulse is re-read on
 * every navigation, which is when a change made on a page can have landed.
 */
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { CalendarDays, ClipboardList, FileText, LayoutGrid, Plus, Search, Settings, Users } from 'lucide-vue-next'
import BrandMark from '@/components/ui/BrandMark.vue'
import AppButton from '@/components/ui/AppButton.vue'
import PatientMonogram from '@/components/ui/PatientMonogram.vue'
import { getDashboard, listAppointments, listInvoices } from '@/api'
import { useConfigStore } from '@/stores/config'
import { summariseAttention } from '@/utils/attention'
import { FORFETTARIO_THRESHOLD, readThreshold } from '@/utils/forfettario'
import { formatCurrencyCompact, todayIso } from '@/utils/format'
import { TAX_REGIME_LABEL } from '@/utils/labels'
import { shortcutLabel } from '@/utils/platform'

defineEmits<{ openPalette: [] }>()

const route = useRoute()
const configStore = useConfigStore()

type Badge = { value: number; tone: 'accent' | 'danger' | 'neutral'; title: string }

const todaySessions = ref(0)
const overdue = ref(0)
const drafts = ref(0)
const yearRevenue = ref<number | null>(null)

async function refreshPulse(): Promise<void> {
  const today = todayIso()
  const year = new Date().getFullYear()
  try {
    const [appointments, invoices, dashboard] = await Promise.all([
      listAppointments(today, today),
      listInvoices({ year }),
      getDashboard(year),
    ])
    todaySessions.value = appointments.filter((appointment) => appointment.status !== 'cancelled').length
    const attention = summariseAttention(invoices)
    overdue.value = attention.overdue.count
    drafts.value = attention.drafts.count
    yearRevenue.value = dashboard.total_net_revenue
  } catch {
    // The pulse is a courtesy: a failed read leaves the last known values in place.
  }
}

const NAV_GROUPS = computed(() => [
  {
    label: 'Studio',
    items: [
      { to: '/dashboard', label: 'Panoramica', icon: LayoutGrid, badge: null as Badge | null },
      {
        to: '/agenda',
        label: 'Agenda',
        icon: CalendarDays,
        badge: todaySessions.value > 0 ? { value: todaySessions.value, tone: 'accent', title: 'Sedute di oggi' } as Badge : null,
      },
    ],
  },
  {
    label: 'Archivio',
    items: [
      {
        to: '/invoices',
        label: 'Fatture',
        icon: FileText,
        badge: overdue.value > 0
          ? { value: overdue.value, tone: 'danger', title: 'Fatture scadute' } as Badge
          : drafts.value > 0 ? { value: drafts.value, tone: 'neutral', title: 'Bozze da emettere' } as Badge : null,
      },
      { to: '/clients', label: 'Pazienti', icon: Users, badge: null },
      { to: '/services', label: 'Prestazioni', icon: ClipboardList, badge: null },
    ],
  },
])

const BADGE_CLASS: Record<Badge['tone'], string> = {
  accent: 'bg-accent text-accent-ink',
  danger: 'bg-danger-soft text-danger ring-1 ring-inset ring-danger-line',
  neutral: 'bg-surface-sunken text-text-subtle ring-1 ring-inset ring-border',
}

function isActive(path: string): boolean {
  return route.path === path || route.path.startsWith(`${path}/`)
}

// ─── The sliding marker: one plate that glides to the active place ─────────

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
    void refreshPulse()
  },
)

let resizeObserver: ResizeObserver | null = null
onMounted(async () => {
  await nextTick()
  placeMarker()
  // Enable the glide only after the first placement, so the plate never flies in from the top.
  requestAnimationFrame(() => { markerReady.value = true })
  if (navRef.value) {
    resizeObserver = new ResizeObserver(placeMarker)
    resizeObserver.observe(navRef.value)
  }
  void refreshPulse()
})
onBeforeUnmount(() => resizeObserver?.disconnect())

// ─── Profile and ceiling ────────────────────────────────────────────────────

const profileName = computed(() => {
  const config = configStore.config
  return config ? `${config.first_name} ${config.last_name}`.trim() : 'Profilo'
})

const regimeLabel = computed(() => {
  const regime = configStore.config?.tax_regime
  return regime ? TAX_REGIME_LABEL[regime] : ''
})

const ceiling = computed(() => {
  if (configStore.config?.tax_regime !== 'forfettario' || yearRevenue.value === null) return null
  const reading = readThreshold(yearRevenue.value, new Date().getFullYear())
  return {
    amount: reading.amount,
    percent: Math.min(100, Math.round(reading.ratio * 100)),
    level: reading.level,
  }
})

const CEILING_FILL = {
  ok: 'from-accent/70 to-accent',
  near: 'from-warn/70 to-warn',
  over: 'from-danger/70 to-danger',
} as const
</script>

<template>
  <aside class="sidebar relative flex h-full w-sidebar shrink-0 flex-col">
    <div class="sidebar-top flex items-center gap-2.5 px-5 pt-6 pb-5" data-tauri-drag-region>
      <BrandMark :size="30" />
      <div class="min-w-0" data-tauri-drag-region>
        <span class="display block text-[1.1875rem] leading-none text-text" data-tauri-drag-region>PSI Fatture</span>
        <span class="mt-1 block text-2xs leading-none text-text-subtle" data-tauri-drag-region>Studio di psicologia</span>
      </div>
    </div>

    <div class="space-y-2 px-3">
      <button
        type="button"
        class="search-trigger flex h-control w-full items-center gap-2.5 rounded-control border border-border bg-surface-raised px-3 text-base text-text-subtle transition-[border-color,color,box-shadow] hover:border-border-strong hover:text-text-muted focus-ring"
        @click="$emit('openPalette')"
      >
        <Search :size="15" :stroke-width="1.8" aria-hidden="true" />
        <span class="flex-1 text-left">Cerca o vai a…</span>
        <kbd class="keycap">{{ shortcutLabel('K') }}</kbd>
      </button>
      <AppButton variant="primary" :icon="Plus" block to="/invoices/new">Nuova fattura</AppButton>
    </div>

    <nav ref="navRef" class="relative mt-5 flex-1 overflow-y-auto px-3" aria-label="Sezioni" data-lenis-prevent>
      <!-- The active plate: one element that glides between places. -->
      <span
        v-if="marker"
        class="nav-marker pointer-events-none absolute inset-x-3 rounded-control"
        :class="markerReady ? 'transition-[transform,height] duration-300 ease-out-expo' : ''"
        :style="{ transform: `translateY(${marker.top}px)`, height: `${marker.height}px`, top: 0 }"
        aria-hidden="true"
      >
        <span class="absolute -left-2 top-1/2 h-4 w-[3px] -translate-y-1/2 rounded-r-full bg-accent shadow-glow" />
      </span>

      <div v-for="group in NAV_GROUPS" :key="group.label" class="mb-4 last:mb-0">
        <p class="mb-1 px-3 text-2xs font-medium text-text-subtle">{{ group.label }}</p>
        <div class="space-y-0.5">
          <RouterLink
            v-for="item in group.items"
            :key="item.to"
            :to="item.to"
            class="group relative flex h-9 items-center gap-3 rounded-control px-3 text-base transition-colors duration-150 focus-ring"
            :class="isActive(item.to) ? 'font-medium text-text' : 'text-text-muted hover:bg-surface-hover hover:text-text'"
            :aria-current="isActive(item.to) ? 'page' : undefined"
          >
            <component
              :is="item.icon"
              :size="17"
              :stroke-width="1.75"
              class="relative shrink-0 transition-colors"
              :class="isActive(item.to) ? 'text-accent' : 'text-text-subtle group-hover:text-text-muted'"
              aria-hidden="true"
            />
            <span class="relative flex-1">{{ item.label }}</span>
            <Transition name="swap">
              <span
                v-if="item.badge"
                class="tabular relative grid h-[1.125rem] min-w-[1.125rem] place-items-center rounded-full px-1.5 text-2xs font-semibold"
                :class="BADGE_CLASS[item.badge.tone]"
                :title="item.badge.title"
              >
                {{ item.badge.value }}<span class="sr-only"> {{ item.badge.title.toLowerCase() }}</span>
              </span>
            </Transition>
          </RouterLink>
        </div>
      </div>
    </nav>

    <!-- The ceiling, always in view: the number a forfettario watches most. -->
    <RouterLink
      v-if="ceiling"
      to="/dashboard"
      class="ceiling-card group mx-3 mb-3 block rounded-card border border-border p-3 transition-[border-color,box-shadow] hover:border-border-strong focus-ring"
      :title="`Compensi ${new Date().getFullYear()} rispetto alla soglia di ${formatCurrencyCompact(FORFETTARIO_THRESHOLD)}`"
    >
      <span class="flex items-baseline justify-between gap-2">
        <span class="text-2xs font-medium text-text-subtle">Soglia forfettario</span>
        <span
          class="tabular text-2xs font-semibold"
          :class="ceiling.level === 'over' ? 'text-danger' : ceiling.level === 'near' ? 'text-warn' : 'text-accent'"
        >{{ ceiling.percent }}%</span>
      </span>
      <span class="tabular mt-0.5 block text-sm font-semibold text-text">
        {{ formatCurrencyCompact(ceiling.amount) }} <span class="font-normal text-text-subtle">/ {{ formatCurrencyCompact(FORFETTARIO_THRESHOLD) }}</span>
      </span>
      <span class="mt-2 block h-1.5 overflow-hidden rounded-full bg-surface-sunken">
        <span
          class="block h-full rounded-full bg-gradient-to-r transition-[width] duration-700 ease-out-expo"
          :class="CEILING_FILL[ceiling.level]"
          :style="{ width: `${ceiling.percent}%` }"
        />
      </span>
    </RouterLink>

    <div class="border-t border-border p-3">
      <RouterLink
        to="/settings"
        class="group flex items-center gap-3 rounded-control px-2 py-2 transition-colors hover:bg-surface-hover focus-ring"
        :class="isActive('/settings') ? 'bg-surface-hover' : ''"
        :aria-current="isActive('/settings') ? 'page' : undefined"
      >
        <span class="relative">
          <PatientMonogram :name="`${configStore.config?.first_name ?? ''} ${configStore.config?.last_name ?? ''}`" size="md" />
          <span class="absolute -right-0.5 -bottom-0.5 size-2.5 rounded-full bg-safe ring-2 ring-surface" aria-hidden="true" />
        </span>
        <span class="min-w-0 flex-1">
          <span class="block truncate text-base font-medium text-text">{{ profileName }}</span>
          <span class="block truncate text-xs text-text-subtle">{{ regimeLabel }}</span>
        </span>
        <Settings :size="16" :stroke-width="1.75" class="shrink-0 text-text-subtle transition-transform duration-300 group-hover:rotate-45 group-hover:text-text-muted" aria-label="Impostazioni" />
      </RouterLink>
    </div>
  </aside>
</template>

<style scoped>
/* The pane: a touch lighter at the top, like a desk lamp falling on it, with a ruled right edge. */
.sidebar {
  background:
    radial-gradient(140% 40% at 0% 0%, color-mix(in srgb, var(--accent) 5%, transparent), transparent 70%),
    linear-gradient(180deg, var(--surface), color-mix(in srgb, var(--surface) 70%, var(--canvas)));
  box-shadow: inset -1px 0 0 var(--border);
}

.search-trigger { box-shadow: var(--sheet-highlight), 0 1px 1px rgb(40 34 20 / 0.03); }

.nav-marker {
  background: var(--surface-raised);
  box-shadow: 0 0 0 1px var(--border), var(--sheet-highlight), 0 1px 2px rgb(40 34 20 / 0.06);
}

.ceiling-card {
  background: linear-gradient(180deg, var(--surface-raised), color-mix(in srgb, var(--surface-raised) 60%, var(--surface)));
  box-shadow: var(--sheet-highlight);
}
</style>

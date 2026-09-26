<script setup lang="ts">
/**
 * ⌘K: one box to go anywhere, start anything, or open a patient by typing a
 * few letters of their name or codice fiscale. Keyboard first: arrows move,
 * Enter runs, Escape closes.
 */
import { computed, nextTick, ref, toRef, watch, type Component } from 'vue'
import { useRouter } from 'vue-router'
import {
  CalendarDays, CalendarRange, ClipboardList, CornerDownLeft, FileText, IdCard, LayoutGrid,
  Moon, Plus, Search, Settings, Sun, UserPlus, Users,
} from 'lucide-vue-next'
import { useFocusTrap } from '@/composables/useFocusTrap'
import { useTheme } from '@/composables/useTheme'
import { useClientsStore } from '@/stores/clients'
import { clientDisplayName } from '@/utils/client'
import PatientMonogram from '@/components/ui/PatientMonogram.vue'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()

const router = useRouter()
const clientsStore = useClientsStore()
const { resolved, setPreference } = useTheme()

type Group = 'Azioni' | 'Vai a' | 'Pazienti'

interface PaletteItem {
  id: string
  group: Group
  label: string
  detail?: string
  keywords?: string
  icon?: Component
  patientName?: string
  run: () => void
}

const query = ref('')
const activeIndex = ref(0)
const panelRef = ref<HTMLElement | null>(null)
const listRef = ref<HTMLElement | null>(null)

function go(path: string): () => void {
  return () => { void router.push(path) }
}

const staticItems = computed<PaletteItem[]>(() => [
  { id: 'new-invoice', group: 'Azioni', label: 'Nuova fattura', icon: Plus, run: go('/invoices/new') },
  { id: 'new-patient', group: 'Azioni', label: 'Nuovo paziente', icon: UserPlus, run: go('/clients/new') },
  { id: 'monthly', group: 'Azioni', label: 'Fatturazione mensile', keywords: 'genera mese sedute', icon: CalendarRange, run: go('/invoices/monthly') },
  {
    id: 'theme',
    group: 'Azioni',
    label: resolved.value === 'dark' ? 'Passa al tema chiaro' : 'Passa al tema scuro',
    keywords: 'tema aspetto notte giorno dark light',
    icon: resolved.value === 'dark' ? Sun : Moon,
    run: () => setPreference(resolved.value === 'dark' ? 'light' : 'dark'),
  },
  { id: 'go-dashboard', group: 'Vai a', label: 'Panoramica', keywords: 'dashboard', icon: LayoutGrid, run: go('/dashboard') },
  { id: 'go-agenda', group: 'Vai a', label: 'Agenda', keywords: 'calendario appuntamenti sedute', icon: CalendarDays, run: go('/agenda') },
  { id: 'go-invoices', group: 'Vai a', label: 'Fatture', icon: FileText, run: go('/invoices') },
  { id: 'go-patients', group: 'Vai a', label: 'Pazienti', keywords: 'clienti', icon: Users, run: go('/clients') },
  { id: 'go-services', group: 'Vai a', label: 'Prestazioni', keywords: 'tariffe servizi', icon: ClipboardList, run: go('/services') },
  { id: 'go-sts', group: 'Vai a', label: 'Sistema TS', keywords: 'tessera sanitaria spese sanitarie precompilata 730 trasmissione', icon: IdCard, run: go('/sts') },
  { id: 'go-settings', group: 'Vai a', label: 'Impostazioni', keywords: 'profilo partita iva iban', icon: Settings, run: go('/settings') },
])

const patientItems = computed<PaletteItem[]>(() =>
  clientsStore.clients.map((client) => {
    const name = clientDisplayName(client)
    return {
      id: `patient-${client.id}`,
      group: 'Pazienti' as const,
      label: name,
      detail: client.fiscal_code,
      keywords: `${client.city} ${client.email ?? ''}`,
      patientName: name,
      run: go(`/clients/${client.id}/edit`),
    }
  }),
)

function normalise(text: string): string {
  return text.toLocaleLowerCase('it-IT').normalize('NFD').replace(/\p{Diacritic}/gu, '')
}

const results = computed<PaletteItem[]>(() => {
  const needle = normalise(query.value.trim())
  const matches = (item: PaletteItem): boolean =>
    normalise(`${item.label} ${item.detail ?? ''} ${item.keywords ?? ''}`).includes(needle)
  if (needle === '') return staticItems.value
  const actions = staticItems.value.filter(matches)
  const patients = patientItems.value.filter(matches).slice(0, 8)
  return [...patients, ...actions]
})

const grouped = computed(() => {
  const groups: { name: Group; items: { item: PaletteItem; index: number }[] }[] = []
  results.value.forEach((item, index) => {
    let group = groups.find((candidate) => candidate.name === item.group)
    if (group === undefined) {
      group = { name: item.group, items: [] }
      groups.push(group)
    }
    group.items.push({ item, index })
  })
  return groups
})

watch(results, () => { activeIndex.value = 0 })

watch(
  () => props.open,
  (isOpen) => {
    if (!isOpen) return
    query.value = ''
    activeIndex.value = 0
    if (clientsStore.clients.length === 0) void clientsStore.fetchClients()
  },
)

useFocusTrap(toRef(props, 'open'), panelRef, { onEscape: () => emit('close') })

function run(item: PaletteItem): void {
  emit('close')
  item.run()
}

function scrollActive(): void {
  listRef.value?.querySelector<HTMLElement>(`[data-index="${activeIndex.value}"]`)?.scrollIntoView({ block: 'nearest' })
}

function onKeydown(event: KeyboardEvent): void {
  const count = results.value.length
  if (count === 0) return
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    activeIndex.value = (activeIndex.value + 1) % count
    void nextTick(scrollActive)
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    activeIndex.value = (activeIndex.value - 1 + count) % count
    void nextTick(scrollActive)
  } else if (event.key === 'Enter') {
    event.preventDefault()
    const item = results.value[activeIndex.value]
    if (item) run(item)
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="dialog">
      <div v-if="open" class="fixed inset-0 z-(--z-modal) flex justify-center px-6 pt-[14vh]">
        <div class="absolute inset-0 bg-(--backdrop)" aria-hidden="true" @click="emit('close')" />
        <div
          ref="panelRef"
          role="dialog"
          aria-modal="true"
          aria-label="Cerca o vai a"
          data-lenis-prevent
          class="dialog-panel relative flex max-h-[min(30rem,70vh)] w-full max-w-[36rem] flex-col overflow-hidden rounded-card border border-border bg-surface-raised shadow-modal"
        >
          <div class="flex items-center gap-3 border-b border-border px-4">
            <Search :size="17" class="shrink-0 text-text-subtle" aria-hidden="true" />
            <input
              v-model="query"
              type="text"
              class="h-13 w-full bg-transparent text-md text-text outline-none placeholder:text-text-subtle"
              placeholder="Cerca un paziente, un'azione, una sezione…"
              role="combobox"
              aria-expanded="true"
              aria-controls="palette-list"
              :aria-activedescendant="results[activeIndex] ? `palette-${results[activeIndex].id}` : undefined"
              @keydown="onKeydown"
            />
          </div>

          <div id="palette-list" ref="listRef" role="listbox" class="flex-1 overflow-y-auto p-2">
            <div v-for="group in grouped" :key="group.name" class="mb-1 last:mb-0" role="group" :aria-label="group.name">
              <p class="px-2.5 pt-2 pb-1.5 text-xs font-medium text-text-subtle">{{ group.name }}</p>
              <div
                v-for="{ item, index } in group.items"
                :id="`palette-${item.id}`"
                :key="item.id"
                role="option"
                :data-index="index"
                :aria-selected="index === activeIndex"
                class="flex h-10 cursor-pointer items-center gap-3 rounded-control px-2.5 text-base"
                :class="index === activeIndex ? 'bg-surface-hover text-text' : 'text-text-muted'"
                @mousemove="activeIndex = index"
                @click="run(item)"
              >
                <PatientMonogram v-if="item.patientName" :name="item.patientName" size="sm" />
                <component :is="item.icon" v-else :size="16" :stroke-width="1.75" class="shrink-0" :class="index === activeIndex ? 'text-accent' : 'text-text-subtle'" aria-hidden="true" />
                <span class="min-w-0 flex-1 truncate">{{ item.label }}</span>
                <span v-if="item.detail" class="font-mono text-xs text-text-subtle">{{ item.detail }}</span>
                <CornerDownLeft v-if="index === activeIndex" :size="14" class="shrink-0 text-text-subtle" aria-hidden="true" />
              </div>
            </div>
            <p v-if="results.length === 0" class="px-3 py-10 text-center text-sm text-text-subtle">
              Nessun risultato per “{{ query }}”
            </p>
          </div>

          <footer class="flex items-center gap-4 border-t border-border bg-surface px-4 py-2 text-2xs text-text-subtle">
            <span><kbd class="font-sans">↑↓</kbd> per muoverti</span>
            <span><kbd class="font-sans">↵</kbd> per aprire</span>
            <span><kbd class="font-sans">esc</kbd> per chiudere</span>
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

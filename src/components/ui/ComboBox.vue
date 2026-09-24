<script setup lang="ts">
/**
 * A searchable select for long lists (fifty patients do not fit a native
 * <select> gracefully). Type to filter by label or detail; arrows move,
 * Enter picks, Escape closes. It keeps the recessed `.field` look so it sits
 * in a form like any other input.
 */
import { computed, nextTick, ref, useId, watch } from 'vue'
import { Check, ChevronsUpDown, Search } from 'lucide-vue-next'
import type { ComboOption } from './types'

const props = withDefaults(
  defineProps<{
    options: readonly ComboOption[]
    placeholder?: string
    id?: string
    invalid?: boolean
    describedBy?: string
    emptyText?: string
  }>(),
  { placeholder: 'Seleziona…', emptyText: 'Nessun risultato' },
)

const model = defineModel<number | null>({ required: true })

const listId = useId()
const open = ref(false)
const query = ref('')
const activeIndex = ref(0)
const rootRef = ref<HTMLElement | null>(null)
const searchRef = ref<HTMLInputElement | null>(null)
const listRef = ref<HTMLElement | null>(null)

const selected = computed(() => props.options.find((option) => option.value === model.value) ?? null)

function normalise(text: string): string {
  return text.toLocaleLowerCase('it-IT').normalize('NFD').replace(/\p{Diacritic}/gu, '')
}

const filtered = computed(() => {
  const needle = normalise(query.value.trim())
  if (needle === '') return props.options
  return props.options.filter((option) =>
    normalise(`${option.label} ${option.detail ?? ''}`).includes(needle),
  )
})

watch(filtered, () => { activeIndex.value = 0 })

function openList(): void {
  open.value = true
  query.value = ''
  const current = props.options.findIndex((option) => option.value === model.value)
  activeIndex.value = Math.max(0, current)
  void nextTick(() => {
    searchRef.value?.focus()
    scrollActiveIntoView()
  })
}

function close(): void {
  open.value = false
}

function pick(option: ComboOption): void {
  model.value = option.value
  close()
}

function scrollActiveIntoView(): void {
  const element = listRef.value?.querySelector<HTMLElement>(`[data-index="${activeIndex.value}"]`)
  element?.scrollIntoView({ block: 'nearest' })
}

function onSearchKeydown(event: KeyboardEvent): void {
  const count = filtered.value.length
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    activeIndex.value = count === 0 ? 0 : (activeIndex.value + 1) % count
    void nextTick(scrollActiveIntoView)
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    activeIndex.value = count === 0 ? 0 : (activeIndex.value - 1 + count) % count
    void nextTick(scrollActiveIntoView)
  } else if (event.key === 'Enter') {
    event.preventDefault()
    const option = filtered.value[activeIndex.value]
    if (option) pick(option)
  } else if (event.key === 'Escape') {
    event.preventDefault()
    event.stopPropagation()
    close()
  } else if (event.key === 'Tab') {
    close()
  }
}

function onFocusOut(event: FocusEvent): void {
  const next = event.relatedTarget
  if (next instanceof Node && rootRef.value?.contains(next)) return
  close()
}
</script>

<template>
  <div ref="rootRef" class="relative" @focusout="onFocusOut">
    <button
      :id="id"
      type="button"
      class="field flex items-center gap-2 text-left"
      :aria-invalid="invalid || undefined"
      :aria-describedby="describedBy"
      aria-haspopup="listbox"
      :aria-expanded="open"
      :aria-controls="listId"
      @click="open ? close() : openList()"
      @keydown.down.prevent="openList"
    >
      <span class="min-w-0 flex-1 truncate" :class="selected ? 'text-text' : 'text-text-subtle'">
        {{ selected?.label ?? placeholder }}
      </span>
      <span v-if="selected?.detail" class="font-mono text-xs text-text-subtle">{{ selected.detail }}</span>
      <ChevronsUpDown :size="15" class="shrink-0 text-text-subtle" aria-hidden="true" />
    </button>

    <Transition name="popover">
      <div
        v-if="open"
        class="absolute left-0 right-0 top-[calc(100%+6px)] z-(--z-overlay) overflow-hidden rounded-card border border-border bg-surface-raised shadow-modal"
        data-lenis-prevent
      >
        <div class="flex items-center gap-2 border-b border-border px-3">
          <Search :size="15" class="shrink-0 text-text-subtle" aria-hidden="true" />
          <input
            ref="searchRef"
            v-model="query"
            type="text"
            class="h-10 w-full bg-transparent text-base text-text outline-none placeholder:text-text-subtle"
            placeholder="Cerca…"
            role="combobox"
            aria-autocomplete="list"
            :aria-controls="listId"
            :aria-activedescendant="filtered[activeIndex] ? `${listId}-${activeIndex}` : undefined"
            @keydown="onSearchKeydown"
          />
        </div>
        <ul :id="listId" ref="listRef" role="listbox" class="max-h-64 overflow-y-auto p-1">
          <li
            v-for="(option, index) in filtered"
            :id="`${listId}-${index}`"
            :key="option.value"
            role="option"
            :data-index="index"
            :aria-selected="option.value === model"
            class="flex cursor-pointer items-center gap-2 rounded-[7px] px-2.5 py-2 text-base"
            :class="index === activeIndex ? 'bg-surface-hover text-text' : 'text-text-muted'"
            @mousemove="activeIndex = index"
            @mousedown.prevent="pick(option)"
          >
            <Check :size="14" :stroke-width="2" class="shrink-0" :class="option.value === model ? 'text-accent' : 'invisible'" aria-hidden="true" />
            <span class="min-w-0 flex-1 truncate" :class="option.value === model ? 'text-text font-medium' : ''">{{ option.label }}</span>
            <span v-if="option.detail" class="font-mono text-xs text-text-subtle">{{ option.detail }}</span>
          </li>
          <li v-if="filtered.length === 0" class="px-3 py-6 text-center text-sm text-text-subtle">{{ emptyText }}</li>
        </ul>
      </div>
    </Transition>
  </div>
</template>

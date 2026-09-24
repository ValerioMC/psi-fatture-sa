<script setup lang="ts" generic="T extends string | number | boolean">
/**
 * A choice among a few mutually exclusive options, shown all at once.
 *
 * The selected option sits on a raised thumb that slides to it rather than
 * jumping, so the eye follows the change. Arrow keys move the choice, as in a
 * native radio group.
 */
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { Tone } from '@/utils/labels'
import type { SegmentOption } from './types'

const props = withDefaults(
  defineProps<{
    options: readonly SegmentOption<T>[]
    label: string
    size?: 'sm' | 'md'
    block?: boolean
  }>(),
  { size: 'md', block: false },
)

const model = defineModel<T>({ required: true })

const trackRef = ref<HTMLElement | null>(null)
const buttonRefs = ref<HTMLButtonElement[]>([])
const thumb = ref<{ left: number; width: number } | null>(null)

const DOT_CLASS: Record<Tone, string> = {
  neutral: 'bg-text-subtle',
  accent: 'bg-accent',
  safe: 'bg-safe',
  warn: 'bg-warn',
  danger: 'bg-danger',
}

function measure(): void {
  const index = props.options.findIndex((option) => option.value === model.value)
  const button = buttonRefs.value[index]
  thumb.value = button ? { left: button.offsetLeft, width: button.offsetWidth } : null
}

function select(index: number): void {
  const option = props.options[index]
  if (option === undefined) return
  model.value = option.value
  void nextTick(() => buttonRefs.value[index]?.focus())
}

function onKeydown(event: KeyboardEvent, index: number): void {
  const last = props.options.length - 1
  if (event.key === 'ArrowRight' || event.key === 'ArrowDown') {
    event.preventDefault()
    select(index === last ? 0 : index + 1)
  } else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') {
    event.preventDefault()
    select(index === 0 ? last : index - 1)
  }
}

let observer: ResizeObserver | null = null
onMounted(() => {
  measure()
  observer = new ResizeObserver(measure)
  if (trackRef.value) observer.observe(trackRef.value)
})
onBeforeUnmount(() => observer?.disconnect())
watch([model, () => props.options], () => nextTick(measure), { deep: true })
</script>

<template>
  <div
    ref="trackRef"
    role="radiogroup"
    :aria-label="label"
    class="relative inline-flex items-stretch rounded-control bg-surface-sunken p-0.5 border border-border"
    :class="[block ? 'flex w-full' : '', size === 'sm' ? 'h-control-sm' : 'h-control']"
  >
    <span
      v-if="thumb"
      class="absolute top-0.5 bottom-0.5 rounded-[7px] bg-surface-raised shadow-[0_1px_2px_rgb(40_34_20/0.12),0_0_0_1px_var(--border)] transition-[left,width] duration-200 ease-out-expo"
      :style="{ left: `${thumb.left}px`, width: `${thumb.width}px` }"
      aria-hidden="true"
    />
    <button
      v-for="(option, index) in options"
      :key="String(option.value)"
      ref="buttonRefs"
      type="button"
      role="radio"
      :aria-checked="option.value === model"
      :tabindex="option.value === model ? 0 : -1"
      class="relative z-[1] inline-flex items-center justify-center gap-1.5 rounded-[7px] font-medium whitespace-nowrap transition-colors duration-150 focus-ring"
      :class="[
        size === 'sm' ? 'px-2.5 text-sm' : 'px-3 text-base',
        block ? 'flex-1' : '',
        option.value === model ? 'text-text' : 'text-text-muted hover:text-text',
      ]"
      @click="select(index)"
      @keydown="onKeydown($event, index)"
    >
      <span v-if="option.tone" class="size-1.5 rounded-full" :class="DOT_CLASS[option.tone]" aria-hidden="true" />
      {{ option.label }}
      <span v-if="option.count !== undefined" class="tabular text-2xs text-text-subtle">{{ option.count }}</span>
    </button>
  </div>
</template>

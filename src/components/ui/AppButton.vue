<script setup lang="ts">
/**
 * The only button in the app. A closed set of variants:
 *
 * - primary: the one action a screen exists for, filled with ink.
 * - secondary: a real action that is not the point of the screen.
 * - ghost: navigation-weight actions and toolbar icons.
 * - danger: the confirm button inside a destructive dialog, nowhere else.
 * - danger-quiet: a destructive action repeated down a list. Quiet at rest and
 *   red only on hover, so a column of rows is not a column of alarms; the
 *   confirm dialog is the real guard.
 *
 * Passing `to` renders a RouterLink with the same look. Without a default slot
 * the button is icon-only and `label` becomes its accessible name and tooltip.
 */
import { computed, useSlots, type Component } from 'vue'
import { RouterLink, type RouteLocationRaw } from 'vue-router'
import AppSpinner from './AppSpinner.vue'

type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'danger-quiet'
type ButtonSize = 'sm' | 'md'

const props = withDefaults(
  defineProps<{
    variant?: ButtonVariant
    size?: ButtonSize
    type?: 'button' | 'submit'
    to?: RouteLocationRaw
    icon?: Component
    iconRight?: Component
    label?: string
    loading?: boolean
    disabled?: boolean
    block?: boolean
    shortcut?: string
  }>(),
  { variant: 'secondary', size: 'md', type: 'button', loading: false, disabled: false, block: false },
)

const slots = useSlots()
const iconOnly = computed(() => slots.default === undefined)

const VARIANT_CLASS: Record<ButtonVariant, string> = {
  primary:
    'bg-accent text-accent-ink hover:bg-accent-strong shadow-[inset_0_1px_0_rgb(255_255_255/0.16),0_1px_2px_rgb(20_18_60/0.18)]',
  secondary:
    'bg-surface-raised text-text border border-border-strong hover:bg-surface-hover shadow-[0_1px_1px_rgb(40_34_20/0.04)]',
  ghost: 'text-text-muted hover:text-text hover:bg-surface-hover',
  danger: 'bg-danger text-white hover:brightness-110',
  'danger-quiet': 'text-text-subtle hover:text-danger hover:bg-danger-soft',
}

const SIZE_CLASS: Record<ButtonSize, { text: string; icon: string; iconSize: number }> = {
  md: { text: 'h-control px-3.5 text-base gap-2', icon: 'size-control', iconSize: 16 },
  sm: { text: 'h-control-sm px-2.5 text-sm gap-1.5', icon: 'size-control-sm', iconSize: 15 },
}

const classes = computed(() => [
  'relative inline-flex items-center justify-center rounded-control font-medium whitespace-nowrap select-none',
  'transition-[background-color,border-color,color,box-shadow,filter] duration-150 active:translate-y-px',
  'focus-ring disabled:opacity-50 aria-disabled:opacity-50 aria-disabled:pointer-events-none',
  VARIANT_CLASS[props.variant],
  iconOnly.value ? SIZE_CLASS[props.size].icon : SIZE_CLASS[props.size].text,
  props.block ? 'w-full' : '',
])

const isInactive = computed(() => props.disabled || props.loading)
</script>

<template>
  <RouterLink
    v-if="to !== undefined"
    :to="to"
    :class="classes"
    :aria-label="iconOnly ? label : undefined"
    :title="iconOnly ? label : undefined"
    :aria-disabled="isInactive || undefined"
  >
    <component :is="icon" v-if="icon" :size="SIZE_CLASS[size].iconSize" :stroke-width="1.75" aria-hidden="true" />
    <slot />
    <component :is="iconRight" v-if="iconRight" :size="SIZE_CLASS[size].iconSize" :stroke-width="1.75" aria-hidden="true" />
    <kbd v-if="shortcut" class="ml-1 font-sans text-2xs opacity-70">{{ shortcut }}</kbd>
  </RouterLink>
  <button
    v-else
    :type="type"
    :class="classes"
    :disabled="isInactive"
    :aria-label="iconOnly ? label : undefined"
    :title="iconOnly ? label : undefined"
    :aria-busy="loading || undefined"
  >
    <!-- The spinner overlays the content, which stays in place (invisible) so the button never changes size. -->
    <span v-if="loading" class="absolute inset-0 grid place-items-center">
      <AppSpinner :size="SIZE_CLASS[size].iconSize" />
    </span>
    <span class="contents" :class="loading ? '[&>*]:invisible' : ''">
      <component :is="icon" v-if="icon" :size="SIZE_CLASS[size].iconSize" :stroke-width="1.75" aria-hidden="true" />
      <span v-if="!iconOnly" :class="loading ? 'invisible' : ''"><slot /></span>
      <component :is="iconRight" v-if="iconRight" :size="SIZE_CLASS[size].iconSize" :stroke-width="1.75" aria-hidden="true" />
      <kbd v-if="shortcut" class="ml-1 font-sans text-2xs opacity-70">{{ shortcut }}</kbd>
    </span>
  </button>
</template>

<script setup lang="ts">
/**
 * An on/off setting with its explanation. The whole row is the hit target;
 * the switch itself is the visual, `role="switch"` carries the semantics.
 */
import { useId } from 'vue'

withDefaults(
  defineProps<{
    label: string
    description?: string
    /** Only the switch is shown; the label is kept for screen readers. Used inside table rows. */
    compact?: boolean
    disabled?: boolean
  }>(),
  { compact: false, disabled: false },
)

const model = defineModel<boolean>({ required: true })
const id = useId()
</script>

<template>
  <div class="flex items-start gap-3" :class="compact ? 'inline-flex' : ''">
    <button
      :id="id"
      type="button"
      role="switch"
      :aria-checked="model"
      :aria-describedby="description ? `${id}-desc` : undefined"
      :aria-label="compact ? label : undefined"
      :disabled="disabled"
      class="mt-0.5 relative inline-flex h-5 w-9 shrink-0 items-center rounded-full border transition-colors duration-200 focus-ring disabled:opacity-50"
      :class="model ? 'bg-accent border-accent' : 'bg-surface-sunken border-border-strong'"
      @click="model = !model"
    >
      <span
        class="inline-block size-3.5 rounded-full bg-white shadow-[0_1px_2px_rgb(0_0_0/0.25)] transition-transform duration-200 ease-out-expo"
        :class="model ? 'translate-x-[17px]' : 'translate-x-[2px]'"
      />
    </button>
    <label v-if="!compact" :for="id" class="min-w-0 cursor-pointer select-none">
      <span class="block text-base font-medium text-text">{{ label }}</span>
      <span v-if="description" :id="`${id}-desc`" class="block text-sm text-text-muted mt-0.5">{{ description }}</span>
      <slot />
    </label>
  </div>
</template>

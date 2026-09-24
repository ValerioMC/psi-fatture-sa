<script setup lang="ts">
/**
 * The one modal surface. Traps focus, closes on Escape and on backdrop click,
 * returns focus to its trigger, and uses the shared "dialog" motion.
 */
import { ref, toRef, useId } from 'vue'
import { X } from 'lucide-vue-next'
import { useFocusTrap } from '@/composables/useFocusTrap'
import AppButton from './AppButton.vue'

const props = withDefaults(
  defineProps<{
    open: boolean
    title: string
    description?: string
    size?: 'sm' | 'md' | 'lg'
    role?: 'dialog' | 'alertdialog'
    dismissible?: boolean
  }>(),
  { size: 'md', role: 'dialog', dismissible: true },
)

const emit = defineEmits<{ close: [] }>()

const panelRef = ref<HTMLElement | null>(null)
const titleId = useId()

function requestClose(): void {
  if (props.dismissible) emit('close')
}

/* Content marks its preferred first focus with `data-autofocus`; otherwise the first focusable wins. */
useFocusTrap(toRef(props, 'open'), panelRef, {
  onEscape: requestClose,
  initialFocus: () => panelRef.value?.querySelector<HTMLElement>('[data-autofocus]'),
})

const WIDTH: Record<'sm' | 'md' | 'lg', string> = {
  sm: 'max-w-[26rem]',
  md: 'max-w-[34rem]',
  lg: 'max-w-[44rem]',
}
</script>

<template>
  <Teleport to="body">
    <Transition name="dialog">
      <div v-if="open" class="fixed inset-0 z-(--z-modal) flex items-center justify-center p-6">
        <div class="absolute inset-0 bg-(--backdrop) backdrop-blur-[2px]" aria-hidden="true" @click="requestClose" />
        <div
          ref="panelRef"
          :role="role"
          aria-modal="true"
          :aria-labelledby="titleId"
          tabindex="-1"
          data-lenis-prevent
          class="dialog-panel relative w-full max-h-[calc(100vh-3rem)] overflow-y-auto rounded-card border border-border bg-surface-raised shadow-modal outline-none"
          :class="WIDTH[size]"
        >
          <header class="flex items-start gap-3 px-6 pt-5 pb-1">
            <div class="min-w-0 flex-1">
              <h2 :id="titleId" class="display text-xl text-text">{{ title }}</h2>
              <p v-if="description" class="mt-1 text-sm text-text-muted">{{ description }}</p>
            </div>
            <AppButton v-if="dismissible" variant="ghost" size="sm" :icon="X" label="Chiudi" class="-mr-2 -mt-0.5" @click="requestClose" />
          </header>
          <div class="px-6 pt-4 pb-6">
            <slot />
          </div>
          <footer v-if="$slots.footer" class="flex items-center gap-2 px-6 py-4 border-t border-border bg-surface rounded-b-card">
            <slot name="footer" />
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

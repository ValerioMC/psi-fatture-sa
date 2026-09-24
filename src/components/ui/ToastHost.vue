<script setup lang="ts">
/**
 * Where toasts appear: bottom-right, stacked, announced politely. Successes
 * leave on their own; errors stay longer and can be dismissed by hand.
 */
import { CircleCheck, CircleAlert, Info, X } from 'lucide-vue-next'
import { useToastStore, type ToastKind } from '@/stores/toast'

const toastStore = useToastStore()

const ICON = { success: CircleCheck, error: CircleAlert, info: Info } as const
const ICON_CLASS: Record<ToastKind, string> = {
  success: 'text-safe',
  error: 'text-danger',
  info: 'text-accent',
}
</script>

<template>
  <div
    class="pointer-events-none fixed bottom-5 right-5 z-(--z-toast) flex w-[22rem] flex-col items-end gap-2"
    role="region"
    aria-label="Notifiche"
  >
    <TransitionGroup name="rise" tag="div" class="flex w-full flex-col items-end gap-2" aria-live="polite">
      <div
        v-for="toast in toastStore.toasts"
        :key="toast.id"
        :role="toast.kind === 'error' ? 'alert' : 'status'"
        class="pointer-events-auto flex w-full items-start gap-3 rounded-card border border-border bg-surface-raised px-4 py-3 shadow-modal"
      >
        <component :is="ICON[toast.kind]" :size="18" :stroke-width="1.8" class="mt-px shrink-0" :class="ICON_CLASS[toast.kind]" aria-hidden="true" />
        <p class="min-w-0 flex-1 text-base text-text">{{ toast.message }}</p>
        <button
          v-if="toast.action"
          type="button"
          class="shrink-0 rounded-[6px] px-1.5 text-base font-medium text-accent hover:underline underline-offset-2 focus-ring"
          @click="toastStore.runAction(toast)"
        >
          {{ toast.action.label }}
        </button>
        <button
          type="button"
          class="-mr-1 shrink-0 rounded-[6px] p-0.5 text-text-subtle hover:text-text focus-ring"
          aria-label="Chiudi notifica"
          @click="toastStore.dismiss(toast.id)"
        >
          <X :size="15" aria-hidden="true" />
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

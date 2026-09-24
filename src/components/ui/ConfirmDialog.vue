<script setup lang="ts">
/**
 * Confirmation before anything irreversible or wide-reaching.
 *
 * `blastRadius` is one concrete sentence on what will actually change ("La
 * fattura 12/2026 di Rossi Maria, 102,00 €, verrà eliminata."), set apart on a
 * hatched strip. "Sei sicuro?" is not a blast radius. Focus lands on the safe
 * button, so Enter never destroys by accident.
 */
import AppButton from './AppButton.vue'
import AppDialog from './AppDialog.vue'

withDefaults(
  defineProps<{
    open: boolean
    title: string
    message?: string
    blastRadius?: string
    confirmLabel?: string
    tone?: 'danger' | 'accent'
    loading?: boolean
  }>(),
  { confirmLabel: 'Elimina', tone: 'danger', loading: false },
)

const emit = defineEmits<{ confirm: []; cancel: [] }>()
</script>

<template>
  <AppDialog
    :open="open"
    :title="title"
    size="sm"
    role="alertdialog"
    @close="emit('cancel')"
  >
    <p v-if="message" class="text-base text-text-muted">{{ message }}</p>
    <p
      v-if="blastRadius"
      class="blast-radius mt-4 rounded-control px-3.5 py-3 text-sm ring-1 ring-inset"
      :class="tone === 'danger' ? 'text-danger ring-danger-line' : 'text-accent ring-accent-line'"
    >
      {{ blastRadius }}
    </p>
    <template #footer>
      <div class="flex-1" />
      <AppButton variant="ghost" data-autofocus @click="emit('cancel')">Annulla</AppButton>
      <AppButton :variant="tone === 'danger' ? 'danger' : 'primary'" :loading="loading" @click="emit('confirm')">
        {{ confirmLabel }}
      </AppButton>
    </template>
  </AppDialog>
</template>

<style scoped>
/* Hazard hatching at a whisper: the strip reads as "this is the consequence" without shouting. */
.blast-radius {
  background-color: color-mix(in srgb, currentColor 6%, transparent);
  background-image: repeating-linear-gradient(
    -45deg,
    color-mix(in srgb, currentColor 7%, transparent) 0 6px,
    transparent 6px 12px
  );
}
</style>

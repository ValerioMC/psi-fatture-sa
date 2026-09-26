<script setup lang="ts">
/**
 * One secret kept in the OS keychain: whether it is there, and a field to
 * set or replace it. The value is never read back; replacing means typing
 * a new one, and removing asks first.
 */
import { ref } from 'vue'
import { Eye, EyeOff } from 'lucide-vue-next'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import FormField from '@/components/ui/FormField.vue'
import { errorMessage } from '@/stores/toast'

const props = defineProps<{
  label: string
  hint: string
  configured: boolean
  /** Rejects whitespace, as a PINCODE must. */
  noSpaces?: boolean
  save: (value: string) => Promise<void>
}>()

const emit = defineEmits<{ remove: [] }>()

const editing = ref(false)
const value = ref('')
const revealed = ref(false)
const error = ref('')
const saving = ref(false)

function validate(): boolean {
  if (value.value.trim() === '') error.value = `Inserisci ${props.label.toLowerCase()}`
  else if (props.noSpaces && /\s/.test(value.value.trim())) error.value = `${props.label} senza spazi`
  else error.value = ''
  return error.value === ''
}

async function submit(): Promise<void> {
  if (!validate()) return
  saving.value = true
  try {
    await props.save(value.value)
    close()
  } catch (failure) {
    error.value = errorMessage(failure)
  } finally {
    saving.value = false
  }
}

function close(): void {
  editing.value = false
  value.value = ''
  revealed.value = false
  error.value = ''
}
</script>

<template>
  <div class="rounded-control border border-border bg-surface px-3 py-2.5">
    <div class="flex items-center justify-between gap-2">
      <span class="text-sm font-medium text-text">{{ label }}</span>
      <div class="flex items-center gap-1">
        <AppBadge v-if="configured" tone="safe" dot>Nel portachiavi</AppBadge>
        <AppBadge v-else tone="warn" dot>Da inserire</AppBadge>
      </div>
    </div>

    <form v-if="editing || !configured" class="mt-2.5 space-y-2" novalidate @submit.prevent="submit">
      <FormField v-slot="{ id, invalid, describedBy }" :label="`${label} Sistema TS`" :error="error" :hint="hint">
        <div class="relative">
          <input
            :id="id"
            v-model="value"
            :type="revealed ? 'text' : 'password'"
            class="field pr-10 font-mono"
            autocomplete="off"
            spellcheck="false"
            :aria-invalid="invalid"
            :aria-describedby="describedBy"
            @input="error = ''"
          />
          <button
            type="button"
            class="absolute inset-y-0 right-0 flex w-9 items-center justify-center rounded-control text-text-subtle hover:text-text focus-ring"
            :aria-label="revealed ? `Nascondi ${label}` : `Mostra ${label}`"
            :aria-pressed="revealed"
            @click="revealed = !revealed"
          >
            <component :is="revealed ? EyeOff : Eye" :size="15" :stroke-width="1.8" aria-hidden="true" />
          </button>
        </div>
      </FormField>
      <div class="flex justify-end gap-2">
        <AppButton v-if="configured" variant="ghost" size="sm" @click="close">Annulla</AppButton>
        <AppButton variant="primary" size="sm" type="submit" :loading="saving">Salva</AppButton>
      </div>
    </form>

    <div v-else class="mt-1.5 flex justify-end gap-1">
      <AppButton variant="ghost" size="sm" @click="editing = true">Sostituisci</AppButton>
      <AppButton variant="danger-quiet" size="sm" @click="emit('remove')">Rimuovi</AppButton>
    </div>
  </div>
</template>

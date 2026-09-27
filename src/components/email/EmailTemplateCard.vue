<script setup lang="ts">
/**
 * The text every invoice email starts from. Placeholders go in with a click at the
 * caret of the field last touched; the preview beside it is the backend's own
 * rendering with an example patient, so what is shown is what will be sent.
 */
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { FileText, PenLine, Paperclip, RotateCcw } from 'lucide-vue-next'
import { getEmailPlaceholders, getEmailTemplate, previewEmailTemplate, resetEmailTemplate, updateEmailTemplate } from '@/api'
import { useEmailStore } from '@/stores/email'
import { errorMessage, useToastStore } from '@/stores/toast'
import type { EmailPlaceholderInfo, EmailPreview, EmailTemplate } from '@/types'
import ActionButton from '@/components/ui/ActionButton.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import CardHeader from '@/components/ui/CardHeader.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import FormField from '@/components/ui/FormField.vue'
import { insertPlaceholder } from '@/utils/email'

const PREVIEW_DELAY_MS = 250

const email = useEmailStore()
const toast = useToastStore()

const form = reactive<EmailTemplate>({ subject: '', body: '' })
const saved = ref<EmailTemplate | null>(null)
const placeholders = ref<EmailPlaceholderInfo[]>([])
const preview = ref<EmailPreview | null>(null)
const error = ref('')
const resetOpen = ref(false)
const resetting = ref(false)
const subjectRef = ref<HTMLInputElement | null>(null)
const bodyRef = ref<HTMLTextAreaElement | null>(null)
const lastField = ref<'subject' | 'body'>('body')

onMounted(async () => {
  try {
    const [template, available] = await Promise.all([getEmailTemplate(), getEmailPlaceholders()])
    placeholders.value = available
    adopt(template)
  } catch (failure) {
    toast.notifyError(failure, 'Modello non leggibile')
  }
})

function adopt(template: EmailTemplate): void {
  Object.assign(form, template)
  saved.value = { ...template }
}

const dirty = computed(() => saved.value !== null && (form.subject.trim() !== saved.value.subject || form.body.trim() !== saved.value.body))

/** Names between braces that are not placeholders, flagged before the backend refuses them. */
const unknown = computed(() => {
  const known = new Set(placeholders.value.map((p) => p.key))
  const names = [...`${form.subject}\n${form.body}`.matchAll(/\{\s*([^{}\s]+)\s*\}/g)].map((match) => match[1]!)
  return [...new Set(names.filter((name) => !known.has(name)))]
})

const unknownList = computed(() => unknown.value.map((name) => `{${name}}`).join(', '))

let timer: ReturnType<typeof setTimeout> | null = null
watch(
  form,
  () => {
    error.value = ''
    if (timer !== null) clearTimeout(timer)
    timer = setTimeout(refreshPreview, PREVIEW_DELAY_MS)
  },
  { deep: true },
)
onBeforeUnmount(() => {
  if (timer !== null) clearTimeout(timer)
})

async function refreshPreview(): Promise<void> {
  try {
    preview.value = await previewEmailTemplate({ ...form })
  } catch (failure) {
    toast.notifyError(failure, 'Anteprima non disponibile')
  }
}

async function insert(key: string): Promise<void> {
  const field = lastField.value === 'subject' ? subjectRef.value : bodyRef.value
  if (field === null) return
  const start = field.selectionStart ?? form[lastField.value].length
  const end = field.selectionEnd ?? start
  const next = insertPlaceholder(form[lastField.value], start, end, key)
  form[lastField.value] = next.text
  await nextTick()
  field.focus()
  field.setSelectionRange(next.caret, next.caret)
}

async function save(): Promise<boolean> {
  try {
    adopt(await updateEmailTemplate({ ...form }))
    return true
  } catch (failure) {
    error.value = errorMessage(failure)
    return false
  }
}

async function confirmReset(): Promise<void> {
  resetting.value = true
  try {
    adopt(await resetEmailTemplate())
    toast.notify('Modello predefinito ripristinato')
    resetOpen.value = false
  } catch (failure) {
    toast.notifyError(failure, 'Non ripristinato')
  } finally {
    resetting.value = false
  }
}

const sender = computed(() => {
  const account = email.account
  if (!account || account.sender_address === '') return 'La tua casella'
  return account.sender_name ? `${account.sender_name} <${account.sender_address}>` : account.sender_address
})
</script>

<template>
  <AppCard :padded="false">
    <CardHeader title="Modello dell’email" subtitle="Il testo proposto per ogni fattura, che puoi ritoccare prima di inviarla" :icon="PenLine">
      <AppButton variant="ghost" size="sm" :icon="RotateCcw" @click="resetOpen = true">Ripristina</AppButton>
    </CardHeader>

    <div class="grid grid-cols-1 gap-6 px-5 pb-5 xl:grid-cols-2">
      <!-- The editor. -->
      <div class="min-w-0 space-y-4">
        <FormField v-slot="{ id, invalid, describedBy }" label="Oggetto" :error="error || undefined">
          <input
            :id="id"
            ref="subjectRef"
            v-model="form.subject"
            type="text"
            class="field"
            maxlength="200"
            :aria-invalid="invalid"
            :aria-describedby="describedBy"
            @focus="lastField = 'subject'"
          />
        </FormField>
        <FormField v-slot="{ id }" label="Testo">
          <textarea
            :id="id"
            ref="bodyRef"
            v-model="form.body"
            rows="12"
            class="field leading-relaxed"
            @focus="lastField = 'body'"
          />
        </FormField>

        <div>
          <p class="mb-2 text-xs text-text-subtle">
            Inserisci nel campo {{ lastField === 'subject' ? 'Oggetto' : 'Testo' }}: ogni segnaposto diventa il dato della fattura.
          </p>
          <div class="flex flex-wrap gap-1.5">
            <button
              v-for="placeholder in placeholders"
              :key="placeholder.key"
              type="button"
              class="placeholder-chip inline-flex h-7 items-center gap-1.5 rounded-full px-2.5 text-xs text-text-muted transition-colors hover:text-text focus-ring"
              :title="`Inserisce {${placeholder.key}}`"
              @mousedown.prevent
              @click="insert(placeholder.key)"
            >
              <span class="font-mono text-2xs text-accent">{}</span>
              {{ placeholder.label }}
            </button>
          </div>
          <p v-if="unknown.length > 0" class="mt-2 text-xs text-danger" role="alert">
            Non riconosciuti: {{ unknownList }}. Usa i segnaposto qui sopra.
          </p>
        </div>

        <div v-if="dirty" class="flex items-center justify-end gap-3 pt-1">
          <AppButton variant="ghost" size="sm" @click="saved && adopt(saved)">Annulla</AppButton>
          <ActionButton :run="save" size="sm" :disabled="unknown.length > 0" done-label="Salvato" working-label="Salvataggio del modello">
            Salva modello
          </ActionButton>
        </div>
      </div>

      <!-- The preview: an email as the patient will open it. -->
      <div class="min-w-0">
        <p class="mb-1.5 text-sm font-medium text-text-muted">Anteprima con un paziente di esempio</p>
        <article class="mail-sheet overflow-hidden rounded-card" aria-label="Anteprima dell’email">
          <dl class="grid grid-cols-[3.5rem_minmax(0,1fr)] gap-x-2 gap-y-1 border-b border-border px-5 py-3.5 text-xs">
            <dt class="text-text-subtle">Da</dt>
            <dd class="truncate text-text-muted">{{ sender }}</dd>
            <dt class="text-text-subtle">A</dt>
            <dd class="truncate text-text-muted">Anna Bianchi &lt;anna.bianchi@email.it&gt;</dd>
          </dl>
          <div class="px-5 pt-4 pb-5">
            <Transition name="swap" mode="out-in">
              <div :key="preview ? 'ready' : 'loading'">
                <template v-if="preview">
                  <h3 class="text-md font-semibold text-text">{{ preview.subject }}</h3>
                  <p class="mt-3 whitespace-pre-wrap text-base leading-relaxed text-text">{{ preview.body }}</p>
                </template>
                <div v-else class="space-y-2" role="status" aria-busy="true">
                  <span class="sr-only">Preparazione dell’anteprima</span>
                  <div class="skeleton h-4 w-2/3" />
                  <div class="skeleton h-3 w-full" />
                  <div class="skeleton h-3 w-5/6" />
                </div>
              </div>
            </Transition>
            <div class="mt-5 inline-flex items-center gap-2.5 rounded-control border border-border bg-surface px-3 py-2">
              <span class="icon-chip icon-chip-sm" aria-hidden="true"><FileText :size="13" :stroke-width="1.8" /></span>
              <span class="text-sm text-text">Fattura_12_{{ new Date().getFullYear() }}.pdf</span>
              <Paperclip :size="13" :stroke-width="1.8" class="text-text-subtle" aria-hidden="true" />
            </div>
          </div>
        </article>
      </div>
    </div>

    <ConfirmDialog
      :open="resetOpen"
      title="Ripristinare il modello predefinito?"
      blast-radius="Oggetto e testo tornano quelli proposti dall’app; le tue modifiche al modello vanno perse."
      confirm-label="Ripristina"
      tone="accent"
      :loading="resetting"
      @confirm="confirmReset"
      @cancel="resetOpen = false"
    />
  </AppCard>
</template>

<style scoped>
.placeholder-chip {
  background: var(--surface-raised);
  box-shadow: inset 0 0 0 1px var(--border-strong);
}
.placeholder-chip:hover { box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent) 45%, var(--border-strong)); }

/* The preview sits a step below the card, like a message open in a mail client. */
.mail-sheet {
  background: var(--surface);
  box-shadow: inset 0 0 0 1px var(--border), inset 0 1px 2px rgb(40 34 20 / 0.04);
}
</style>

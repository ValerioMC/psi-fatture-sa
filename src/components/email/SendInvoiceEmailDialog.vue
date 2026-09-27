<script setup lang="ts">
/**
 * The email for one invoice, proposed from the template and editable before it goes.
 * The attachment opens in the system viewer, so what is sent can be checked first.
 * "Invia" flies the paper plane; on success the tick draws and the dialog closes.
 */
import { computed, ref, watch } from 'vue'
import { FileText, Info } from 'lucide-vue-next'
import { openInvoicePdf, prepareInvoiceEmail } from '@/api'
import { useEmailStore } from '@/stores/email'
import { errorMessage, useToastStore } from '@/stores/toast'
import type { EmailDraft, Invoice } from '@/types'
import ActionButton from '@/components/ui/ActionButton.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import FormField from '@/components/ui/FormField.vue'
import { validateEmail } from '@/utils/validation'

const props = defineProps<{ open: boolean; invoice: Invoice }>()
const emit = defineEmits<{ close: []; sent: [] }>()

const email = useEmailStore()
const toast = useToastStore()

const draft = ref<EmailDraft | null>(null)
const loadError = ref('')
const recipient = ref('')
const subject = ref('')
const body = ref('')
const remember = ref(true)
const recipientError = ref('')
const opening = ref(false)

watch(
  () => props.open,
  async (open) => {
    if (!open) return
    draft.value = null
    loadError.value = ''
    recipientError.value = ''
    try {
      const prepared = await prepareInvoiceEmail(props.invoice.id)
      draft.value = prepared
      recipient.value = prepared.recipient
      subject.value = prepared.subject
      body.value = prepared.body
      remember.value = !prepared.recipient_on_file
    } catch (error) {
      loadError.value = errorMessage(error)
    }
  },
  { immediate: true },
)

const certified = computed(() => email.preset?.certified === true)

function validRecipient(): boolean {
  const result = validateEmail(recipient.value)
  recipientError.value = recipient.value.trim() === '' ? 'Serve l’indirizzo del paziente.' : result.valid ? '' : (result.message ?? '')
  return recipientError.value === ''
}

async function send(): Promise<boolean> {
  if (!validRecipient()) return false
  try {
    await email.send({
      invoice_id: props.invoice.id,
      recipient: recipient.value,
      subject: subject.value,
      body: body.value,
      remember_recipient: draft.value?.recipient_on_file === false && remember.value,
    })
    return true
  } catch (error) {
    toast.notifyError(error, 'Email non inviata')
    return false
  }
}

async function preview(): Promise<void> {
  opening.value = true
  try {
    await openInvoicePdf(props.invoice.id)
  } catch (error) {
    toast.notifyError(error, 'PDF non disponibile')
  } finally {
    opening.value = false
  }
}

function finished(): void {
  emit('sent')
  emit('close')
}
</script>

<template>
  <AppDialog :open="open" size="lg" :title="`Invia la fattura N. ${invoice.invoice_number}/${invoice.year}`" :description="`A ${invoice.client_name}, con il PDF allegato.`" @close="emit('close')">
    <p v-if="loadError" class="rounded-control bg-danger-soft px-3 py-2 text-sm text-danger ring-1 ring-inset ring-danger-line" role="alert">{{ loadError }}</p>

    <div v-else-if="draft === null" class="space-y-3" role="status" aria-busy="true">
      <span class="sr-only">Preparazione dell’email</span>
      <div class="skeleton h-9 w-full" />
      <div class="skeleton h-9 w-full" />
      <div class="skeleton h-40 w-full" />
    </div>

    <form v-else class="space-y-4" novalidate @submit.prevent>
      <FormField v-slot="{ id, invalid, describedBy }" label="A" :error="recipientError || undefined" :hint="draft.recipient_on_file ? undefined : 'Il paziente non ha un indirizzo nella sua scheda.'">
        <input
          :id="id"
          v-model="recipient"
          type="email"
          class="field"
          spellcheck="false"
          autocomplete="off"
          :data-autofocus="draft.recipient_on_file ? undefined : true"
          :aria-invalid="invalid"
          :aria-describedby="describedBy"
          @input="recipientError = ''"
          @blur="recipient.trim() !== '' && validRecipient()"
        />
      </FormField>
      <label v-if="!draft.recipient_on_file" class="-mt-1 flex items-center gap-2 text-sm text-text-muted">
        <input v-model="remember" type="checkbox" />
        Salva l’indirizzo nella scheda di {{ invoice.client_name }}
      </label>

      <FormField v-slot="{ id }" label="Oggetto">
        <input :id="id" v-model="subject" type="text" class="field" maxlength="200" />
      </FormField>
      <FormField v-slot="{ id }" label="Testo">
        <textarea :id="id" v-model="body" rows="9" class="field leading-relaxed" />
      </FormField>

      <div class="flex flex-wrap items-center gap-3">
        <button
          type="button"
          class="attachment inline-flex items-center gap-2.5 rounded-control px-3 py-2 text-left transition-colors hover:bg-surface-hover focus-ring"
          :aria-busy="opening || undefined"
          @click="preview"
        >
          <span class="icon-chip icon-chip-sm" aria-hidden="true"><FileText :size="13" :stroke-width="1.8" /></span>
          <span>
            <span class="block text-sm text-text">{{ draft.attachment_name }}</span>
            <span class="block text-xs text-text-subtle">{{ opening ? 'Apertura…' : 'Apri per controllarla' }}</span>
          </span>
        </button>
        <p v-if="certified" class="flex min-w-0 flex-1 items-start gap-2 text-xs leading-relaxed text-text-subtle">
          <Info :size="13" :stroke-width="1.8" class="mt-0.5 shrink-0" aria-hidden="true" />
          Parte dalla tua PEC: il paziente la riceve come posta certificata, con la fattura nel messaggio allegato.
        </p>
      </div>
    </form>

    <template #footer>
      <p class="mr-auto truncate text-xs text-text-subtle">Da {{ email.account?.sender_address }}</p>
      <AppButton variant="ghost" @click="emit('close')">Annulla</AppButton>
      <ActionButton motion="send" :run="send" :disabled="draft === null" done-label="Inviata" working-label="Invio in corso" @done="finished">
        Invia
      </ActionButton>
    </template>
  </AppDialog>
</template>

<style scoped>
.attachment { box-shadow: inset 0 0 0 1px var(--border-strong); background: var(--surface); }
</style>

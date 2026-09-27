<script setup lang="ts">
/**
 * The mailbox invoices leave from. Choosing a provider fills in the server, so a
 * psypec.it address needs only its password; "Altro provider" asks for the server.
 * The password is kept encrypted by the app, and one live login checks it all.
 */
import { computed, reactive, ref, watch } from 'vue'
import { Info, Mail, PlugZap, ShieldCheck } from 'lucide-vue-next'
import { checkEmailConnection } from '@/api'
import { useEmailStore } from '@/stores/email'
import { errorMessage, useToastStore } from '@/stores/toast'
import type { EmailConnectionCheck, EmailProvider, EmailSecurity, UpdateEmailAccountInput } from '@/types'
import ActionButton from '@/components/ui/ActionButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppCard from '@/components/ui/AppCard.vue'
import CardHeader from '@/components/ui/CardHeader.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import FormField from '@/components/ui/FormField.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import type { SegmentOption } from '@/components/ui/types'
import SecretRow from '@/components/profile/SecretRow.vue'
import { detectProvider } from '@/utils/email'
import { validateEmail } from '@/utils/validation'

const email = useEmailStore()
const toast = useToastStore()

const form = reactive<UpdateEmailAccountInput>({
  provider: 'psypec',
  sender_address: '',
  sender_name: '',
  smtp_host: '',
  smtp_port: 465,
  security: 'tls',
  username: '',
  bcc_self: false,
})
const addressError = ref('')
const check = ref<EmailConnectionCheck | null>(null)
const removeOpen = ref(false)
const removing = ref(false)

watch(
  () => email.account,
  (account) => {
    if (account === null) return
    const { saved: _saved, ...editable } = account
    Object.assign(form, editable)
  },
  { immediate: true },
)

const SHORT_HINT: Record<EmailProvider, string> = {
  psypec: 'PEC dell’Ordine',
  aruba_pec: 'Indirizzi @pec.it',
  gmail: 'Password per le app',
  custom: 'Server a mano',
}

const preset = computed(() => email.providers.find((p) => p.provider === form.provider) ?? null)
const custom = computed(() => form.provider === 'custom')

/** Typing an address of a known domain picks its provider, so the server follows. */
function onAddressInput(): void {
  addressError.value = ''
  const detected = detectProvider(form.sender_address, email.providers)
  if (detected !== 'custom') choose(detected)
}

function choose(provider: EmailProvider): void {
  form.provider = provider
  const chosen = email.providers.find((p) => p.provider === provider)
  if (chosen && provider !== 'custom') {
    form.smtp_host = chosen.host
    form.smtp_port = chosen.port
    form.security = chosen.security
  }
}

const SECURITY_OPTIONS: SegmentOption<EmailSecurity>[] = [
  { value: 'tls', label: 'SSL/TLS' },
  { value: 'starttls', label: 'STARTTLS' },
]

/** Switching the protection moves the port to that protection's standard one. */
function setSecurity(security: EmailSecurity): void {
  form.security = security
  form.smtp_port = security === 'tls' ? 465 : 587
}

const dirty = computed(() => {
  const saved = email.account
  if (saved === null) return false
  if (!saved.saved) return true
  return (Object.keys(form) as (keyof UpdateEmailAccountInput)[]).some((key) => form[key] !== saved[key])
})

watch(form, () => { check.value = null })

const passwordHint = computed(() =>
  form.provider === 'gmail'
    ? 'La password per le app creata nell’account Google, non quella di accesso.'
    : 'La password con cui entri nella casella.',
)

async function save(): Promise<boolean> {
  const result = validateEmail(form.sender_address)
  if (form.sender_address.trim() === '' || !result.valid) {
    addressError.value = result.message ?? 'Serve l’indirizzo da cui inviare.'
    return false
  }
  try {
    await email.saveAccount({ ...form, username: custom.value ? form.username : '' })
    return true
  } catch (error) {
    toast.notifyError(error, 'Casella non salvata')
    return false
  }
}

async function savePassword(value: string): Promise<void> {
  await email.savePassword(value)
  toast.notify('Password salvata e cifrata')
}

async function confirmRemove(): Promise<void> {
  removing.value = true
  try {
    await email.removePassword()
    toast.notify('Password rimossa')
    removeOpen.value = false
  } catch (error) {
    toast.notifyError(error, 'Non rimossa')
  } finally {
    removing.value = false
  }
}

async function verify(): Promise<boolean> {
  try {
    check.value = await checkEmailConnection()
  } catch (error) {
    check.value = { ok: false, message: errorMessage(error) }
  }
  return check.value.ok
}

const canVerify = computed(() => email.ready && !dirty.value)
</script>

<template>
  <AppCard :padded="false">
    <CardHeader title="Casella di invio" subtitle="Da dove partono le fatture ai pazienti" :icon="Mail">
      <AppBadge v-if="email.ready" tone="safe" dot>Pronta</AppBadge>
      <AppBadge v-else tone="warn" dot>Da configurare</AppBadge>
    </CardHeader>

    <div class="space-y-5 px-5 pb-5">
      <!-- Provider: one tile each, psypec.it first because every psychologist has one. -->
      <div class="grid grid-cols-2 gap-2.5 sm:grid-cols-4" role="radiogroup" aria-label="Provider della casella">
        <button
          v-for="option in email.providers"
          :key="option.provider"
          type="button"
          role="radio"
          :aria-checked="form.provider === option.provider"
          class="provider-tile flex flex-col items-start gap-0.5 rounded-control px-3 py-2.5 text-left transition-[box-shadow,background-color] duration-150 focus-ring"
          :class="form.provider === option.provider ? 'is-chosen' : 'ring-1 ring-border hover:ring-border-strong hover:bg-surface-hover'"
          @click="choose(option.provider)"
        >
          <span class="text-sm font-medium" :class="form.provider === option.provider ? 'text-text' : 'text-text-muted'">{{ option.label }}</span>
          <span class="text-xs text-text-subtle">{{ SHORT_HINT[option.provider] }}</span>
          <span v-if="option.provider === 'psypec'" class="mt-1 text-2xs font-medium text-accent">Consigliata</span>
        </button>
      </div>

      <div class="grid grid-cols-1 gap-x-4 gap-y-5 sm:grid-cols-2">
        <FormField v-slot="{ id, invalid, describedBy }" label="Indirizzo email" required :error="addressError" :hint="form.provider === 'psypec' ? 'Es. nome.cognome@psypec.it' : undefined">
          <input
            :id="id"
            v-model="form.sender_address"
            type="email"
            class="field"
            autocomplete="email"
            spellcheck="false"
            :aria-invalid="invalid"
            :aria-describedby="describedBy"
            @input="onAddressInput"
          />
        </FormField>
        <FormField v-slot="{ id }" label="Nome del mittente" hint="Come ti vede il paziente nella sua casella.">
          <input :id="id" v-model="form.sender_name" type="text" class="field" maxlength="100" />
        </FormField>
      </div>

      <!-- What this provider needs to know, and what a PEC means for the patient. -->
      <div v-if="preset" class="flex gap-2.5 rounded-control bg-surface-sunken/60 px-3.5 py-3 text-xs leading-relaxed text-text-muted">
        <Info :size="14" :stroke-width="1.8" class="mt-0.5 shrink-0 text-accent" aria-hidden="true" />
        <div class="space-y-1.5">
          <p>{{ preset.note }}</p>
          <p v-if="preset.certified">
            Da una PEC il paziente riceve un messaggio di posta certificata: la fattura è dentro il messaggio originale allegato
            (<span class="font-mono">postacert.eml</span>), e a te arriva la ricevuta di accettazione.
          </p>
        </div>
      </div>

      <div v-if="custom" class="grid grid-cols-[minmax(0,1fr)_6rem] gap-x-4 gap-y-5">
        <FormField v-slot="{ id }" label="Server SMTP" required hint="Nella guida del provider, alla voce posta in uscita.">
          <input :id="id" v-model="form.smtp_host" type="text" class="field field-mono" spellcheck="false" autocomplete="off" placeholder="smtp.esempio.it" />
        </FormField>
        <FormField v-slot="{ id }" label="Porta" required>
          <input :id="id" v-model.number="form.smtp_port" type="number" min="1" max="65535" class="field tabular" />
        </FormField>
        <div class="col-span-2 flex flex-wrap items-end gap-x-6 gap-y-4">
          <div>
            <p class="mb-1.5 text-sm font-medium text-text-muted">Sicurezza</p>
            <SegmentedControl :model-value="form.security" :options="SECURITY_OPTIONS" label="Sicurezza della connessione" size="sm" @update:model-value="setSecurity" />
          </div>
          <FormField v-slot="{ id }" class="min-w-56 flex-1" label="Nome utente" optional hint="Vuoto: l’indirizzo email.">
            <input :id="id" v-model="form.username" type="text" class="field" spellcheck="false" autocomplete="off" />
          </FormField>
        </div>
      </div>
      <p v-else-if="preset" class="flex flex-wrap items-center gap-x-2 text-xs text-text-subtle">
        <span>Server</span>
        <span class="font-mono text-text-muted">{{ preset.host }}</span>
        <span aria-hidden="true">·</span>
        <span class="tabular">porta {{ preset.port }}</span>
        <span aria-hidden="true">·</span>
        <span>{{ preset.security === 'tls' ? 'SSL/TLS' : 'STARTTLS' }}</span>
      </p>

      <ToggleSwitch v-model="form.bcc_self" label="Mandami una copia di ogni invio" description="In copia nascosta: il paziente non la vede." />

      <div v-if="dirty" class="flex items-center justify-end gap-3">
        <span class="text-xs text-text-subtle">{{ email.account?.saved ? 'Modifiche da salvare' : 'Salva per usare questa casella' }}</span>
        <ActionButton :run="save" size="sm" done-label="Salvata" working-label="Salvataggio in corso">Salva casella</ActionButton>
      </div>

      <div v-if="email.credentials" class="space-y-2">
        <SecretRow
          label="Password"
          field-label="Password della casella"
          :hint="passwordHint"
          :configured="email.credentials.password_configured"
          :save="savePassword"
          @remove="removeOpen = true"
        />
        <p class="flex items-start gap-2 text-xs leading-relaxed text-text-subtle">
          <ShieldCheck :size="14" :stroke-width="1.8" class="mt-0.5 shrink-0" aria-hidden="true" />
          Resta su questo computer, cifrata dall’app con una chiave legata al computer, e viaggia solo verso il server di posta, su connessione protetta.
        </p>
      </div>

      <div class="border-t border-border pt-4">
        <ActionButton
          block
          variant="secondary"
          :icon="PlugZap"
          :run="verify"
          :disabled="!canVerify"
          done-label="Collegata"
          working-label="Verifica del collegamento in corso"
        >
          Verifica collegamento
        </ActionButton>
        <p v-if="!canVerify" class="mt-2 text-xs text-text-subtle">
          {{ dirty ? 'Salva la casella per verificarla.' : 'Serve la password per verificare il collegamento.' }}
        </p>
        <Transition name="pane">
          <p
            v-if="check"
            class="mt-2 rounded-control px-3 py-2 text-xs leading-relaxed ring-1 ring-inset"
            :class="check.ok ? 'bg-safe-soft text-safe ring-safe-line' : 'bg-danger-soft text-danger ring-danger-line'"
            role="status"
          >
            {{ check.message }}
          </p>
        </Transition>
      </div>
    </div>

    <ConfirmDialog
      :open="removeOpen"
      title="Rimuovere la password?"
      message="Viene cancellata dall’archivio cifrato dell’app."
      blast-radius="Finché non la inserisci di nuovo, nessuna fattura può partire via email."
      confirm-label="Rimuovi"
      :loading="removing"
      @confirm="confirmRemove"
      @cancel="removeOpen = false"
    />
  </AppCard>
</template>

<style scoped>
/* The chosen provider looks picked, not just coloured: a raised tile with an ink edge and a soft glow. */
.provider-tile.is-chosen {
  background: var(--surface-raised);
  box-shadow: 0 0 0 1.5px var(--accent), var(--sheet-highlight), 0 6px 16px -10px color-mix(in srgb, var(--accent) 60%, transparent);
}
</style>

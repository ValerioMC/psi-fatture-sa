<script setup lang="ts">
/**
 * The Sistema TS connection: which environment, who logs in, under which
 * P.IVA documents go out, and the two secrets kept in the OS keychain. A
 * single live call checks that all of it is accepted.
 *
 * A step-by-step guide says where the credentials come from; the test
 * environment accepts only the public user of the Sogei kit, so there it can
 * be filled in with one click.
 */
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute } from 'vue-router'
import { FlaskConical, KeyRound, PlugZap, ShieldCheck } from 'lucide-vue-next'
import {
  checkTsConnection,
  deleteTsPassword,
  deleteTsPincode,
  getTsCredentialsStatus,
  saveTsPassword,
  saveTsPincode,
} from '@/api'
import { useStsStore } from '@/stores/sts'
import { errorMessage, useToastStore } from '@/stores/toast'
import type { TsConnectionCheck, TsEnvironment, TsSettings } from '@/types'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import CardHeader from '@/components/ui/CardHeader.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import FormField from '@/components/ui/FormField.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import TsCredentialsGuide from '@/components/sts/TsCredentialsGuide.vue'
import TsSecretRow from './TsSecretRow.vue'

/** The public psychologist user of the Sogei development kit (kit730P 20240214). */
const SOGEI_TEST_USER = {
  username: 'MTOMRA66A41G224M',
  vat_number: '65498732105',
  password: 'Salve123',
  pincode: '3489543096',
} as const

const ENVIRONMENTS = [
  { value: 'produzione', label: 'Produzione' },
  { value: 'test', label: 'Test Sogei', tone: 'warn' },
] as const

const sts = useStsStore()
const toast = useToastStore()

const form = reactive<TsSettings>({ environment: 'produzione', username: '', vat_number: '' })
const { credentials } = storeToRefs(sts)
const route = useRoute()
const sectionRef = ref<HTMLElement | null>(null)
const saving = ref(false)
const checking = ref(false)
const check = ref<TsConnectionCheck | null>(null)
const removing = ref<'password' | 'pincode' | null>(null)
const removeBusy = ref(false)

onMounted(async () => {
  try {
    if (!sts.loaded) await sts.load()
    credentials.value = await getTsCredentialsStatus()
  } catch (error) {
    toast.notifyError(error, 'Impostazioni Sistema TS non leggibili')
  }
  if (route.query.focus === 'sts') sectionRef.value?.scrollIntoView({ behavior: 'smooth', block: 'start' })
})

const incomplete = computed(() => credentials.value !== null && !(credentials.value.password_configured && credentials.value.pincode_configured))

watch(
  () => sts.settings,
  (settings) => {
    if (settings) Object.assign(form, settings)
  },
  { immediate: true },
)

const dirty = computed(() => {
  const saved = sts.settings
  return saved !== null && (saved.environment !== form.environment || saved.username !== form.username.trim().toUpperCase() || saved.vat_number !== form.vat_number.trim())
})

watch(form, () => { check.value = null })

async function saveIdentity(): Promise<void> {
  saving.value = true
  try {
    await sts.saveSettings({ ...form })
    toast.notify(form.environment === 'test' ? 'Ambiente di test attivo' : 'Impostazioni Sistema TS salvate')
  } catch (error) {
    toast.notifyError(error, 'Non salvate')
  } finally {
    saving.value = false
  }
}

async function savePassword(value: string): Promise<void> {
  credentials.value = await saveTsPassword(value)
  toast.notify('Password nel portachiavi')
}

async function savePincode(value: string): Promise<void> {
  credentials.value = await saveTsPincode(value)
  toast.notify('PINCODE nel portachiavi')
}

async function confirmRemove(): Promise<void> {
  const which = removing.value
  if (which === null) return
  removeBusy.value = true
  try {
    credentials.value = which === 'password' ? await deleteTsPassword() : await deleteTsPincode()
    toast.notify(which === 'password' ? 'Password rimossa' : 'PINCODE rimosso')
    removing.value = null
  } catch (error) {
    toast.notifyError(error, 'Non rimosso')
  } finally {
    removeBusy.value = false
  }
}

async function useSogeiTestUser(): Promise<void> {
  saving.value = true
  try {
    await sts.saveSettings({ environment: 'test', username: SOGEI_TEST_USER.username, vat_number: SOGEI_TEST_USER.vat_number })
    await saveTsPassword(SOGEI_TEST_USER.password)
    credentials.value = await saveTsPincode(SOGEI_TEST_USER.pincode)
    toast.notify('Utenza di prova Sogei pronta')
  } catch (error) {
    toast.notifyError(error, 'Utenza di prova non impostata')
  } finally {
    saving.value = false
  }
}

async function verify(): Promise<void> {
  checking.value = true
  try {
    check.value = await checkTsConnection()
  } catch (error) {
    check.value = { ok: false, message: errorMessage(error) }
  } finally {
    checking.value = false
  }
}

function setEnvironment(value: TsEnvironment): void {
  form.environment = value
}
</script>

<template>
  <section id="sistema-ts" ref="sectionRef" class="scroll-mt-32">
    <AppCard :padded="false">
      <CardHeader title="Sistema Tessera Sanitaria" subtitle="Invio delle spese sanitarie" :icon="KeyRound" />

      <div class="space-y-4 px-5 pb-5">
        <SegmentedControl
          :model-value="form.environment"
          :options="ENVIRONMENTS"
          label="Ambiente Sistema TS"
          block
          size="sm"
          @update:model-value="setEnvironment"
        />

        <p v-if="form.environment === 'test'" class="flex items-start gap-2 rounded-control bg-warn-soft px-3 py-2 text-xs leading-relaxed text-text-muted ring-1 ring-inset ring-warn-line">
          <FlaskConical :size="14" class="mt-0.5 shrink-0 text-warn" aria-hidden="true" />
          <span>
            Le trasmissioni vanno all’ambiente di prova di Sogei e non hanno valore fiscale.
            <button type="button" class="font-medium text-text underline decoration-border-strong underline-offset-2 hover:decoration-text focus-ring rounded-sm" :disabled="saving" @click="useSogeiTestUser">
              Usa l’utenza di prova
            </button>
          </span>
        </p>

        <TsCredentialsGuide v-if="form.environment === 'produzione'" :initially-open="incomplete || route.query.focus === 'sts'" />

        <form class="space-y-3" novalidate @submit.prevent="saveIdentity">
          <FormField v-slot="{ id, invalid, describedBy }" label="Codice fiscale di accesso" hint="L’utente con cui entri nel Sistema TS.">
            <input :id="id" v-model="form.username" class="field font-mono uppercase" autocomplete="off" spellcheck="false" maxlength="16" :aria-invalid="invalid" :aria-describedby="describedBy" />
          </FormField>
          <FormField v-slot="{ id, invalid, describedBy }" label="Partita IVA" hint="Identifica le tue fatture sul Sistema TS.">
            <input :id="id" v-model="form.vat_number" class="field font-mono" inputmode="numeric" autocomplete="off" maxlength="11" :aria-invalid="invalid" :aria-describedby="describedBy" />
          </FormField>
          <div v-if="dirty" class="flex justify-end">
            <AppButton variant="primary" size="sm" type="submit" :loading="saving">Salva ambiente e utente</AppButton>
          </div>
        </form>

        <div v-if="credentials" class="space-y-2">
          <TsSecretRow label="Password" hint="La password del Sistema TS." :configured="credentials.password_configured" :save="savePassword" @remove="removing = 'password'" />
          <TsSecretRow label="PINCODE" hint="Quello delle tue credenziali Sistema TS." :configured="credentials.pincode_configured" no-spaces :save="savePincode" @remove="removing = 'pincode'" />
        </div>

        <p class="flex items-start gap-2 text-xs leading-relaxed text-text-subtle">
          <ShieldCheck :size="14" :stroke-width="1.8" class="mt-0.5 shrink-0" aria-hidden="true" />
          Password e PINCODE stanno solo nel portachiavi di sistema, mai su disco. Viaggiano cifrati come richiede il Sistema TS.
        </p>

        <div class="border-t border-border pt-4">
          <AppButton block :icon="PlugZap" :loading="checking" :disabled="dirty" @click="verify">Verifica credenziali</AppButton>
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
        :open="removing !== null"
        :title="removing === 'password' ? 'Rimuovere la password?' : 'Rimuovere il PINCODE?'"
        message="Viene cancellato dal portachiavi di sistema."
        blast-radius="Le trasmissioni in coda al Sistema TS restano ferme finché non lo inserisci di nuovo."
        confirm-label="Rimuovi"
        :loading="removeBusy"
        @confirm="confirmRemove"
        @cancel="removing = null"
      />
    </AppCard>
  </section>
</template>

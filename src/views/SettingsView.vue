<script setup lang="ts">
/**
 * Settings, one context per page. The profile form spans "Profilo" and "Fatturazione"
 * and is saved from one bar that rises when something differs; leaving with changes asks first.
 */
import { computed, onMounted, ref } from 'vue'
import { onBeforeRouteLeave, useRouter } from 'vue-router'
import { DatabaseBackup, IdCard, KeyRound, Mail, Palette, ReceiptText, Scale, Settings } from 'lucide-vue-next'
import { useEmailStore } from '@/stores/email'
import { useStsStore } from '@/stores/sts'
import { useToastStore } from '@/stores/toast'
import PageHeader from '@/components/ui/PageHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import SettingsNav, { type SettingsNavItem } from '@/components/profile/SettingsNav.vue'
import { provideSettingsProfile } from '@/composables/useSettingsProfile'

const router = useRouter()
const toast = useToastStore()
const email = useEmailStore()
const sts = useStsStore()
const profile = provideSettingsProfile()

onMounted(async () => {
  await profile.load()
  // The status dots are a courtesy: without them the pages still work.
  email.load().catch(() => undefined)
  if (!sts.loaded) sts.load().catch(() => undefined)
})

const items = computed<SettingsNavItem[]>(() => [
  { to: '/settings/profile', label: 'Profilo', hint: 'Chi sei, studio, pagamenti', icon: IdCard },
  { to: '/settings/invoicing', label: 'Fatturazione', hint: 'Regime, numerazione, stampa', icon: ReceiptText },
  {
    to: '/settings/email',
    label: 'Email',
    hint: 'Casella e modello',
    icon: Mail,
    status: email.loaded ? (email.ready ? { tone: 'safe', label: 'Pronta per inviare' } : { tone: 'warn', label: 'Da configurare' }) : undefined,
  },
  {
    to: '/settings/sts',
    label: 'Sistema TS',
    hint: 'Credenziali di invio',
    icon: KeyRound,
    status: sts.loaded && !sts.connected ? { tone: 'warn', label: 'Credenziali mancanti' } : undefined,
  },
  { to: '/settings/data', label: 'Dati e backup', hint: 'Una copia dell’archivio', icon: DatabaseBackup },
  { to: '/settings/appearance', label: 'Aspetto', hint: 'Tema chiaro o scuro', icon: Palette },
  { to: '/settings/legal', label: 'Condizioni d’uso', hint: 'Il testo che hai accettato', icon: Scale },
])

async function save(): Promise<void> {
  const outcome = await profile.save()
  if (outcome.ok) {
    toast.notify('Profilo salvato')
    return
  }
  if (outcome.page !== null) void router.push(`/settings/${outcome.page}`)
  toast.notifyError(outcome.error, outcome.page !== null ? undefined : 'Salvataggio non riuscito')
}

const leaveOpen = ref(false)
let pendingLeave: (() => void) | null = null
onBeforeRouteLeave((_to, _from, next) => {
  if (!profile.dirty.value) return next()
  pendingLeave = () => next()
  leaveOpen.value = true
  next(false)
})

function discardAndLeave(): void {
  leaveOpen.value = false
  profile.reset()
  pendingLeave?.()
}
</script>

<template>
  <div>
    <PageHeader title="Impostazioni" subtitle="Il tuo profilo, come fatturi, la casella email, il Sistema TS e l'aspetto dell'app." :icon="Settings" />

    <div class="page grid grid-cols-1 items-start gap-6 pt-6 pb-28 @min-[48rem]/pane:grid-cols-[14rem_minmax(0,1fr)]">
      <aside class="settle @min-[48rem]/pane:sticky @min-[48rem]/pane:top-32">
        <SettingsNav :items="items" />
      </aside>
      <div class="@container/settings min-w-0">
        <RouterView v-slot="{ Component }">
          <Transition name="pane" mode="out-in">
            <component :is="Component" />
          </Transition>
        </RouterView>
      </div>
    </div>

    <Teleport to="body">
      <Transition name="rise">
        <form
          v-if="profile.dirty.value"
          class="fixed bottom-6 left-[calc(50%+var(--spacing-sidebar)/2)] z-(--z-overlay) flex -translate-x-1/2 items-center gap-4 rounded-card border border-border bg-surface-raised py-2 pl-5 pr-2 shadow-modal"
          aria-label="Modifiche non salvate al profilo"
          novalidate
          @submit.prevent="save"
        >
          <span class="whitespace-nowrap text-base text-text">Modifiche non salvate al profilo</span>
          <AppButton variant="ghost" size="sm" @click="profile.reset">Annulla</AppButton>
          <AppButton variant="primary" size="sm" type="submit" :loading="profile.saving.value">Salva</AppButton>
        </form>
      </Transition>
    </Teleport>

    <ConfirmDialog
      :open="leaveOpen"
      title="Uscire senza salvare?"
      blast-radius="Le modifiche al profilo andranno perse."
      confirm-label="Esci senza salvare"
      @confirm="discardAndLeave"
      @cancel="leaveOpen = false"
    />
  </div>
</template>

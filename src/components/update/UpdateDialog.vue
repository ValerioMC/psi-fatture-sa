<script setup lang="ts">
/**
 * Announces a newer release at startup. On a Mac one button installs it over this app and
 * restarts; elsewhere it opens the download. The archive lives in its own folder and stays.
 */
import { computed } from 'vue'
import { Download } from 'lucide-vue-next'
import { useUpdateStore } from '@/stores/update'
import { useToastStore } from '@/stores/toast'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'

const update = useUpdateStore()
const toast = useToastStore()

const message = computed(() =>
  update.installsItself
    ? 'L’app scarica la nuova versione, la installa al posto di questa e si riavvia. Pazienti, fatture e impostazioni restano al loro posto.'
    : `Scarica il file per ${update.latest?.platform_label ?? 'il tuo sistema'} e installalo sopra l’app attuale: pazienti, fatture e impostazioni restano al loro posto.`,
)

async function apply(): Promise<void> {
  try {
    await update.apply()
  } catch (error) {
    toast.notifyError(error, 'Impossibile aprire il download')
  }
}
</script>

<template>
  <AppDialog
    :open="update.promptOpen && update.latest !== null"
    :title="`PSI Fatture ${update.latest?.latest_version ?? ''} è disponibile`"
    :description="`Hai la versione ${update.latest?.current_version ?? ''}.`"
    size="sm"
    :dismissible="!update.installing"
    @close="update.dismiss"
  >
    <p class="text-base text-text-muted">{{ message }}</p>
    <p v-if="update.installFailure" class="mt-3 text-sm text-danger" role="alert">
      Installazione non riuscita: {{ update.installFailure }}. Puoi scaricarla dal sito e installarla a mano.
    </p>
    <template #footer>
      <div class="flex-1" />
      <AppButton variant="ghost" :disabled="update.installing" @click="update.dismiss">Più tardi</AppButton>
      <AppButton variant="primary" :icon="Download" :loading="update.installing" data-autofocus @click="apply">
        {{ update.installing ? 'Installazione…' : 'Scarica' }}
      </AppButton>
    </template>
  </AppDialog>
</template>

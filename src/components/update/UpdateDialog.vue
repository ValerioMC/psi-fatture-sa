<script setup lang="ts">
/**
 * Announces a newer release at startup, with the download for this system. Installing over
 * the current app keeps the archive, which lives in its own folder.
 */
import { Download } from 'lucide-vue-next'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useUpdateStore } from '@/stores/update'
import { useToastStore } from '@/stores/toast'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'

const update = useUpdateStore()
const toast = useToastStore()

async function download(url: string): Promise<void> {
  try {
    await openUrl(url)
    update.dismiss()
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
    @close="update.dismiss"
  >
    <template v-if="update.latest">
      <p class="text-base text-text-muted">
        Scarica il file per {{ update.latest.platform_label }} e installalo sopra l’app attuale: pazienti, fatture e
        impostazioni restano al loro posto.
      </p>
      <a
        :href="update.latest.download_url"
        class="mt-4 block break-all rounded-control bg-surface-sunken px-3 py-2 font-mono text-xs text-text-muted ring-1 ring-inset ring-border hover:text-text focus-ring"
        @click.prevent="download(update.latest.download_url)"
      >{{ update.latest.download_url }}</a>
    </template>
    <template #footer>
      <div class="flex-1" />
      <AppButton variant="ghost" @click="update.dismiss">Più tardi</AppButton>
      <AppButton
        v-if="update.latest"
        variant="primary"
        :icon="Download"
        data-autofocus
        @click="download(update.latest.download_url)"
      >
        Scarica
      </AppButton>
    </template>
  </AppDialog>
</template>

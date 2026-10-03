<script setup lang="ts">
/** This build's version, and a check by hand against the newest release, with its download. */
import { computed, onMounted, ref } from 'vue'
import { BadgeInfo, Download, RefreshCw } from 'lucide-vue-next'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useUpdateStore } from '@/stores/update'
import { errorMessage, useToastStore } from '@/stores/toast'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import CardHeader from '@/components/ui/CardHeader.vue'

const update = useUpdateStore()
const toast = useToastStore()
const failure = ref<string | null>(null)

const subtitle = computed(() => (update.version ? `Versione ${update.version}` : 'Versione installata'))

async function checkNow(): Promise<void> {
  failure.value = null
  try {
    await update.check()
  } catch (error) {
    failure.value = errorMessage(error)
  }
}

async function download(url: string): Promise<void> {
  try {
    await openUrl(url)
  } catch (error) {
    toast.notifyError(error, 'Impossibile aprire il download')
  }
}

onMounted(() => {
  update.loadVersion().catch((error: unknown) => toast.notifyError(error, 'Versione non leggibile'))
})
</script>

<template>
  <AppCard :padded="false" class="settle max-w-2xl">
    <CardHeader title="PSI Fatture" :subtitle="subtitle" :icon="BadgeInfo">
      <AppButton size="sm" :icon="RefreshCw" :loading="update.checking" @click="checkNow">Controlla aggiornamenti</AppButton>
    </CardHeader>
    <div class="space-y-4 px-5 pb-5 text-sm leading-relaxed text-text-muted">
      <p>All’avvio l’app controlla se è uscita una versione più recente e, se c’è, ti propone il file da scaricare.</p>

      <p v-if="failure" class="text-danger" role="alert">{{ failure }}</p>

      <div v-else-if="update.latest && update.latest.update_available" class="flex items-center gap-3 rounded-control bg-accent-soft px-3.5 py-3 ring-1 ring-inset ring-accent-line">
        <span class="min-w-0 flex-1 text-text">
          È disponibile la versione <strong class="font-semibold">{{ update.latest.latest_version }}</strong> per {{ update.latest.platform_label }}.
        </span>
        <AppButton variant="primary" size="sm" :icon="Download" @click="download(update.latest.download_url)">Scarica</AppButton>
      </div>

      <p v-else-if="update.latest" class="flex items-center gap-2">
        <AppBadge tone="safe" dot>Aggiornata</AppBadge>
        <span>{{ update.latest.current_version }} è l’ultima versione pubblicata.</span>
      </p>
    </div>
  </AppCard>
</template>

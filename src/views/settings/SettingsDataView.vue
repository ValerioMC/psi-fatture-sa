<script setup lang="ts">
/**
 * The archive and its copies: one file with patients, invoices, agenda and settings.
 * A rotation of automatic copies sits beside it; saving one elsewhere is one click;
 * restoring is replacing the file by hand.
 */
import { computed, onMounted, ref } from 'vue'
import { DatabaseBackup, FolderOpen, HardDriveDownload, History, RefreshCw, TriangleAlert } from 'lucide-vue-next'
import { save as chooseSavePath } from '@tauri-apps/plugin-dialog'
import { createBackup, exportBackup, getBackupFileName, getBackupOverview, revealBackupsFolder } from '@/api'
import { useToastStore } from '@/stores/toast'
import { formatBytes, formatLocalTimestamp } from '@/utils/format'
import type { BackupOverview, BackupReason } from '@/types'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import CardHeader from '@/components/ui/CardHeader.vue'
import SkeletonRows from '@/components/ui/SkeletonRows.vue'

const toast = useToastStore()
const overview = ref<BackupOverview | null>(null)
const backingUp = ref(false)
const saving = ref(false)

const REASON_LABEL: Record<BackupReason, string> = { auto: 'Automatico', manual: 'Manuale' }

const subtitle = computed(() =>
  overview.value ? `Uno al giorno, si tengono gli ultimi ${overview.value.keep}` : 'Uno al giorno',
)

async function load(): Promise<void> {
  try {
    overview.value = await getBackupOverview()
  } catch (error) {
    toast.notifyError(error, 'Backup non leggibili')
  }
}

async function backUpNow(): Promise<void> {
  backingUp.value = true
  try {
    await createBackup()
    toast.notify('Backup creato')
  } catch (error) {
    toast.notifyError(error, 'Backup non creato')
  } finally {
    backingUp.value = false
    await load()
  }
}

async function openFolder(): Promise<void> {
  try {
    await revealBackupsFolder()
  } catch (error) {
    toast.notifyError(error, 'Cartella non apribile')
  }
}

async function saveBackup(): Promise<void> {
  saving.value = true
  try {
    const path = await chooseSavePath({ defaultPath: await getBackupFileName(), filters: [{ name: 'Backup PSI Fatture', extensions: ['db'] }] })
    if (path === null) return
    await exportBackup(path)
    toast.notify('Backup salvato')
  } catch (error) {
    toast.notifyError(error, 'Backup non salvato')
  } finally {
    saving.value = false
  }
}

onMounted(load)
</script>

<template>
  <div class="max-w-2xl space-y-6">
    <AppCard :padded="false" class="settle">
      <CardHeader title="Backup automatici" :subtitle="subtitle" :icon="History" divided>
        <AppButton size="sm" :icon="RefreshCw" :loading="backingUp" @click="backUpNow">Backup ora</AppButton>
      </CardHeader>

      <dl class="grid gap-3 border-b border-border px-5 py-4 text-sm">
        <div>
          <dt class="text-xs text-text-subtle">Archivio in uso</dt>
          <dd class="mt-0.5 break-all font-mono text-xs text-text">{{ overview?.database_path ?? '…' }}</dd>
        </div>
        <div class="flex items-end gap-3">
          <div class="min-w-0 flex-1">
            <dt class="text-xs text-text-subtle">Cartella dei backup</dt>
            <dd class="mt-0.5 break-all font-mono text-xs text-text">{{ overview?.backups_folder ?? '…' }}</dd>
          </div>
          <AppButton variant="ghost" size="sm" :icon="FolderOpen" @click="openFolder">Apri</AppButton>
        </div>
      </dl>

      <p
        v-if="overview?.last_failure"
        class="mx-5 mt-4 flex items-start gap-2 rounded-control bg-danger-soft px-3 py-2.5 text-sm text-danger ring-1 ring-inset ring-danger-line"
        role="alert"
      >
        <TriangleAlert :size="15" :stroke-width="1.8" class="mt-0.5 shrink-0" aria-hidden="true" />
        <span>L’ultimo backup non è riuscito: {{ overview.last_failure }}</span>
      </p>

      <SkeletonRows v-if="overview === null" variant="list" :count="3" class="px-5 py-4" />
      <p v-else-if="overview.backups.length === 0" class="px-5 py-5 text-sm text-text-muted">
        Nessun backup ancora: il primo viene creato pochi secondi dopo l’avvio dell’app.
      </p>
      <ul v-else class="divide-y divide-border" aria-label="Backup disponibili">
        <li v-for="backup in overview.backups" :key="backup.name" class="flex items-center gap-3 px-5 py-2.5 text-sm">
          <span class="min-w-0 flex-1">
            <span class="block text-text tabular-nums">{{ formatLocalTimestamp(backup.created_at) }}</span>
            <span class="block truncate font-mono text-xs text-text-subtle" :title="backup.path">{{ backup.name }}</span>
          </span>
          <AppBadge :tone="backup.reason === 'auto' ? 'neutral' : 'accent'">{{ REASON_LABEL[backup.reason] }}</AppBadge>
          <span class="w-16 text-right text-xs text-text-muted tabular-nums">{{ formatBytes(backup.size_bytes) }}</span>
        </li>
      </ul>

      <p class="border-t border-border px-5 py-4 text-xs leading-relaxed text-text-subtle">
        Per ripristinare un backup chiudi l’app, sostituisci <code class="font-mono">database.db</code> con la copia scelta
        rinominandola <code class="font-mono">database.db</code> ed elimina, se ci sono, i file che finiscono in
        <code class="font-mono">-wal</code> e <code class="font-mono">-shm</code>. Le copie restano su questo computer: per
        proteggerti da un guasto del disco salvane una anche altrove.
      </p>
    </AppCard>

    <AppCard :padded="false" class="settle">
      <CardHeader title="Salva una copia altrove" subtitle="Pazienti, fatture, agenda e impostazioni in un solo file" :icon="DatabaseBackup" />
      <div class="space-y-4 px-5 pb-5 text-sm leading-relaxed text-text-muted">
        <p>
          L’archivio vive solo su questo computer: se il disco si guasta o il computer viene perso, le fatture se ne vanno con lui.
          Salva una copia almeno una volta al mese su un disco esterno o in una cartella sincronizzata.
        </p>
        <p class="text-xs text-text-subtle">
          La copia contiene dati sanitari dei pazienti: conservala in un posto protetto. Password del Sistema TS e della
          casella email non sono incluse: dopo un ripristino su un altro computer vanno reinserite.
        </p>
        <AppButton variant="primary" :icon="HardDriveDownload" :loading="saving" @click="saveBackup">Salva un backup…</AppButton>
      </div>
    </AppCard>
  </div>
</template>

<script setup lang="ts">
/**
 * The archive and its backup: one file with patients, invoices, agenda and
 * settings. Saving a copy is one click; restoring is replacing the file by hand.
 */
import { ref } from 'vue'
import { DatabaseBackup, HardDriveDownload } from 'lucide-vue-next'
import { save as chooseSavePath } from '@tauri-apps/plugin-dialog'
import { exportBackup, getBackupFileName } from '@/api'
import { useToastStore } from '@/stores/toast'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import CardHeader from '@/components/ui/CardHeader.vue'

const toast = useToastStore()
const saving = ref(false)

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
</script>

<template>
  <AppCard :padded="false" class="settle max-w-2xl">
    <CardHeader title="Backup dell'archivio" subtitle="Pazienti, fatture, agenda e impostazioni in un solo file" :icon="DatabaseBackup" />
    <div class="space-y-4 px-5 pb-5 text-sm leading-relaxed text-text-muted">
      <p>
        L’archivio vive solo su questo computer: se il disco si guasta o il Mac viene perso, le fatture se ne vanno con lui.
        Salva una copia almeno una volta al mese su un disco esterno o in una cartella sincronizzata.
      </p>
      <p class="text-xs text-text-subtle">
        La copia contiene dati sanitari dei pazienti: conservala in un posto protetto. Password del Sistema TS e della
        casella email non sono incluse: dopo un ripristino su un altro computer vanno reinserite.
      </p>
      <AppButton variant="primary" :icon="HardDriveDownload" :loading="saving" @click="saveBackup">Salva un backup…</AppButton>
    </div>
  </AppCard>
</template>

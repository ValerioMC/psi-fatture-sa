<script setup lang="ts">
/** The selection bar's "Invia via email" action and its confirmation, for the invoice list. */
import { Send } from 'lucide-vue-next'
import { useEmailStore } from '@/features/email/store/email'
import { useBulkInvoiceEmail } from '@/features/email/composables/useBulkInvoiceEmail'
import type { InvoiceSelection } from '@/features/invoice/composables/useInvoiceSelection'
import AppButton from '@/components/ui/AppButton.vue'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'

const props = defineProps<{ selection: InvoiceSelection }>()

const email = useEmailStore()
const { bulkEmailOpen, bulkEmailProgress, emailable, bulkEmailBlastRadius, bulkEmailMessage, runBulkEmail } = useBulkInvoiceEmail(props.selection)
</script>

<template>
  <span :title="email.ready ? undefined : 'Configura prima la casella in Impostazioni → Email'">
    <AppButton size="sm" :icon="Send" :disabled="!email.ready || emailable.length === 0" @click="bulkEmailOpen = true">Invia via email</AppButton>
  </span>
  <ConfirmDialog
    :open="bulkEmailOpen"
    title="Inviare le fatture ai pazienti?"
    :message="bulkEmailMessage"
    :blast-radius="bulkEmailBlastRadius"
    :confirm-label="`Invia ${emailable.length}`"
    tone="accent"
    :loading="bulkEmailProgress !== null"
    @confirm="runBulkEmail"
    @cancel="bulkEmailOpen = false"
  />
</template>

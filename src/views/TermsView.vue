<script setup lang="ts">
/**
 * The terms of use, before anything else: one box to accept them and a separate one for the
 * specific approval of the limitation clauses. Shown once per version of the text.
 */
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Check } from 'lucide-vue-next'
import BrandMark from '@/components/ui/BrandMark.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import TermsDocument from '@/components/legal/TermsDocument.vue'
import { TERMS_VERSION, specificApprovalText } from '@/legal/terms'
import { useTermsStore } from '@/stores/terms'
import { errorMessage } from '@/stores/toast'
import { formatDateLong } from '@/utils/format'

const router = useRouter()
const terms = useTermsStore()

const termsAccepted = ref(false)
const clausesApproved = ref(false)
const saving = ref(false)
const error = ref('')

const approvalText = specificApprovalText()
const complete = computed(() => termsAccepted.value && clausesApproved.value)

async function accept(): Promise<void> {
  saving.value = true
  error.value = ''
  try {
    await terms.accept({ termsAccepted: termsAccepted.value, clausesApproved: clausesApproved.value })
    await router.replace('/')
  } catch (failure) {
    error.value = errorMessage(failure)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="min-h-screen px-8 py-12 lg:px-16" data-tauri-drag-region>
    <div class="mx-auto max-w-[46rem]">
      <div class="flex items-center gap-2.5" data-tauri-drag-region>
        <BrandMark :size="30" />
        <span class="display text-xl text-text">PSI Fatture</span>
      </div>

      <h1 class="display settle mt-10 text-[2.25rem] leading-[1.1] text-text">Condizioni d’uso</h1>
      <p class="mt-3 text-md text-text-muted">Leggile prima di iniziare. Le accetti una volta per questa versione.</p>

      <AppCard class="settle mt-8 max-h-[46vh] overflow-y-auto" tabindex="0" aria-label="Testo delle condizioni d’uso" style="--settle: 1">
        <TermsDocument />
      </AppCard>

      <form class="mt-6 space-y-3" novalidate @submit.prevent="accept">
        <label class="flex items-start gap-3 text-base text-text">
          <input v-model="termsAccepted" type="checkbox" class="mt-1 shrink-0" />
          <span>Ho letto e accetto le condizioni d’uso.</span>
        </label>
        <label class="flex items-start gap-3 text-base text-text">
          <input v-model="clausesApproved" type="checkbox" class="mt-1 shrink-0" />
          <span>{{ approvalText }}</span>
        </label>

        <p v-if="error" class="rounded-control bg-danger-soft px-3 py-2 text-sm text-danger ring-1 ring-inset ring-danger-line" role="alert">{{ error }}</p>

        <div class="flex items-center gap-3 border-t border-border pt-6">
          <span class="text-sm text-text-subtle">Versione del {{ formatDateLong(TERMS_VERSION) }}</span>
          <div class="flex-1" />
          <AppButton type="submit" variant="primary" :icon="Check" :loading="saving" :disabled="!complete">Accetta e continua</AppButton>
        </div>
      </form>
    </div>
  </div>
</template>

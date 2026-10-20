<script setup lang="ts">
/** The terms of use the user accepted, to read again, with the date of the acceptance. */
import { computed } from 'vue'
import { Scale } from 'lucide-vue-next'
import AppCard from '@/components/ui/AppCard.vue'
import CardHeader from '@/components/ui/CardHeader.vue'
import TermsDocument from '@/features/terms/components/TermsDocument.vue'
import { TERMS_VERSION } from '@/features/terms/lib/terms'
import { useTermsStore } from '@/features/terms/store/terms'
import { formatDateLong, formatUtcTimestamp } from '@/common/lib/format'

const terms = useTermsStore()

const subtitle = computed(() =>
  terms.acceptance
    ? `Versione del ${formatDateLong(TERMS_VERSION)}, accettata il ${formatUtcTimestamp(terms.acceptance.accepted_at)}`
    : `Versione del ${formatDateLong(TERMS_VERSION)}`,
)
</script>

<template>
  <AppCard :padded="false" class="settle max-w-2xl">
    <CardHeader title="Condizioni d’uso" :subtitle="subtitle" :icon="Scale" />
    <div class="px-5 pb-5">
      <TermsDocument />
    </div>
  </AppCard>
</template>

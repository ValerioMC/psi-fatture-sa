<script setup lang="ts">
/** How invoices are made: tax regime, numbering and what the printed page shows. */
import AppCard from '@/components/ui/AppCard.vue'
import LetterheadPreview from '@/components/profile/LetterheadPreview.vue'
import ProfileSections from '@/components/profile/ProfileSections.vue'
import { PAGE_SECTIONS, useSettingsProfile } from '@/composables/useSettingsProfile'

const { form, errors, check } = useSettingsProfile()
</script>

<template>
  <div class="grid grid-cols-1 items-start gap-5 xl:grid-cols-[minmax(0,1fr)_20rem]">
    <AppCard class="settle min-w-0 px-8 py-4">
      <ProfileSections :form="form" :errors="errors" :sections="PAGE_SECTIONS.invoicing" @check="check" />
    </AppCard>
    <aside class="settle xl:sticky xl:top-32" style="--settle: 1">
      <LetterheadPreview :form="form" />
    </aside>
  </div>
</template>

<script setup lang="ts">
/** The invoice letterhead as it will print, redrawn live from the profile form. */
import { computed } from 'vue'
import { FileText } from 'lucide-vue-next'
import type { UpsertConfigInput } from '@/types'
import AppCard from '@/components/ui/AppCard.vue'
import CardHeader from '@/components/ui/CardHeader.vue'
import { TAX_REGIME_LABEL } from '@/utils/labels'

const props = defineProps<{ form: UpsertConfigInput }>()

const letterhead = computed(() => {
  const form = props.form
  const place = [form.zip_code, form.city].filter(Boolean).join(' ')
  return {
    name: [form.title, form.first_name, form.last_name].filter(Boolean).join(' ').trim() || 'Il tuo nome',
    profession: form.profession === 'psicoterapeuta' ? 'Psicoterapeuta' : 'Psicologo',
    lines: [
      form.address,
      place + (form.province ? ` (${form.province})` : ''),
      form.vat_number ? `P.IVA ${form.vat_number}` : '',
      form.fiscal_code ? `C.F. ${form.fiscal_code}` : '',
      form.albo_number ? `Albo ${form.albo_region ?? ''} n. ${form.albo_number}`.replace(/\s+/g, ' ') : '',
    ].filter((line) => line && line.trim() !== ''),
    regime: form.tax_regime ? TAX_REGIME_LABEL[form.tax_regime] : '',
  }
})
</script>

<template>
  <AppCard :padded="false">
    <CardHeader title="Intestazione in fattura" subtitle="Anteprima dal profilo" :icon="FileText" />
    <div class="px-5 pb-5">
      <div class="letterhead rounded-[10px] px-4 py-4">
        <p class="display text-lg leading-tight text-[#1d1b24]">{{ letterhead.name }}</p>
        <p class="mt-0.5 text-2xs font-medium tracking-[0.06em] text-[#3d35b0] uppercase">{{ letterhead.profession }}</p>
        <div class="my-3 h-px bg-[#e4e0d7]" />
        <p v-for="line in letterhead.lines" :key="line" class="truncate text-2xs leading-[1.1rem] text-[#56525e]">{{ line }}</p>
        <p v-if="letterhead.regime" class="mt-2 inline-block rounded-full bg-[#edeae3] px-2 text-[10px] leading-4 text-[#56525e]">{{ letterhead.regime }}</p>
      </div>
    </div>
  </AppCard>
</template>

<style scoped>
/* The letterhead is paper in both themes: it previews a printed page. */
.letterhead {
  background: #ffffff;
  box-shadow: 0 0 0 1px rgb(40 34 20 / 0.08), 0 6px 16px -8px rgb(40 34 20 / 0.25);
}
</style>

<script setup lang="ts">
/**
 * The professional profile's fields, grouped in the sections asked for.
 * Settings renders them all; onboarding renders a few per step.
 */
import type { TaxRegime, Profession, UpsertConfigInput } from '@/types'
import FormField from '@/components/ui/FormField.vue'
import FormSection from '@/components/ui/FormSection.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import type { SegmentOption } from '@/components/ui/types'
import type { ProfileField, ProfileSection } from '@/composables/useProfileForm'

defineProps<{
  form: UpsertConfigInput
  errors: Partial<Record<ProfileField, string>>
  sections: readonly ProfileSection[]
}>()

const emit = defineEmits<{ check: [field: ProfileField] }>()

const TITLES = ['Dott.ssa', 'Dott.', 'Dr.ssa', 'Dr.', 'Prof.ssa', 'Prof.']

const REGIMES: SegmentOption<TaxRegime>[] = [
  { value: 'forfettario', label: 'Forfettario' },
  { value: 'ordinario', label: 'Ordinario' },
]

const PROFESSIONS: SegmentOption<Profession>[] = [
  { value: 'psicologo', label: 'Psicologo/a' },
  { value: 'psicoterapeuta', label: 'Psicoterapeuta' },
]
</script>

<template>
  <div>
    <FormSection v-if="sections.includes('identity')" title="Chi sei" description="Il tuo nome e i dati fiscali, così come compaiono in testa a ogni fattura.">
      <div class="grid grid-cols-[7.5rem_1fr_1fr] gap-x-4 gap-y-5">
        <FormField v-slot="{ id }" label="Titolo">
          <select :id="id" v-model="form.title" class="field">
            <option v-for="title in TITLES" :key="title" :value="title">{{ title }}</option>
          </select>
        </FormField>
        <FormField v-slot="{ id, invalid, describedBy }" label="Nome" required :error="errors.first_name">
          <input :id="id" v-model="form.first_name" type="text" class="field" autocomplete="given-name" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="emit('check', 'first_name')" />
        </FormField>
        <FormField v-slot="{ id, invalid, describedBy }" label="Cognome" required :error="errors.last_name">
          <input :id="id" v-model="form.last_name" type="text" class="field" autocomplete="family-name" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="emit('check', 'last_name')" />
        </FormField>
      </div>
      <div class="mt-5 grid grid-cols-2 gap-x-4 gap-y-5">
        <FormField v-slot="{ id, invalid, describedBy }" label="Partita IVA" required :error="errors.vat_number">
          <input :id="id" v-model="form.vat_number" type="text" inputmode="numeric" maxlength="11" class="field field-mono" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="emit('check', 'vat_number')" />
        </FormField>
        <FormField v-slot="{ id, invalid, describedBy }" label="Codice fiscale" required :error="errors.fiscal_code">
          <input
            :id="id"
            v-model="form.fiscal_code"
            type="text"
            maxlength="16"
            spellcheck="false"
            class="field field-mono uppercase"
            :aria-invalid="invalid"
            :aria-describedby="describedBy"
            @input="form.fiscal_code = form.fiscal_code.toUpperCase()"
            @blur="emit('check', 'fiscal_code')"
          />
        </FormField>
      </div>
    </FormSection>

    <FormSection v-if="sections.includes('profession')" title="Professione" description="Compare sotto il tuo nome in fattura, con l'iscrizione all'albo.">
      <SegmentedControl v-model="form.profession" :options="PROFESSIONS" label="Professione" />
      <div class="mt-5 grid grid-cols-2 gap-x-4 gap-y-5">
        <FormField v-slot="{ id }" label="Iscrizione all'albo" optional>
          <input :id="id" v-model="form.albo_number" type="text" class="field tabular" placeholder="Numero" />
        </FormField>
        <FormField v-slot="{ id }" label="Ordine regionale" optional>
          <input :id="id" v-model="form.albo_region" type="text" class="field" placeholder="Lombardia" />
        </FormField>
      </div>
      <ToggleSwitch
        v-model="form.is_psicoanalista"
        class="mt-5"
        label="Sono psicoanalista, membro IPA"
        description="Aggiunge in fattura “Membro della International Psychoanalytical Association”."
      />
      <FormField
        v-slot="{ id }"
        class="mt-5 max-w-96"
        label="Specializzazione personalizzata"
        optional
        hint="Alternativa al check IPA: un testo libero mostrato in fattura al posto della qualifica standard."
      >
        <input :id="id" v-model="form.specialization" type="text" class="field" :disabled="form.is_psicoanalista" placeholder="Es. Specialista in terapia EMDR" />
      </FormField>
    </FormSection>

    <FormSection v-if="sections.includes('tax')" title="Regime fiscale" description="Decide cosa compare in fattura: IVA, ritenuta d'acconto, marca da bollo e diciture di legge.">
      <SegmentedControl v-model="form.tax_regime" :options="REGIMES" label="Regime fiscale" />
      <FormField
        v-slot="{ id, invalid, describedBy }"
        class="mt-5 max-w-56"
        label="Coefficiente di redditività"
        :hint="form.tax_regime === 'forfettario' ? 'Per gli psicologi, codice ATECO 86.90.30: 78%.' : 'Quota dei compensi considerata reddito nella stima.'"
        :error="errors.coefficient"
      >
        <div class="relative">
          <input :id="id" v-model.number="form.coefficient" type="number" min="1" max="100" class="field tabular pr-8" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="emit('check', 'coefficient')" />
          <span class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-sm text-text-subtle">%</span>
        </div>
      </FormField>
    </FormSection>

    <FormSection v-if="sections.includes('numbering')" title="Numerazione" description="Se usi l'app da metà anno, riparti dal numero successivo all'ultima fattura emessa.">
      <FormField
        v-slot="{ id, invalid, describedBy }"
        class="max-w-56"
        label="Primo numero dell'anno"
        hint="Ultima fattura n. 23? Scrivi 24. Si parte da zero? Lascia 1."
        :error="errors.initial_invoice_number"
      >
        <input :id="id" v-model.number="form.initial_invoice_number" type="number" min="1" class="field tabular" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="emit('check', 'initial_invoice_number')" />
      </FormField>
    </FormSection>

    <FormSection v-if="sections.includes('invoice')" title="Fattura" description="Cosa compare nel documento stampato.">
      <ToggleSwitch
        v-model="form.hide_quantity_in_invoice"
        label="Non mostrare la quantità in fattura"
        description="Nasconde le colonne Quantità e Prezzo unitario; resta solo l'importo per riga."
      />
    </FormSection>

    <FormSection v-if="sections.includes('studio')" title="Studio" description="L'indirizzo del tuo domicilio fiscale o dello studio, stampato in fattura.">
      <div class="grid grid-cols-[1fr_5rem_6.5rem] gap-x-4 gap-y-5">
        <FormField v-slot="{ id, invalid, describedBy }" class="col-span-3" label="Via e numero civico" required :error="errors.address">
          <input :id="id" v-model="form.address" type="text" class="field" autocomplete="street-address" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="emit('check', 'address')" />
        </FormField>
        <FormField v-slot="{ id, invalid, describedBy }" label="Città" required :error="errors.city">
          <input :id="id" v-model="form.city" type="text" class="field" autocomplete="address-level2" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="emit('check', 'city')" />
        </FormField>
        <FormField v-slot="{ id, invalid, describedBy }" label="Prov." required :error="errors.province">
          <input :id="id" v-model="form.province" type="text" maxlength="2" class="field uppercase" :aria-invalid="invalid" :aria-describedby="describedBy" @input="form.province = form.province.toUpperCase()" @blur="emit('check', 'province')" />
        </FormField>
        <FormField v-slot="{ id, invalid, describedBy }" label="CAP" required :error="errors.zip_code">
          <input :id="id" v-model="form.zip_code" type="text" inputmode="numeric" maxlength="5" class="field tabular" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="emit('check', 'zip_code')" />
        </FormField>
      </div>
    </FormSection>

    <FormSection v-if="sections.includes('payment')" title="Contatti e pagamento" description="L'IBAN compare in fattura quando il paziente paga con bonifico.">
      <div class="grid grid-cols-2 gap-x-4 gap-y-5">
        <FormField v-slot="{ id }" label="Telefono" optional>
          <input :id="id" v-model="form.phone" type="tel" class="field tabular" autocomplete="tel" />
        </FormField>
        <FormField v-slot="{ id, invalid, describedBy }" label="PEC" optional :error="errors.pec_email">
          <input :id="id" v-model="form.pec_email" type="email" class="field" :aria-invalid="invalid" :aria-describedby="describedBy" @blur="emit('check', 'pec_email')" />
        </FormField>
        <FormField v-slot="{ id, invalid, describedBy }" class="col-span-2" label="IBAN" optional :error="errors.iban">
          <input
            :id="id"
            v-model="form.iban"
            type="text"
            spellcheck="false"
            class="field field-mono uppercase"
            :aria-invalid="invalid"
            :aria-describedby="describedBy"
            @input="form.iban = form.iban.toUpperCase()"
            @blur="emit('check', 'iban')"
          />
        </FormField>
      </div>
    </FormSection>
  </div>
</template>

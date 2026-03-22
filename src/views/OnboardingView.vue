<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { FileText } from 'lucide-vue-next'
import { useConfigStore } from '@/stores/config'
import type { UpsertConfigInput } from '@/types'

const router = useRouter()
const configStore = useConfigStore()

const saving = ref(false)
const error = ref<string | null>(null)

const form = reactive<UpsertConfigInput>({
  title: 'Dott.',
  first_name: '',
  last_name: '',
  vat_number: '',
  fiscal_code: '',
  tax_regime: 'forfettario',
  albo_number: '',
  albo_region: '',
  address: '',
  city: '',
  province: '',
  zip_code: '',
  country: 'IT',
  phone: '',
  pec_email: '',
  iban: '',
  coefficient: 78,
  is_psicoanalista: false,
  initial_invoice_number: 1,
})

const TITLE_OPTIONS = ['Dott.', 'Dott.ssa', 'Dr.', 'Dr.ssa', 'Prof.', 'Prof.ssa']

const TAX_REGIME_OPTIONS = [
  { value: 'forfettario', label: 'Regime Forfettario' },
  { value: 'ordinario', label: 'Regime Ordinario' },
]

async function onSubmit() {
  saving.value = true
  error.value = null
  try {
    await configStore.saveConfig(form)
    router.push('/dashboard')
  } catch (e) {
    error.value = String(e)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="min-h-screen" style="background: linear-gradient(135deg, #f8fafc 0%, #eef2ff 100%)">
    <div class="flex items-start justify-center py-12 px-4">
      <div class="w-full max-w-2xl animate-in">
        <!-- Welcome header -->
        <div class="text-center mb-8">
          <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl mb-4"
            style="background: linear-gradient(135deg, #1e1b4b, #4338ca)">
            <FileText class="w-8 h-8 text-white" />
          </div>
          <h1 class="text-3xl font-bold heading-serif gradient-text">Benvenuto in PSI Fatture</h1>
          <p class="text-sage-600 mt-2 text-sm">
            Configura il tuo profilo professionale per iniziare a emettere fatture.
          </p>
        </div>

        <form class="glass-card rounded-2xl p-8 space-y-6 shadow-xl" @submit.prevent="onSubmit">
          <!-- Personal info -->
          <div>
            <h2 class="text-sm font-semibold text-sage-700 uppercase tracking-wider mb-4">Dati personali</h2>
            <div class="grid grid-cols-3 gap-4">
              <div>
                <label class="text-sm font-medium text-sage-700 block mb-1">Titolo</label>
                <select
                  v-model="form.title"
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                >
                  <option v-for="t in TITLE_OPTIONS" :key="t" :value="t">{{ t }}</option>
                </select>
              </div>
              <div>
                <label class="text-sm font-medium text-sage-700 block mb-1">Nome</label>
                <input
                  v-model="form.first_name"
                  type="text"
                  required
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                />
              </div>
              <div>
                <label class="text-sm font-medium text-sage-700 block mb-1">Cognome</label>
                <input
                  v-model="form.last_name"
                  type="text"
                  required
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                />
              </div>
            </div>
            <div class="grid grid-cols-2 gap-4 mt-4">
              <div>
                <label class="text-sm font-medium text-sage-700 block mb-1">Partita IVA</label>
                <input
                  v-model="form.vat_number"
                  type="text"
                  required
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                />
              </div>
              <div>
                <label class="text-sm font-medium text-sage-700 block mb-1">Codice Fiscale</label>
                <input
                  v-model="form.fiscal_code"
                  type="text"
                  required
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                />
              </div>
            </div>
          </div>

          <hr class="border-sage-100" />

          <!-- Tax regime -->
          <div>
            <h2 class="text-sm font-semibold text-sage-700 uppercase tracking-wider mb-4">Regime fiscale</h2>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="text-sm font-medium text-sage-700 block mb-1">Regime</label>
                <select
                  v-model="form.tax_regime"
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                >
                  <option v-for="opt in TAX_REGIME_OPTIONS" :key="opt.value" :value="opt.value">
                    {{ opt.label }}
                  </option>
                </select>
              </div>
              <div>
                <label class="text-sm font-medium text-sage-700 block mb-1">Coefficiente (%)</label>
                <input
                  v-model.number="form.coefficient"
                  type="number"
                  min="1"
                  max="100"
                  required
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                />
              </div>
            </div>
            <div class="mt-3">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="form.is_psicoanalista"
                  type="checkbox"
                  class="rounded border-sage-300 text-sage-600 focus:ring-sage-400"
                />
                <span class="text-sm text-sage-700">Sono iscritto all'albo degli psicoanalisti (ENPAP non applicabile)</span>
              </label>
            </div>
          </div>

          <hr class="border-sage-100" />

          <!-- Albo -->
          <div>
            <h2 class="text-sm font-semibold text-sage-700 uppercase tracking-wider mb-4">Albo professionale</h2>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="text-sm font-medium text-sage-700 block mb-1">Numero iscrizione</label>
                <input
                  v-model="form.albo_number"
                  type="text"
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                />
              </div>
              <div>
                <label class="text-sm font-medium text-sage-700 block mb-1">Regione albo</label>
                <input
                  v-model="form.albo_region"
                  type="text"
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                />
              </div>
            </div>
          </div>

          <hr class="border-sage-100" />

          <!-- Address -->
          <div>
            <h2 class="text-sm font-semibold text-sage-700 uppercase tracking-wider mb-4">Indirizzo studio</h2>
            <div class="space-y-4">
              <div>
                <label class="text-sm font-medium text-sage-700 block mb-1">Indirizzo</label>
                <input
                  v-model="form.address"
                  type="text"
                  required
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                />
              </div>
              <div class="grid grid-cols-3 gap-4">
                <div class="col-span-1">
                  <label class="text-sm font-medium text-sage-700 block mb-1">Città</label>
                  <input
                    v-model="form.city"
                    type="text"
                    required
                    class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                  />
                </div>
                <div>
                  <label class="text-sm font-medium text-sage-700 block mb-1">Provincia</label>
                  <input
                    v-model="form.province"
                    type="text"
                    maxlength="2"
                    required
                    class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80 uppercase"
                    @input="form.province = form.province.toUpperCase()"
                  />
                </div>
                <div>
                  <label class="text-sm font-medium text-sage-700 block mb-1">CAP</label>
                  <input
                    v-model="form.zip_code"
                    type="text"
                    maxlength="5"
                    pattern="\d{5}"
                    required
                    class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                  />
                </div>
              </div>
            </div>
          </div>

          <hr class="border-sage-100" />

          <!-- Contacts -->
          <div>
            <h2 class="text-sm font-semibold text-sage-700 uppercase tracking-wider mb-4">Contatti e pagamento</h2>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="text-sm font-medium text-sage-700 block mb-1">Telefono</label>
                <input
                  v-model="form.phone"
                  type="tel"
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                />
              </div>
              <div>
                <label class="text-sm font-medium text-sage-700 block mb-1">PEC</label>
                <input
                  v-model="form.pec_email"
                  type="email"
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                />
              </div>
              <div class="col-span-2">
                <label class="text-sm font-medium text-sage-700 block mb-1">IBAN</label>
                <input
                  v-model="form.iban"
                  type="text"
                  class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
                />
              </div>
            </div>
          </div>

          <!-- Error -->
          <div v-if="error" class="rounded-lg bg-red-50 border border-red-200 px-4 py-3 text-sm text-red-700">
            {{ error }}
          </div>

          <div class="flex justify-end pt-2">
            <button
              type="submit"
              :disabled="saving"
              class="group relative overflow-hidden text-white font-semibold px-6 py-2.5 rounded-xl text-sm flex items-center gap-2 transition-all duration-200 disabled:opacity-60 cursor-pointer focus:outline-none"
              style="background: linear-gradient(135deg, #1e1b4b, #4338ca); box-shadow: 0 4px 20px rgba(67, 56, 202, 0.4);"
            >
              <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 transform -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700" aria-hidden="true" />
              <span class="relative z-10">{{ saving ? 'Salvataggio...' : 'Inizia a usare PSI Fatture →' }}</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

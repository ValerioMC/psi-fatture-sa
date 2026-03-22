<script setup lang="ts">
import { reactive, ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useClientsStore } from '@/stores/clients'
import { getClient } from '@/api'
import type { CreateClientInput, UpdateClientInput } from '@/types'
import { User, Building2, MapPin, Phone, ShieldCheck, ArrowLeft, Check, X } from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()
const clientsStore = useClientsStore()

const editId = computed(() =>
  route.params.id ? Number(route.params.id) : null,
)
const isEdit = computed(() => editId.value !== null)
const loading = ref(false)
const saving = ref(false)
const error = ref<string | null>(null)

const form = reactive<CreateClientInput>({
  client_type: 'persona_fisica',
  first_name: '',
  last_name: '',
  birth_date: undefined,
  gender: undefined,
  fiscal_code: '',
  vat_number: undefined,
  address: '',
  city: '',
  province: '',
  zip_code: '',
  email: undefined,
  phone: '',
  notes: undefined,
  sts_authorization: false,
})

const patientDisplayName = computed(() => {
  if (!isEdit.value) return null
  const parts = [form.last_name, form.first_name].filter(Boolean)
  return parts.length > 0 ? parts.join(' ') : null
})

onMounted(async () => {
  if (!isEdit.value) return
  loading.value = true
  try {
    const client = await getClient(editId.value!)
    Object.assign(form, {
      client_type: client.client_type,
      first_name: client.first_name,
      last_name: client.last_name,
      birth_date: client.birth_date,
      gender: client.gender,
      fiscal_code: client.fiscal_code,
      vat_number: client.vat_number,
      address: client.address,
      city: client.city,
      province: client.province,
      zip_code: client.zip_code,
      email: client.email,
      phone: client.phone,
      notes: client.notes,
      sts_authorization: client.sts_authorization,
    })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})

async function onSubmit() {
  saving.value = true
  error.value = null
  try {
    if (isEdit.value) {
      const input: UpdateClientInput = { id: editId.value!, ...form }
      await clientsStore.editClient(input)
    } else {
      await clientsStore.addClient(form)
    }
    router.push('/clients')
  } catch (e) {
    error.value = String(e)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="p-8">
    <div class="max-w-2xl mx-auto">
      <!-- Breadcrumb bar -->
      <div class="flex items-center gap-2 mb-6 animate-in">
        <button
          type="button"
          class="flex items-center gap-1.5 text-sm text-sage-500 hover:text-sage-800 transition-colors cursor-pointer group"
          @click="router.push('/clients')"
        >
          <ArrowLeft class="w-4 h-4 transition-transform group-hover:-translate-x-0.5" />
          <span>Pazienti</span>
        </button>
        <template v-if="patientDisplayName">
          <span class="text-sage-300 text-sm">/</span>
          <span class="text-sm font-semibold text-sage-800 truncate max-w-[220px]">{{ patientDisplayName }}</span>
        </template>
      </div>

      <!-- Loading -->
      <div v-if="loading" class="flex flex-col items-center justify-center py-20 gap-3">
        <div class="w-8 h-8 rounded-full border-2 border-sage-200 border-t-sage-500 animate-spin" />
        <p class="text-sm text-sage-400">Caricamento dati…</p>
      </div>

      <form v-else class="space-y-5" @submit.prevent="onSubmit">

        <!-- Tipo cliente: segment control -->
        <div class="glass-card rounded-2xl p-5 animate-in">
          <div class="flex items-center gap-2.5 mb-4">
            <div class="w-7 h-7 rounded-lg flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #5d8062, #48654c)">
              <User class="w-3.5 h-3.5 text-white" />
            </div>
            <h2 class="text-xs font-semibold text-sage-500 uppercase tracking-wider">Tipo cliente</h2>
          </div>

          <div class="flex rounded-xl border border-sage-200 p-1 bg-sage-50/50 gap-1">
            <button
              type="button"
              class="flex-1 flex items-center justify-center gap-2 py-2 px-4 text-sm font-medium rounded-lg transition-all duration-200 cursor-pointer"
              :class="form.client_type === 'persona_fisica'
                ? 'bg-white shadow-sm text-sage-900'
                : 'text-sage-500 hover:text-sage-700'"
              @click="form.client_type = 'persona_fisica'"
            >
              <User class="w-4 h-4" />
              Persona Fisica
            </button>
            <button
              type="button"
              class="flex-1 flex items-center justify-center gap-2 py-2 px-4 text-sm font-medium rounded-lg transition-all duration-200 cursor-pointer"
              :class="form.client_type === 'azienda'
                ? 'bg-white shadow-sm text-sage-900'
                : 'text-sage-500 hover:text-sage-700'"
              @click="form.client_type = 'azienda'"
            >
              <Building2 class="w-4 h-4" />
              Azienda
            </button>
          </div>
        </div>

        <!-- Dati anagrafici -->
        <div class="glass-card rounded-2xl p-5 animate-in-d1">
          <div class="flex items-center gap-2.5 mb-4">
            <div class="w-7 h-7 rounded-lg flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #0c8aeb, #0153a2)">
              <User class="w-3.5 h-3.5 text-white" />
            </div>
            <h2 class="text-xs font-semibold text-sage-500 uppercase tracking-wider">Dati anagrafici</h2>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">
                {{ form.client_type === 'azienda' ? 'Ragione Sociale' : 'Nome' }}
              </label>
              <input
                v-model="form.first_name"
                type="text"
                :required="form.client_type === 'persona_fisica'"
                :placeholder="form.client_type === 'azienda' ? 'Studio Medico…' : 'Mario'"
                class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 placeholder-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
              />
            </div>
            <div>
              <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Cognome</label>
              <input
                v-model="form.last_name"
                type="text"
                required
                placeholder="Rossi"
                class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 placeholder-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
              />
            </div>

            <template v-if="form.client_type === 'persona_fisica'">
              <div>
                <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Data di nascita</label>
                <input
                  v-model="form.birth_date"
                  type="date"
                  class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
                />
              </div>
              <div>
                <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Sesso</label>
                <select
                  v-model="form.gender"
                  class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow appearance-none cursor-pointer"
                >
                  <option value="">— Seleziona —</option>
                  <option value="M">Maschio</option>
                  <option value="F">Femmina</option>
                </select>
              </div>
            </template>

            <div>
              <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Codice Fiscale</label>
              <input
                v-model="form.fiscal_code"
                type="text"
                required
                placeholder="RSSMRA80A01H501U"
                class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 placeholder-sage-300 font-mono uppercase focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
                @input="form.fiscal_code = form.fiscal_code.toUpperCase()"
              />
            </div>
            <div v-if="form.client_type === 'azienda'">
              <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Partita IVA</label>
              <input
                v-model="form.vat_number"
                type="text"
                placeholder="01234567890"
                class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 placeholder-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
              />
            </div>
          </div>
        </div>

        <!-- Indirizzo -->
        <div class="glass-card rounded-2xl p-5 animate-in-d2">
          <div class="flex items-center gap-2.5 mb-4">
            <div class="w-7 h-7 rounded-lg flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #b88e67, #8a5f42)">
              <MapPin class="w-3.5 h-3.5 text-white" />
            </div>
            <h2 class="text-xs font-semibold text-sage-500 uppercase tracking-wider">Indirizzo</h2>
          </div>

          <div class="space-y-3">
            <div>
              <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Via / Indirizzo</label>
              <input
                v-model="form.address"
                type="text"
                placeholder="Via Roma 1"
                class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 placeholder-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
              />
            </div>
            <div class="grid grid-cols-5 gap-3">
              <div class="col-span-3">
                <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Città</label>
                <input
                  v-model="form.city"
                  type="text"
                  placeholder="Roma"
                  class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 placeholder-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
                />
              </div>
              <div>
                <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Prov.</label>
                <input
                  v-model="form.province"
                  type="text"
                  maxlength="2"
                  placeholder="RM"
                  class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 placeholder-sage-300 uppercase text-center focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
                  @input="form.province = form.province.toUpperCase()"
                />
              </div>
              <div>
                <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">CAP</label>
                <input
                  v-model="form.zip_code"
                  type="text"
                  maxlength="5"
                  pattern="\d{5}"
                  placeholder="00100"
                  class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 placeholder-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
                />
              </div>
            </div>
          </div>
        </div>

        <!-- Contatti -->
        <div class="glass-card rounded-2xl p-5 animate-in-d3">
          <div class="flex items-center gap-2.5 mb-4">
            <div class="w-7 h-7 rounded-lg flex items-center justify-center shrink-0" style="background: linear-gradient(135deg, #d4a017, #a16207)">
              <Phone class="w-3.5 h-3.5 text-white" />
            </div>
            <h2 class="text-xs font-semibold text-sage-500 uppercase tracking-wider">Contatti</h2>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Email</label>
              <input
                v-model="form.email"
                type="email"
                placeholder="mario.rossi@email.it"
                class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 placeholder-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
              />
            </div>
            <div>
              <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Telefono</label>
              <input
                v-model="form.phone"
                type="tel"
                placeholder="+39 06 1234567"
                class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 placeholder-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow"
              />
            </div>
          </div>

          <div class="mt-4">
            <label class="text-xs font-semibold text-sage-600 uppercase tracking-wider block mb-1.5">Note</label>
            <textarea
              v-model="form.notes"
              rows="3"
              placeholder="Note aggiuntive sul paziente…"
              class="w-full border border-sage-200 rounded-xl px-3.5 py-2.5 text-sm text-sage-900 placeholder-sage-300 focus:outline-none focus:ring-2 focus:ring-sage-400/40 focus:border-sage-400 bg-white/80 transition-shadow resize-none"
            />
          </div>

          <!-- STS toggle -->
          <label class="mt-4 flex items-center gap-3 cursor-pointer group/sts p-3.5 rounded-xl border border-sage-100/60 hover:bg-sage-50/60 transition-colors">
            <div
              class="w-9 h-5 rounded-full transition-colors duration-200 relative shrink-0"
              :class="form.sts_authorization ? 'bg-amber-500' : 'bg-sage-200'"
            >
              <div
                class="absolute top-[3px] w-3.5 h-3.5 rounded-full bg-white shadow-sm transition-transform duration-200"
                :class="form.sts_authorization ? 'translate-x-[17px]' : 'translate-x-[3px]'"
              />
            </div>
            <input v-model="form.sts_authorization" type="checkbox" class="sr-only" />
            <div class="flex items-center gap-2 flex-1">
              <ShieldCheck
                class="w-4 h-4 transition-colors"
                :class="form.sts_authorization ? 'text-amber-500' : 'text-sage-300'"
              />
              <div>
                <p class="text-sm font-medium text-sage-800">Autorizzazione STS</p>
                <p class="text-xs text-sage-400">Consenso all'invio dati al Sistema Tessera Sanitaria</p>
              </div>
            </div>
            <span
              v-if="form.sts_authorization"
              class="text-[10px] font-semibold px-2 py-0.5 rounded-full bg-amber-50 text-amber-600 border border-amber-100 shrink-0"
            >
              Autorizzato
            </span>
          </label>
        </div>

        <!-- Error + Submit card -->
        <div class="glass-card rounded-2xl p-5 animate-in-d4">
          <div v-if="error" class="rounded-xl bg-red-50 border border-red-200 px-4 py-3 text-sm text-red-700 flex items-center gap-2 mb-4">
            <X class="w-4 h-4 shrink-0 text-red-400" />
            {{ error }}
          </div>
          <div class="flex justify-between items-center">
            <button
              type="button"
              class="flex items-center gap-1.5 text-sm text-sage-500 hover:text-sage-700 transition-colors cursor-pointer group"
              @click="router.push('/clients')"
            >
              <ArrowLeft class="w-4 h-4 transition-transform group-hover:-translate-x-0.5" />
              Annulla
            </button>
            <button
              type="submit"
              :disabled="saving"
              class="group relative overflow-hidden text-white font-semibold px-6 py-2.5 rounded-xl text-sm flex items-center gap-2 transition-all duration-200 disabled:opacity-60 cursor-pointer focus:outline-none"
              style="background: linear-gradient(135deg, #5d8062, #0c8aeb); box-shadow: 0 4px 14px rgba(93,128,98,0.3);"
            >
              <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 transform -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700" aria-hidden="true" />
              <Check class="w-4 h-4 relative z-10" />
              <span class="relative z-10">{{ saving ? 'Salvataggio…' : isEdit ? 'Aggiorna paziente' : 'Crea paziente' }}</span>
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
</template>

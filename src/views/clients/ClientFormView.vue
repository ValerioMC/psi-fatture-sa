<script setup lang="ts">
import { reactive, ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useClientsStore } from '@/stores/clients'
import { getClient } from '@/api'
import PageHeader from '@/components/ui/PageHeader.vue'
import type { CreateClientInput, UpdateClientInput } from '@/types'

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
    <PageHeader
      :title="isEdit ? 'Modifica Cliente' : 'Nuovo Cliente'"
      :subtitle="isEdit ? 'Aggiorna i dati del cliente.' : 'Aggiungi un nuovo paziente o cliente.'"
    />

    <div v-if="loading" class="text-sm text-sage-400">Caricamento...</div>

    <form v-else class="space-y-6" @submit.prevent="onSubmit">
      <!-- Type -->
      <div class="glass-card rounded-xl p-6 animate-in">
        <h2 class="text-sm font-semibold text-sage-700 uppercase tracking-wider mb-4">Tipo cliente</h2>
        <div class="flex gap-6">
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              v-model="form.client_type"
              type="radio"
              value="persona_fisica"
              class="text-sage-600 focus:ring-sage-400"
            />
            <span class="text-sm text-sage-700">Persona Fisica</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              v-model="form.client_type"
              type="radio"
              value="azienda"
              class="text-sage-600 focus:ring-sage-400"
            />
            <span class="text-sm text-sage-700">Azienda</span>
          </label>
        </div>
      </div>

      <!-- Anagrafica -->
      <div class="glass-card rounded-xl p-6 animate-in-d1">
        <h2 class="text-sm font-semibold text-sage-700 uppercase tracking-wider mb-4">Dati anagrafici</h2>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="text-sm font-medium text-sage-700 block mb-1">
              {{ form.client_type === 'azienda' ? 'Ragione Sociale' : 'Nome' }}
            </label>
            <input
              v-model="form.first_name"
              type="text"
              :required="form.client_type === 'persona_fisica'"
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

          <template v-if="form.client_type === 'persona_fisica'">
            <div>
              <label class="text-sm font-medium text-sage-700 block mb-1">Data di nascita</label>
              <input
                v-model="form.birth_date"
                type="date"
                class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
              />
            </div>
            <div>
              <label class="text-sm font-medium text-sage-700 block mb-1">Sesso</label>
              <select
                v-model="form.gender"
                class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
              >
                <option value="">— Seleziona —</option>
                <option value="M">Maschio</option>
                <option value="F">Femmina</option>
              </select>
            </div>
          </template>

          <div>
            <label class="text-sm font-medium text-sage-700 block mb-1">Codice Fiscale</label>
            <input
              v-model="form.fiscal_code"
              type="text"
              required
              class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80 uppercase"
              @input="form.fiscal_code = form.fiscal_code.toUpperCase()"
            />
          </div>
          <div v-if="form.client_type === 'azienda'">
            <label class="text-sm font-medium text-sage-700 block mb-1">Partita IVA</label>
            <input
              v-model="form.vat_number"
              type="text"
              class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
            />
          </div>
        </div>
      </div>

      <!-- Address -->
      <div class="glass-card rounded-xl p-6 animate-in-d2">
        <h2 class="text-sm font-semibold text-sage-700 uppercase tracking-wider mb-4">Indirizzo</h2>
        <div class="space-y-4">
          <div>
            <label class="text-sm font-medium text-sage-700 block mb-1">Via / Indirizzo</label>
            <input
              v-model="form.address"
              type="text"
              class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
            />
          </div>
          <div class="grid grid-cols-3 gap-4">
            <div class="col-span-1">
              <label class="text-sm font-medium text-sage-700 block mb-1">Città</label>
              <input
                v-model="form.city"
                type="text"
                class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
              />
            </div>
            <div>
              <label class="text-sm font-medium text-sage-700 block mb-1">Provincia</label>
              <input
                v-model="form.province"
                type="text"
                maxlength="2"
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
                class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Contacts -->
      <div class="glass-card rounded-xl p-6 animate-in-d3">
        <h2 class="text-sm font-semibold text-sage-700 uppercase tracking-wider mb-4">Contatti</h2>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="text-sm font-medium text-sage-700 block mb-1">Email</label>
            <input
              v-model="form.email"
              type="email"
              class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
            />
          </div>
          <div>
            <label class="text-sm font-medium text-sage-700 block mb-1">Telefono</label>
            <input
              v-model="form.phone"
              type="tel"
              class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80"
            />
          </div>
        </div>
        <div class="mt-4">
          <label class="text-sm font-medium text-sage-700 block mb-1">Note</label>
          <textarea
            v-model="form.notes"
            rows="3"
            class="w-full border border-sage-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-sage-400 bg-white/80 resize-none"
          />
        </div>
        <div class="mt-3">
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              v-model="form.sts_authorization"
              type="checkbox"
              class="rounded border-sage-300 text-sage-600 focus:ring-sage-400"
            />
            <span class="text-sm text-sage-700">Autorizzazione STS (invio dati al sistema TS)</span>
          </label>
        </div>
      </div>

      <div v-if="error" class="rounded-lg bg-red-50 border border-red-200 px-4 py-3 text-sm text-red-700">
        {{ error }}
      </div>

      <div class="flex justify-end gap-3 pb-4">
        <button
          type="button"
          class="border border-sage-200 text-sage-700 hover:bg-sage-50 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
          @click="router.push('/clients')"
        >
          Annulla
        </button>
        <button
          type="submit"
          :disabled="saving"
          class="bg-gradient-to-r from-sage-600 to-ocean-500 text-white hover:from-sage-700 hover:to-ocean-600 px-4 py-2 rounded-lg text-sm font-medium transition-all disabled:opacity-60"
        >
          {{ saving ? 'Salvataggio...' : isEdit ? 'Aggiorna cliente' : 'Crea cliente' }}
        </button>
      </div>
    </form>
    </div>
  </div>
</template>

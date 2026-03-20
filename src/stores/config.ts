import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { getConfig, upsertConfig } from '@/api'
import type { ProfessionalConfig, UpsertConfigInput } from '@/types'

export const useConfigStore = defineStore('config', () => {
  const config = ref<ProfessionalConfig | null>(null)
  const loading = ref(false)

  const isConfigured = computed(
    () => config.value !== null && config.value.first_name.length > 0,
  )

  const fullName = computed(() => {
    if (!config.value) return ''
    return `${config.value.title} ${config.value.first_name} ${config.value.last_name}`.trim()
  })

  async function loadConfig() {
    loading.value = true
    try {
      config.value = await getConfig()
    } finally {
      loading.value = false
    }
  }

  async function saveConfig(input: UpsertConfigInput) {
    config.value = await upsertConfig(input)
  }

  return { config, loading, isConfigured, fullName, loadConfig, saveConfig }
})

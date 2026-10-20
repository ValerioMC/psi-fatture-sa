/** The professional profile. */
import { invoke } from '@tauri-apps/api/core'
import type { ProfessionalConfig, UpsertConfigInput } from '@/features/settings/model/config'

export const getConfig = () => invoke<ProfessionalConfig | null>('get_config')
export const upsertConfig = (input: UpsertConfigInput) =>
  invoke<ProfessionalConfig>('upsert_config', { input })

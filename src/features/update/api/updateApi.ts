/** The app version and its updates. */
import { invoke } from '@tauri-apps/api/core'
import type { UpdateCheck } from '@/features/update/model/update'

export const getAppVersion = () => invoke<string>('get_app_version')
export const checkForUpdate = () => invoke<UpdateCheck>('check_for_update')
/** Mac only: installs the newest release over this app and restarts, so on success it never resolves. */
export const installUpdate = () => invoke<void>('install_update')

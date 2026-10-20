/** Backups of the whole archive. */
import { invoke } from '@tauri-apps/api/core'
import type { BackupFile, BackupOverview } from '@/features/settings/model/backup'

export const exportBackup = (path: string) => invoke<void>('export_backup', { path })
export const getBackupFileName = () => invoke<string>('get_backup_file_name')
export const getBackupOverview = () => invoke<BackupOverview>('get_backup_overview')
export const createBackup = () => invoke<BackupFile>('create_backup')
export const revealBackupsFolder = () => invoke<void>('reveal_backups_folder')

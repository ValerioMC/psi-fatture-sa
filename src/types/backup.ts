export type BackupReason = 'auto' | 'manual'

export interface BackupFile {
  name: string
  path: string
  size_bytes: number
  /** Local time, "YYYY-MM-DD HH:MM:SS". */
  created_at: string
  reason: BackupReason
}

export interface BackupOverview {
  database_path: string
  backups_folder: string
  keep: number
  /** Newest first. */
  backups: BackupFile[]
  last_failure: string | null
}

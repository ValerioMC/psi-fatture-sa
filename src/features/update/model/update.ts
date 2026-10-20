export type Platform = 'mac_os_arm64' | 'mac_os_x64' | 'windows_x64' | 'other'

export interface UpdateCheck {
  current_version: string
  latest_version: string
  update_available: boolean
  /** The bundle for this system, or the release page when none is published for it. */
  download_url: string
  platform: Platform
  platform_label: string
}

/** The yearly dashboard figures. */
import { invoke } from '@tauri-apps/api/core'
import type { DashboardData } from '@/features/dashboard/model/dashboard'

export const getDashboard = (year: number) => invoke<DashboardData>('get_dashboard', { year })

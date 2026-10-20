import type { Invoice } from '@/features/invoice/model/invoice'

export interface MonthlyRevenue {
  month: number
  month_name: string
  revenue: number
  invoice_count: number
}

export interface DashboardData {
  year: number
  total_revenue: number
  total_net_revenue: number
  paid_revenue: number
  unpaid_revenue: number
  total_invoices: number
  paid_invoices: number
  draft_invoices: number
  monthly_revenue: MonthlyRevenue[]
  recent_invoices: Invoice[]
}

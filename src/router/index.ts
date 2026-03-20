import { createRouter, createWebHashHistory } from 'vue-router'
import { useConfigStore } from '@/stores/config'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      component: () => import('@/components/layout/AppLayout.vue'),
      children: [
        { path: '', redirect: '/dashboard' },
        { path: 'dashboard', name: 'dashboard', component: () => import('@/views/DashboardView.vue') },
        { path: 'clients', name: 'clients', component: () => import('@/views/clients/ClientListView.vue') },
        { path: 'clients/new', name: 'clients.new', component: () => import('@/views/clients/ClientFormView.vue') },
        { path: 'clients/:id/edit', name: 'clients.edit', component: () => import('@/views/clients/ClientFormView.vue') },
        { path: 'services', name: 'services', component: () => import('@/views/services/ServiceListView.vue') },
        { path: 'invoices', name: 'invoices', component: () => import('@/views/invoices/InvoiceListView.vue') },
        { path: 'invoices/new', name: 'invoices.new', component: () => import('@/views/invoices/InvoiceFormView.vue') },
        { path: 'invoices/:id/edit', name: 'invoices.edit', component: () => import('@/views/invoices/InvoiceFormView.vue') },
        { path: 'invoices/:id', name: 'invoices.detail', component: () => import('@/views/invoices/InvoiceDetailView.vue') },
        { path: 'agenda', name: 'agenda', component: () => import('@/views/agenda/AgendaView.vue') },
        { path: 'settings', name: 'settings', component: () => import('@/views/SettingsView.vue') },
      ],
    },
    {
      path: '/onboarding',
      name: 'onboarding',
      component: () => import('@/views/OnboardingView.vue'),
    },
    {
      path: '/invoices/:id/print',
      name: 'invoices.print',
      component: () => import('@/views/invoices/InvoicePrintView.vue'),
    },
  ],
})

router.beforeEach(async (to) => {
  if (to.name === 'onboarding') return true

  const configStore = useConfigStore()
  if (!configStore.isConfigured) {
    await configStore.loadConfig()
  }

  if (!configStore.isConfigured && to.name !== 'onboarding') {
    return { name: 'onboarding' }
  }
})

export default router

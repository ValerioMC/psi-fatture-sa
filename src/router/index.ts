import { createRouter, createWebHashHistory } from 'vue-router'
import { useConfigStore } from '@/stores/config'
import { useTermsStore } from '@/stores/terms'

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
        { path: 'invoices/monthly', name: 'invoices.monthly', component: () => import('@/views/invoices/MonthlyInvoiceView.vue') },
        { path: 'invoices/new', name: 'invoices.new', component: () => import('@/views/invoices/InvoiceFormView.vue') },
        { path: 'invoices/:id/edit', name: 'invoices.edit', component: () => import('@/views/invoices/InvoiceFormView.vue') },
        { path: 'invoices/:id', name: 'invoices.detail', component: () => import('@/views/invoices/InvoiceDetailView.vue') },
        { path: 'sts', name: 'sts', component: () => import('@/views/sts/StsView.vue') },
        { path: 'agenda', name: 'agenda', component: () => import('@/views/agenda/AgendaView.vue') },
        {
          path: 'settings',
          component: () => import('@/views/SettingsView.vue'),
          children: [
            // `?focus=sts` is the old link to the Sistema TS credentials.
            { path: '', redirect: (to) => ({ path: to.query.focus === 'sts' ? '/settings/sts' : '/settings/profile', query: {} }) },
            { path: 'profile', name: 'settings.profile', component: () => import('@/views/settings/SettingsProfileView.vue') },
            { path: 'invoicing', name: 'settings.invoicing', component: () => import('@/views/settings/SettingsInvoicingView.vue') },
            { path: 'email', name: 'settings.email', component: () => import('@/views/settings/SettingsEmailView.vue') },
            { path: 'sts', name: 'settings.sts', component: () => import('@/views/settings/SettingsStsView.vue') },
            { path: 'data', name: 'settings.data', component: () => import('@/views/settings/SettingsDataView.vue') },
            { path: 'appearance', name: 'settings.appearance', component: () => import('@/views/settings/SettingsAppearanceView.vue') },
            { path: 'legal', name: 'settings.legal', component: () => import('@/views/settings/SettingsLegalView.vue') },
          ],
        },
      ],
    },
    {
      path: '/terms',
      name: 'terms',
      component: () => import('@/views/TermsView.vue'),
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

// The terms of use come first, then the profile: nothing else opens until both are done.
router.beforeEach(async (to) => {
  const termsStore = useTermsStore()
  if (!termsStore.loaded) {
    await termsStore.load()
  }
  if (!termsStore.accepted) {
    return to.name === 'terms' ? true : { name: 'terms' }
  }
  if (to.name === 'terms') return { path: '/' }

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

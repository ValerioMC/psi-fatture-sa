import { createRouter, createWebHashHistory } from 'vue-router'
import { useConfigStore } from '@/features/settings/store/config'
import { useTermsStore } from '@/features/terms/store/terms'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      component: () => import('@/components/layout/AppLayout.vue'),
      children: [
        { path: '', redirect: '/dashboard' },
        { path: 'dashboard', name: 'dashboard', component: () => import('@/features/dashboard/views/DashboardView.vue') },
        { path: 'clients', name: 'clients', component: () => import('@/features/client/views/ClientListView.vue') },
        { path: 'clients/new', name: 'clients.new', component: () => import('@/features/client/views/ClientFormView.vue') },
        { path: 'clients/:id/edit', name: 'clients.edit', component: () => import('@/features/client/views/ClientFormView.vue') },
        { path: 'services', name: 'services', component: () => import('@/features/service/views/ServiceListView.vue') },
        { path: 'invoices', name: 'invoices', component: () => import('@/features/invoice/views/InvoiceListView.vue') },
        { path: 'invoices/monthly', name: 'invoices.monthly', component: () => import('@/features/invoice/views/MonthlyInvoiceView.vue') },
        { path: 'invoices/new', name: 'invoices.new', component: () => import('@/features/invoice/views/InvoiceFormView.vue') },
        { path: 'invoices/:id/edit', name: 'invoices.edit', component: () => import('@/features/invoice/views/InvoiceFormView.vue') },
        { path: 'invoices/:id', name: 'invoices.detail', component: () => import('@/features/invoice/views/InvoiceDetailView.vue') },
        { path: 'sts', name: 'sts', component: () => import('@/features/sts/views/StsView.vue') },
        { path: 'agenda', name: 'agenda', component: () => import('@/features/appointment/views/AgendaView.vue') },
        {
          path: 'settings',
          component: () => import('@/features/settings/views/SettingsView.vue'),
          children: [
            // `?focus=sts` is the old link to the Sistema TS credentials.
            { path: '', redirect: (to) => ({ path: to.query.focus === 'sts' ? '/settings/sts' : '/settings/profile', query: {} }) },
            { path: 'profile', name: 'settings.profile', component: () => import('@/features/settings/views/SettingsProfileView.vue') },
            { path: 'invoicing', name: 'settings.invoicing', component: () => import('@/features/settings/views/SettingsInvoicingView.vue') },
            { path: 'email', name: 'settings.email', component: () => import('@/features/settings/views/SettingsEmailView.vue') },
            { path: 'sts', name: 'settings.sts', component: () => import('@/features/settings/views/SettingsStsView.vue') },
            { path: 'data', name: 'settings.data', component: () => import('@/features/settings/views/SettingsDataView.vue') },
            { path: 'appearance', name: 'settings.appearance', component: () => import('@/features/settings/views/SettingsAppearanceView.vue') },
            { path: 'legal', name: 'settings.legal', component: () => import('@/features/settings/views/SettingsLegalView.vue') },
            { path: 'version', name: 'settings.version', component: () => import('@/features/settings/views/SettingsVersionView.vue') },
          ],
        },
      ],
    },
    {
      path: '/terms',
      name: 'terms',
      component: () => import('@/features/terms/views/TermsView.vue'),
    },
    {
      path: '/onboarding',
      name: 'onboarding',
      component: () => import('@/features/settings/views/OnboardingView.vue'),
    },
    {
      path: '/invoices/:id/print',
      name: 'invoices.print',
      component: () => import('@/features/invoice/views/InvoicePrintView.vue'),
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

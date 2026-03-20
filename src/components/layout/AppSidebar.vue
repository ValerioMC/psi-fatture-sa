<script setup lang="ts">
import { useRoute, RouterLink } from 'vue-router'
import { useConfigStore } from '@/stores/config'
import {
  LayoutDashboard,
  Users,
  Briefcase,
  FileText,
  Calendar,
  Settings,
} from 'lucide-vue-next'

const route = useRoute()
const configStore = useConfigStore()

const navItems = [
  { to: '/dashboard', label: 'Dashboard', icon: LayoutDashboard },
  { to: '/clients', label: 'Clienti', icon: Users },
  { to: '/services', label: 'Prestazioni', icon: Briefcase },
  { to: '/invoices', label: 'Fatture', icon: FileText },
  { to: '/agenda', label: 'Agenda', icon: Calendar },
]

const isActive = (path: string) =>
  route.path === path || route.path.startsWith(path + '/')
</script>

<template>
  <aside class="w-56 bg-white border-r border-gray-200 flex flex-col h-full shrink-0">
    <!-- Logo / App name -->
    <div class="px-5 py-5 border-b border-gray-100">
      <div class="flex items-center gap-2">
        <div class="w-7 h-7 rounded-lg bg-primary-600 flex items-center justify-center">
          <FileText class="w-4 h-4 text-white" />
        </div>
        <span class="font-semibold text-gray-900 text-sm">PSI Fatture</span>
      </div>
      <p v-if="configStore.fullName" class="text-xs text-gray-500 mt-1 truncate">
        {{ configStore.fullName }}
      </p>
    </div>

    <!-- Navigation -->
    <nav class="flex-1 px-3 py-3 space-y-0.5">
      <RouterLink
        v-for="item in navItems"
        :key="item.to"
        :to="item.to"
        class="flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors"
        :class="isActive(item.to)
          ? 'bg-primary-50 text-primary-700 font-medium'
          : 'text-gray-600 hover:bg-gray-100 hover:text-gray-900'"
      >
        <component :is="item.icon" class="w-4 h-4 shrink-0" />
        {{ item.label }}
      </RouterLink>
    </nav>

    <!-- Settings at bottom -->
    <div class="px-3 py-3 border-t border-gray-100">
      <RouterLink
        to="/settings"
        class="flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors"
        :class="isActive('/settings')
          ? 'bg-primary-50 text-primary-700 font-medium'
          : 'text-gray-600 hover:bg-gray-100 hover:text-gray-900'"
      >
        <Settings class="w-4 h-4 shrink-0" />
        Impostazioni
      </RouterLink>
    </div>
  </aside>
</template>

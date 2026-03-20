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
  Plus,
} from 'lucide-vue-next'

const route = useRoute()
const configStore = useConfigStore()

const navItems = [
  { to: '/dashboard',  label: 'Dashboard',   icon: LayoutDashboard },
  { to: '/clients',    label: 'Pazienti',     icon: Users },
  { to: '/services',   label: 'Prestazioni',  icon: Briefcase },
  { to: '/invoices',   label: 'Fatture',      icon: FileText },
  { to: '/agenda',     label: 'Agenda',       icon: Calendar },
]

const isActive = (path: string) =>
  route.path === path || route.path.startsWith(path + '/')
</script>

<template>
  <aside
    class="w-60 flex flex-col h-full shrink-0 glass-card"
    style="border-right: 1px solid rgba(163,186,163,0.25); border-radius: 0;"
  >
    <!-- ── Logo ── -->
    <div class="px-6 py-6 flex items-center gap-3">
      <div
        class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0"
        style="background: linear-gradient(135deg, #5d8062, #0c8aeb); box-shadow: 0 4px 12px rgba(93,128,98,0.35);"
      >
        <FileText class="w-4.5 h-4.5 text-white" />
      </div>
      <div>
        <h1 class="text-base font-bold leading-none gradient-text heading-serif">PSI Fatture</h1>
        <p v-if="configStore.fullName" class="text-[10px] text-sage-500 mt-1 font-medium tracking-wider uppercase truncate max-w-[7rem]">
          {{ configStore.fullName }}
        </p>
      </div>
    </div>

    <!-- ── Divider ── -->
    <div class="mx-5 h-px" style="background: linear-gradient(to right, transparent, rgba(93,128,98,0.2), transparent)" />

    <!-- ── Navigation ── -->
    <nav class="flex-1 px-3 py-4 space-y-0.5">
      <RouterLink
        v-for="item in navItems"
        :key="item.to"
        :to="item.to"
        class="group flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-sm font-medium transition-all duration-200 focus:outline-none"
        :class="isActive(item.to)
          ? 'bg-white/75 text-sage-800 shadow-sm'
          : 'text-sage-600 hover:bg-white/45 hover:text-sage-800'"
      >
        <component
          :is="item.icon"
          class="w-4 h-4 shrink-0 transition-colors duration-200"
          :class="isActive(item.to) ? 'text-sage-600' : 'text-sage-400 group-hover:text-sage-600'"
        />
        <span class="flex-1">{{ item.label }}</span>
        <!-- active indicator -->
        <span
          v-if="isActive(item.to)"
          class="w-1.5 h-4 rounded-full shrink-0"
          style="background: linear-gradient(to bottom, #5d8062, #0c8aeb)"
        />
      </RouterLink>
    </nav>

    <!-- ── Footer ── -->
    <div class="px-3 pb-5 space-y-2.5">
      <!-- CTA Nuova Fattura -->
      <RouterLink
        to="/invoices/new"
        class="group w-full text-white font-semibold py-2.5 px-4 rounded-xl flex items-center justify-center gap-2 transition-all duration-200 focus:outline-none relative overflow-hidden"
        style="background: linear-gradient(135deg, #5d8062, #0c8aeb); box-shadow: 0 4px 14px rgba(93,128,98,0.3);"
      >
        <!-- Shine effect -->
        <div
          class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/15 to-white/0 transform -skew-x-12 -translate-x-full group-hover:translate-x-full transition-transform duration-700"
          aria-hidden="true"
        />
        <Plus class="w-4 h-4 relative z-10" />
        <span class="text-sm relative z-10">Nuova Fattura</span>
      </RouterLink>

      <!-- Divider -->
      <div class="h-px mx-1" style="background: rgba(93,128,98,0.15)" />

      <!-- Settings -->
      <RouterLink
        to="/settings"
        class="group flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-sm font-medium transition-all duration-200 focus:outline-none"
        :class="isActive('/settings')
          ? 'bg-white/75 text-sage-800 shadow-sm'
          : 'text-sage-500 hover:bg-white/45 hover:text-sage-700'"
      >
        <Settings
          class="w-4 h-4 shrink-0 transition-colors"
          :class="isActive('/settings') ? 'text-sage-600' : 'text-sage-400 group-hover:text-sage-600'"
        />
        <span class="flex-1">Impostazioni</span>
        <span
          v-if="isActive('/settings')"
          class="w-1.5 h-4 rounded-full shrink-0"
          style="background: linear-gradient(to bottom, #5d8062, #0c8aeb)"
        />
      </RouterLink>
    </div>
  </aside>
</template>

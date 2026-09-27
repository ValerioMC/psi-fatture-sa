<script setup lang="ts">
/** The look of the app. It applies at once: each option is a tiny window in that theme. */
import { Monitor, Moon, Palette, Sun } from 'lucide-vue-next'
import AppCard from '@/components/ui/AppCard.vue'
import CardHeader from '@/components/ui/CardHeader.vue'
import { useTheme } from '@/composables/useTheme'
import type { ThemePreference } from '@/utils/theme'

const { preference, setPreference } = useTheme()

const THEMES: { value: ThemePreference; label: string; icon: typeof Sun }[] = [
  { value: 'system', label: 'Come il sistema', icon: Monitor },
  { value: 'light', label: 'Chiaro', icon: Sun },
  { value: 'dark', label: 'Scuro', icon: Moon },
]
</script>

<template>
  <AppCard :padded="false" class="settle max-w-2xl">
    <CardHeader title="Aspetto" subtitle="Si applica subito, solo su questo computer" :icon="Palette" />
    <div class="grid grid-cols-3 gap-3 px-5 pb-5" role="radiogroup" aria-label="Tema">
      <button
        v-for="theme in THEMES"
        :key="theme.value"
        type="button"
        role="radio"
        :aria-checked="preference === theme.value"
        class="group flex flex-col items-stretch gap-2.5 rounded-control p-2 text-sm transition-colors focus-ring"
        :class="preference === theme.value ? 'bg-accent-soft font-medium text-text ring-1 ring-accent' : 'text-text-muted ring-1 ring-border hover:ring-border-strong hover:text-text'"
        @click="setPreference(theme.value)"
      >
        <span class="theme-swatch" :data-swatch="theme.value" aria-hidden="true">
          <span class="theme-swatch-side" />
          <span class="theme-swatch-main"><span /><span /><span /></span>
        </span>
        <span class="flex items-center justify-center gap-1.5">
          <component :is="theme.icon" :size="13" :stroke-width="1.8" :class="preference === theme.value ? 'text-accent' : ''" aria-hidden="true" />
          {{ theme.label }}
        </span>
      </button>
    </div>
  </AppCard>
</template>

<style scoped>
/* A theme option: a thumbnail window, sidebar and three content lines, in that theme's colours. */
.theme-swatch {
  display: flex;
  height: 5rem;
  overflow: hidden;
  border-radius: 6px;
  box-shadow: inset 0 0 0 1px rgb(0 0 0 / 0.08);
}
.theme-swatch-side { width: 30%; }
.theme-swatch-main { flex: 1; display: flex; flex-direction: column; gap: 6px; padding: 10px 8px; }
.theme-swatch-main > span { height: 5px; border-radius: 99px; }
.theme-swatch-main > span:nth-child(1) { width: 70%; }
.theme-swatch-main > span:nth-child(2) { width: 90%; }
.theme-swatch-main > span:nth-child(3) { width: 45%; }

[data-swatch='light'] .theme-swatch-side { background: #fbfaf7; box-shadow: inset -1px 0 0 #e4e0d7; }
[data-swatch='light'] .theme-swatch-main { background: #f5f3ee; }
[data-swatch='light'] .theme-swatch-main > span { background: #cfc9bc; }
[data-swatch='light'] .theme-swatch-main > span:first-child { background: #3a3e9f; }

[data-swatch='dark'] .theme-swatch-side { background: #17161d; box-shadow: inset -1px 0 0 #2a2834; }
[data-swatch='dark'] .theme-swatch-main { background: #121117; }
[data-swatch='dark'] .theme-swatch-main > span { background: #3b3947; }
[data-swatch='dark'] .theme-swatch-main > span:first-child { background: #a5a9ff; }

/* "Come il sistema" is literally half of each. */
[data-swatch='system'] { background: linear-gradient(135deg, #f5f3ee 50%, #121117 50%); }
[data-swatch='system'] .theme-swatch-side { background: linear-gradient(135deg, #fbfaf7 50%, #17161d 50%); }
[data-swatch='system'] .theme-swatch-main > span { background: #8b8799; }
[data-swatch='system'] .theme-swatch-main > span:first-child { background: #6e72d0; }
</style>

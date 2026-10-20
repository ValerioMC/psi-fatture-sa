<script setup lang="ts">
/** The look of the app. It applies at once: each option is a tiny window in that theme. */
import { Monitor, Moon, Palette, Sun } from 'lucide-vue-next'
import AppCard from '@/components/ui/AppCard.vue'
import CardHeader from '@/components/ui/CardHeader.vue'
import { useTheme } from '@/common/composables/useTheme'
import type { ThemePreference } from '@/common/lib/theme'

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
  box-shadow: inset 0 0 0 1px rgb(var(--shade) / 0.1);
}
.theme-swatch-side { width: 30%; }
.theme-swatch-main { flex: 1; display: flex; flex-direction: column; gap: 6px; padding: 10px 8px; }
.theme-swatch-main > span { height: 5px; border-radius: 99px; }
.theme-swatch-main > span:nth-child(1) { width: 70%; }
.theme-swatch-main > span:nth-child(2) { width: 90%; }
.theme-swatch-main > span:nth-child(3) { width: 45%; }

[data-swatch='light'] .theme-swatch-side { background: #f8f7fd; box-shadow: inset -1px 0 0 #e0ddf0; }
[data-swatch='light'] .theme-swatch-main { background: #f3f2fa; }
[data-swatch='light'] .theme-swatch-main > span { background: #c7c3e3; }
[data-swatch='light'] .theme-swatch-main > span:first-child { background: #4540b8; }

[data-swatch='dark'] .theme-swatch-side { background: #151426; box-shadow: inset -1px 0 0 #2a2843; }
[data-swatch='dark'] .theme-swatch-main { background: #100f1c; }
[data-swatch='dark'] .theme-swatch-main > span { background: #3a3757; }
[data-swatch='dark'] .theme-swatch-main > span:first-child { background: #aeaaff; }

/* "Come il sistema" is literally half of each. */
[data-swatch='system'] { background: linear-gradient(135deg, #f3f2fa 50%, #100f1c 50%); }
[data-swatch='system'] .theme-swatch-side { background: linear-gradient(135deg, #f8f7fd 50%, #151426 50%); }
[data-swatch='system'] .theme-swatch-main > span { background: #8d89ad; }
[data-swatch='system'] .theme-swatch-main > span:first-child { background: #7a75e0; }
</style>

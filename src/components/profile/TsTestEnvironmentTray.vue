<script setup lang="ts">
/**
 * The Sogei test environment switch, drawn as scaffolding rather than product:
 * a dashed, sunken tray that only developer builds mount. Distributed builds
 * never render it, so end users see production alone.
 */
import { FlaskConical, Wrench } from 'lucide-vue-next'
import type { TsEnvironment } from '@/types'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'

defineProps<{ busy: boolean }>()
const environment = defineModel<TsEnvironment>({ required: true })
const emit = defineEmits<{ 'use-test-user': [] }>()

const ENVIRONMENTS = [
  { value: 'produzione', label: 'Produzione' },
  { value: 'test', label: 'Test Sogei', tone: 'warn' },
] as const
</script>

<template>
  <div class="space-y-3 rounded-control border border-dashed border-border-strong bg-surface-sunken px-3 py-3">
    <p class="flex items-center gap-2 text-xs text-text-subtle">
      <Wrench :size="13" :stroke-width="1.8" class="shrink-0" aria-hidden="true" />
      <span><span class="font-medium text-text-muted">Solo build di sviluppo.</span> Chi installa l’app non vede questo riquadro.</span>
    </p>

    <SegmentedControl v-model="environment" :options="ENVIRONMENTS" label="Ambiente Sistema TS" block size="sm" />

    <p v-if="environment === 'test'" class="flex items-start gap-2 rounded-control bg-warn-soft px-3 py-2 text-xs leading-relaxed text-text-muted ring-1 ring-inset ring-warn-line">
      <FlaskConical :size="14" class="mt-0.5 shrink-0 text-warn" aria-hidden="true" />
      <span>
        Le trasmissioni vanno all’ambiente di prova di Sogei e non hanno valore fiscale.
        <button type="button" class="font-medium text-text underline decoration-border-strong underline-offset-2 hover:decoration-text focus-ring rounded-sm" :disabled="busy" @click="emit('use-test-user')">
          Usa l’utenza di prova
        </button>
      </span>
    </p>
  </div>
</template>

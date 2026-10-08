<script setup lang="ts">
/**
 * A send the user started, while it runs: a ring that fills as the Sistema TS
 * answers, with the count at its centre. An arc keeps orbiting between answers,
 * since one call can take seconds. Closing it leaves the send running.
 */
import { computed } from 'vue'
import type { TsDispatchProgress } from '@/types'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import { DISPATCH_OUTCOMES, dispatchPercent, outcomeLabel } from '@/utils/sts'

const props = defineProps<{ open: boolean; progress: TsDispatchProgress }>()
const emit = defineEmits<{ close: [] }>()

const RADIUS = 52
const CIRCUMFERENCE = 2 * Math.PI * RADIUS
/** The orbiting arc: long enough to read as motion, short enough not to look like progress. */
const ORBIT_LENGTH = CIRCUMFERENCE * 0.07

const percent = computed(() => dispatchPercent(props.progress))
const filledOffset = computed(() => CIRCUMFERENCE * (1 - percent.value / 100))

const outcomes = computed(() =>
  DISPATCH_OUTCOMES.filter((outcome) => props.progress.summary[outcome] > 0).map((outcome) => ({
    outcome,
    label: outcomeLabel(outcome, props.progress.summary[outcome]),
  })),
)
</script>

<template>
  <AppDialog :open="open" size="sm" title="Invio al Sistema TS" description="Le fatture partono una alla volta." @close="emit('close')">
    <div class="flex flex-col items-center">
      <div
        class="progress-ring relative"
        role="progressbar"
        aria-label="Fatture inviate al Sistema TS"
        :aria-valuemin="0"
        :aria-valuemax="progress.total"
        :aria-valuenow="progress.processed"
        :aria-valuetext="`${progress.processed} di ${progress.total} fatture inviate, ${percent}%`"
      >
        <svg viewBox="0 0 120 120" class="size-full -rotate-90" aria-hidden="true">
          <circle class="track" cx="60" cy="60" :r="RADIUS" />
          <circle class="filled" cx="60" cy="60" :r="RADIUS" :stroke-dasharray="CIRCUMFERENCE" :stroke-dashoffset="filledOffset" />
          <circle v-if="!progress.finished" class="orbit" cx="60" cy="60" :r="RADIUS" :stroke-dasharray="`${ORBIT_LENGTH} ${CIRCUMFERENCE}`" />
        </svg>
        <div class="absolute inset-0 flex flex-col items-center justify-center" aria-hidden="true">
          <span class="tabular text-[2.25rem] leading-none font-semibold tracking-[-0.03em] text-text">{{ progress.processed }}</span>
          <span class="tabular mt-1.5 text-sm text-text-subtle">di {{ progress.total }}</span>
        </div>
      </div>

      <p class="mt-4 text-sm text-text-muted">fatture inviate</p>
      <ul class="mt-3 flex min-h-5 flex-wrap justify-center gap-x-4 gap-y-1 text-xs text-text-muted">
        <li v-for="entry in outcomes" :key="entry.outcome" class="tabular flex items-center gap-1.5">
          <span class="swatch" :data-outcome="entry.outcome" aria-hidden="true" />{{ entry.label }}
        </li>
      </ul>
    </div>

    <template #footer>
      <p class="mr-auto text-xs text-text-subtle">L’invio prosegue anche chiudendo.</p>
      <AppButton variant="secondary" size="sm" data-autofocus @click="emit('close')">Continua in background</AppButton>
    </template>
  </AppDialog>
</template>

<style scoped>
.progress-ring { width: 9.5rem; height: 9.5rem; }

.track, .filled, .orbit {
  fill: none;
  stroke-width: 6;
  stroke-linecap: round;
}
.track { stroke: var(--surface-sunken); }
.filled {
  stroke: var(--accent);
  transition: stroke-dashoffset 700ms var(--ease-out-expo);
}
/* Rides the track ahead of the fill, so the ring is visibly working while it waits for Sogei. */
.orbit {
  stroke: color-mix(in srgb, var(--accent) 45%, transparent);
  transform-origin: 60px 60px;
  animation: orbit 1.6s linear infinite;
}
@keyframes orbit { to { transform: rotate(360deg); } }

.swatch {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 2px;
}
.swatch[data-outcome='accepted'] { background-color: var(--safe); }
.swatch[data-outcome='rejected'] { background-color: var(--danger); }
.swatch[data-outcome='retrying'] { background-color: var(--warn); }
</style>

<script setup lang="ts">
/**
 * A pass over the Sistema TS queue as it happens: up to 60 transmissions, one
 * tick each, filled with its outcome as answers arrive; beyond that, one bar
 * split by outcome. A clean pass leaves by itself, otherwise it waits to be closed.
 */
import { computed, onBeforeUnmount, ref, watch, type Component } from 'vue'
import { CircleAlert, CircleCheck, TriangleAlert, X } from 'lucide-vue-next'
import type { TsDispatchProgress } from '@/types'
import AppButton from '@/components/ui/AppButton.vue'
import AppSpinner from '@/components/ui/AppSpinner.vue'
import { useTweenedNumber } from '@/composables/useTweenedNumber'
import {
  DISPATCH_OUTCOMES,
  answeredOutcome,
  dispatchHeadline,
  dispatchPercent,
  dispatchPhase,
  dispatchSegments,
  outcomeLabel,
  seedOutcomes,
  type DispatchOutcome,
  type DispatchPhase,
} from '@/utils/sts'

const MAX_TICKS = 60
const CLEAN_PASS_LINGER_MS = 5000

const props = defineProps<{ progress: TsDispatchProgress }>()
const emit = defineEmits<{ dismiss: [] }>()

const PHASE_TONE: Record<DispatchPhase, string> = {
  sending: 'var(--accent)',
  done: 'var(--safe)',
  attention: 'var(--warn)',
  interrupted: 'var(--danger)',
}
const PHASE_ICON: Record<Exclude<DispatchPhase, 'sending'>, Component> = {
  done: CircleCheck,
  attention: TriangleAlert,
  interrupted: CircleAlert,
}

const phase = computed(() => dispatchPhase(props.progress))
const headline = computed(() => dispatchHeadline(props.progress))
const percent = computed(() => dispatchPercent(props.progress))
const shownPercent = useTweenedNumber(percent)
const asTicks = computed(() => props.progress.total <= MAX_TICKS)

/** Answers in arrival order, one per tick; reset when a new pass starts. */
const outcomes = ref<DispatchOutcome[]>([])

watch(
  () => props.progress,
  (next, previous) => {
    const restarted = previous === undefined || next.processed < previous.processed || (previous.finished && !next.finished)
    if (restarted) {
      outcomes.value = seedOutcomes(next.summary)
      return
    }
    const outcome = answeredOutcome(previous.summary, next.summary)
    if (outcome !== null) outcomes.value = [...outcomes.value, outcome]
  },
  { immediate: true },
)

const legend = computed(() =>
  DISPATCH_OUTCOMES.filter((outcome) => props.progress.summary[outcome] > 0).map((outcome) => ({
    outcome,
    label: outcomeLabel(outcome, props.progress.summary[outcome]),
  })),
)

let lingerTimer: ReturnType<typeof setTimeout> | undefined
watch(
  phase,
  (current) => {
    clearTimeout(lingerTimer)
    if (current === 'done') lingerTimer = setTimeout(() => emit('dismiss'), CLEAN_PASS_LINGER_MS)
  },
  { immediate: true },
)
onBeforeUnmount(() => clearTimeout(lingerTimer))
</script>

<template>
  <section class="dispatch sheet relative overflow-hidden px-5 pt-4 pb-3.5" :style="{ '--tone': PHASE_TONE[phase] }" aria-labelledby="dispatch-title">
    <div class="flex items-center gap-4">
      <span class="icon-chip" :style="{ '--chip': 'var(--tone)' }" aria-hidden="true">
        <AppSpinner v-if="phase === 'sending'" />
        <component :is="PHASE_ICON[phase]" v-else :size="16" :stroke-width="1.8" />
      </span>
      <div class="min-w-0 flex-1">
        <h2 id="dispatch-title" class="text-md font-semibold text-text">{{ headline.title }}</h2>
        <p class="mt-0.5 text-sm text-text-muted" :aria-live="progress.finished ? 'polite' : 'off'">{{ headline.sentence }}</p>
      </div>
      <p class="tabular shrink-0 text-2xl font-semibold tracking-[-0.02em] text-text" aria-hidden="true">
        {{ Math.round(shownPercent) }}<span class="ml-0.5 text-base font-medium text-text-subtle">%</span>
      </p>
      <AppButton v-if="progress.finished" variant="ghost" size="sm" :icon="X" label="Chiudi" @click="emit('dismiss')" />
    </div>

    <div
      class="mt-3.5"
      role="progressbar"
      aria-label="Trasmissioni inviate al Sistema TS"
      :aria-valuemin="0"
      :aria-valuemax="progress.total"
      :aria-valuenow="progress.processed"
      :aria-valuetext="`${progress.processed} di ${progress.total}`"
    >
      <div v-if="asTicks" class="ticks" :style="{ '--count': progress.total }">
        <span
          v-for="index in progress.total"
          :key="index"
          class="tick"
          :data-outcome="outcomes[index - 1]"
          :data-next="phase === 'sending' && index - 1 === outcomes.length ? '' : undefined"
        />
      </div>
      <div v-else class="bar" :data-sending="phase === 'sending' ? '' : undefined">
        <span v-for="segment in dispatchSegments(progress)" :key="segment.outcome" class="segment" :data-outcome="segment.outcome" :style="{ width: `${segment.share}%` }" />
      </div>
    </div>

    <ul v-if="legend.length > 0" class="mt-2.5 flex flex-wrap gap-x-4 gap-y-1 text-xs text-text-muted">
      <li v-for="entry in legend" :key="entry.outcome" class="tabular flex items-center gap-1.5">
        <span class="swatch" :data-outcome="entry.outcome" aria-hidden="true" />{{ entry.label }}
      </li>
    </ul>
  </section>
</template>

<style scoped>
/* The phase's hue rises from the icon's corner, as on the stat tiles, and shifts with the outcome. */
.dispatch::before {
  content: '';
  position: absolute;
  inset: 0;
  pointer-events: none;
  background: radial-gradient(70% 140% at 0% 0%, color-mix(in srgb, var(--tone) 9%, transparent), transparent 60%);
  transition: background 600ms var(--ease-out-expo);
}
.dispatch > * { position: relative; }

.ticks {
  display: grid;
  grid-template-columns: repeat(var(--count), minmax(0, 1fr));
  gap: 3px;
  height: 0.625rem;
}
.tick {
  border-radius: 2px;
  background-color: var(--surface-sunken);
  box-shadow: inset 0 0 0 1px var(--border);
  transition: background-color 480ms var(--ease-out-expo), box-shadow 480ms var(--ease-out-expo);
}
/* The transmission whose answer is awaited breathes in the accent until it lands. */
.tick[data-next] { animation: awaiting 1.4s ease-in-out infinite; }
@keyframes awaiting {
  50% { background-color: color-mix(in srgb, var(--accent) 32%, var(--surface-sunken)); }
}

.bar {
  display: flex;
  height: 0.625rem;
  overflow: hidden;
  border-radius: 999px;
  background-color: var(--surface-sunken);
  box-shadow: inset 0 0 0 1px var(--border);
}
.bar[data-sending] {
  background-image: linear-gradient(90deg, transparent 0%, color-mix(in srgb, var(--accent) 14%, transparent) 50%, transparent 100%);
  background-size: 200% 100%;
  background-repeat: no-repeat;
  animation: sweep 1.6s ease-in-out infinite;
}
.segment { transition: width 700ms var(--ease-out-expo); }

.swatch {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 2px;
}

[data-outcome='accepted'] { background-color: var(--safe); box-shadow: none; }
[data-outcome='rejected'] { background-color: var(--danger); box-shadow: none; }
[data-outcome='retrying'] { background-color: var(--warn); box-shadow: none; }
</style>

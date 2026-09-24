<script setup lang="ts">
/**
 * The forfettario ceiling as a meter from 0 to 100.000 €, with the 85.000 €
 * line marked. The solid fill is where the year stands; a hatched extension
 * shows where the current pace leads by 31 December. The fill carries
 * severity (ink → warn → danger) and the track stays a lighter step of it.
 */
import { computed } from 'vue'
import { FORFETTARIO_HARD_THRESHOLD, FORFETTARIO_THRESHOLD, readThreshold } from '@/utils/forfettario'
import { formatCurrency, formatCurrencyCompact } from '@/utils/format'

const props = defineProps<{ amount: number; year: number }>()

const reading = computed(() => readThreshold(props.amount, props.year))

function share(value: number): number {
  return Math.min(100, Math.max(0, (value / FORFETTARIO_HARD_THRESHOLD) * 100))
}

const fillWidth = computed(() => share(reading.value.amount))
const projectionWidth = computed(() => {
  const projection = reading.value.projection
  if (projection === null) return 0
  return Math.max(0, share(projection) - fillWidth.value)
})
const markerLeft = share(FORFETTARIO_THRESHOLD)

const TONE = {
  ok: { fill: 'bg-accent', track: 'bg-accent/12', text: 'text-accent' },
  near: { fill: 'bg-warn', track: 'bg-warn/15', text: 'text-warn' },
  over: { fill: 'bg-danger', track: 'bg-danger/15', text: 'text-danger' },
} as const

const tone = computed(() => TONE[reading.value.level])

const sentence = computed(() => {
  const { projection, level, amount } = reading.value
  if (level === 'over') return `Soglia superata di ${formatCurrency(amount - FORFETTARIO_THRESHOLD)}.`
  if (projection === null) return `Margine residuo: ${formatCurrency(FORFETTARIO_THRESHOLD - amount)}.`
  if (projection > FORFETTARIO_THRESHOLD) return `A questo ritmo chiuderai l'anno a circa ${formatCurrency(projection)}, oltre la soglia.`
  return `A questo ritmo chiuderai l'anno a circa ${formatCurrency(projection)}.`
})
</script>

<template>
  <div>
    <p class="text-2xl font-semibold tracking-[-0.01em] text-text">
      {{ formatCurrency(amount) }}
      <span class="text-base font-normal text-text-subtle">su {{ formatCurrencyCompact(FORFETTARIO_THRESHOLD) }}</span>
    </p>

    <div
      class="relative mt-4 h-2.5"
      role="meter"
      :aria-valuemin="0"
      :aria-valuemax="FORFETTARIO_THRESHOLD"
      :aria-valuenow="Math.round(amount)"
      :aria-valuetext="`${formatCurrency(amount)} su ${formatCurrency(FORFETTARIO_THRESHOLD)}`"
      aria-label="Compensi rispetto alla soglia del forfettario"
    >
      <div class="absolute inset-0 overflow-hidden rounded-full" :class="tone.track">
        <div class="absolute inset-y-0 left-0 rounded-full transition-[width] duration-700 ease-out-expo" :class="tone.fill" :style="{ width: `${fillWidth}%` }" />
        <div
          v-if="projectionWidth > 0"
          class="projection absolute inset-y-0 transition-[width] duration-700 ease-out-expo"
          :class="tone.text"
          :style="{ left: `calc(${fillWidth}% + 2px)`, width: `calc(${projectionWidth}% - 2px)` }"
        />
      </div>
      <!-- The 85k line, taller than the track so it reads as a limit, not a segment. -->
      <div class="absolute -top-1 -bottom-1 w-0.5 rounded-full bg-text" :style="{ left: `${markerLeft}%` }" aria-hidden="true" />
    </div>
    <div class="relative mt-1.5 h-4 text-2xs text-text-subtle tabular" aria-hidden="true">
      <span class="absolute left-0">0</span>
      <span class="absolute -translate-x-1/2 whitespace-nowrap font-medium text-text-muted" :style="{ left: `${markerLeft}%` }">soglia 85k</span>
    </div>

    <p class="mt-3 text-sm text-text-muted">{{ sentence }}</p>
  </div>
</template>

<style scoped>
/* The projection is not data yet: the same hue, hatched at 45°, never solid. */
.projection {
  background-image: repeating-linear-gradient(
    -45deg,
    color-mix(in srgb, currentColor 45%, transparent) 0 2px,
    transparent 2px 5px
  );
}
</style>

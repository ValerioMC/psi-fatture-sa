<script setup lang="ts">
/**
 * A word-sized trend: one series, no axes, the last point marked. The line
 * is drawn in the tile's tone with a soft wash beneath it that fades to
 * nothing, so it adds shape to a figure without competing with it.
 */
import { computed, useId } from 'vue'

const props = withDefaults(
  defineProps<{
    values: readonly number[]
    color?: string
    height?: number
    /** Values after this index are not drawn: months still ahead are not zero. */
    lastIndex?: number
  }>(),
  { color: 'var(--accent)', height: 36 },
)

const gradientId = `spark-${useId()}`
const WIDTH = 120

const points = computed(() => {
  const series = props.lastIndex === undefined ? props.values : props.values.slice(0, props.lastIndex + 1)
  if (series.length < 2) return []
  const max = Math.max(...series, 0)
  const min = Math.min(...series, 0)
  const span = max - min || 1
  const step = WIDTH / (props.values.length - 1)
  const pad = 3
  return series.map((value, index) => ({
    x: index * step,
    y: pad + (1 - (value - min) / span) * (props.height - pad * 2),
  }))
})

/** A smoothed path through the points (monotone-ish cubic), so the line reads as a trend, not a zigzag. */
const linePath = computed(() => {
  const list = points.value
  if (list.length === 0) return ''
  let path = `M${list[0].x},${list[0].y}`
  for (let index = 1; index < list.length; index++) {
    const previous = list[index - 1]
    const current = list[index]
    const mid = (current.x - previous.x) / 2
    path += ` C${previous.x + mid},${previous.y} ${current.x - mid},${current.y} ${current.x},${current.y}`
  }
  return path
})

const areaPath = computed(() => {
  const list = points.value
  if (list.length === 0) return ''
  return `${linePath.value} L${list[list.length - 1].x},${props.height} L${list[0].x},${props.height} Z`
})

const last = computed(() => points.value[points.value.length - 1])
</script>

<template>
  <!-- The end dot is HTML, not SVG: the SVG stretches to its width and would draw an ellipse. -->
  <div v-if="points.length > 0" class="relative" aria-hidden="true">
    <svg
      :viewBox="`0 0 ${WIDTH} ${height}`"
      :height="height"
      preserveAspectRatio="none"
      class="block w-full overflow-visible"
    >
      <defs>
        <linearGradient :id="gradientId" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" :stop-color="color" stop-opacity="0.22" />
          <stop offset="1" :stop-color="color" stop-opacity="0" />
        </linearGradient>
      </defs>
      <path :d="areaPath" :fill="`url(#${gradientId})`" />
      <path :d="linePath" fill="none" :stroke="color" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" vector-effect="non-scaling-stroke" />
    </svg>
    <span
      v-if="last"
      class="absolute size-[7px] -translate-x-1/2 -translate-y-1/2 rounded-full ring-2 ring-surface-raised"
      :style="{ left: `${(last.x / WIDTH) * 100}%`, top: `${last.y}px`, backgroundColor: color }"
    />
  </div>
</template>

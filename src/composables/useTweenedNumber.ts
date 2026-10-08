import { onBeforeUnmount, ref, watch, type Ref, type WatchSource } from 'vue'

const DURATION_MS = 600

/** Ease-out expo, the curve the stylesheet names `--ease-out-expo`. */
function easeOutExpo(t: number): number {
  return t >= 1 ? 1 : 1 - 2 ** (-10 * t)
}

/**
 * A number that glides to each new value of `source` instead of jumping, so a
 * counter reads as moving. It jumps when the user prefers reduced motion.
 */
export function useTweenedNumber(source: WatchSource<number>): Ref<number> {
  const shown = ref(0)
  let frame = 0

  function glideTo(target: number): void {
    cancelAnimationFrame(frame)
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      shown.value = target
      return
    }
    const from = shown.value
    const start = performance.now()
    const step = (now: number): void => {
      const t = Math.min(1, (now - start) / DURATION_MS)
      shown.value = from + (target - from) * easeOutExpo(t)
      if (t < 1) frame = requestAnimationFrame(step)
    }
    frame = requestAnimationFrame(step)
  }

  watch(source, glideTo, { immediate: true })
  onBeforeUnmount(() => cancelAnimationFrame(frame))
  return shown
}

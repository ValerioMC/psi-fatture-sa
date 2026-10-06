import Lenis from 'lenis'
import {
  onBeforeUnmount,
  onMounted,
  shallowRef,
  type Ref,
  type ShallowRef,
} from 'vue'

interface SmoothScrollTargets {
  /** Scroll container. Omit to smooth the window/document scroll. */
  wrapper?: Ref<HTMLElement | null>
  /** Direct child of the wrapper holding the scrolled content. Required together with wrapper. */
  content?: Ref<HTMLElement | null>
}

/**
 * Attaches Lenis inertia scrolling to a container (or to the window when no
 * targets are given). Disabled automatically when the user prefers reduced motion.
 */
export function useSmoothScroll(targets: SmoothScrollTargets = {}): ShallowRef<Lenis | null> {
  const lenis = shallowRef<Lenis | null>(null)

  onMounted(() => {
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return

    const wrapper = targets.wrapper?.value ?? undefined
    const content = targets.content?.value ?? undefined
    if (targets.wrapper !== undefined && (wrapper === undefined || content === undefined)) return

    const containerOptions = wrapper !== undefined && content !== undefined ? { wrapper, content } : {}
    lenis.value = new Lenis({
      ...containerOptions,
      duration: 1.1,
      smoothWheel: true,
      autoRaf: true,
    })
  })

  onBeforeUnmount(() => {
    lenis.value?.destroy()
    lenis.value = null
  })

  return lenis
}

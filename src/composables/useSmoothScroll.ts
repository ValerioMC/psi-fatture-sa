import Lenis from 'lenis'
import {
  inject,
  onBeforeUnmount,
  onMounted,
  provide,
  shallowRef,
  type InjectionKey,
  type Ref,
  type ShallowRef,
} from 'vue'

const smoothScrollKey: InjectionKey<ShallowRef<Lenis | null>> = Symbol('smoothScroll')

interface SmoothScrollTargets {
  /** Scroll container. Omit to smooth the window/document scroll. */
  wrapper?: Ref<HTMLElement | null>
  /** Direct child of the wrapper holding the scrolled content. Required together with wrapper. */
  content?: Ref<HTMLElement | null>
}

/**
 * Attaches Lenis inertia scrolling to a container (or to the window when no
 * targets are given) and provides the instance to descendant components.
 * Disabled automatically when the user prefers reduced motion.
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

  provide(smoothScrollKey, lenis)
  return lenis
}

/**
 * Returns the Lenis instance provided by the closest ancestor that called
 * useSmoothScroll, or a null ref when smooth scrolling is not active.
 */
export function useSmoothScrollInstance(): ShallowRef<Lenis | null> {
  return inject(smoothScrollKey, () => shallowRef<Lenis | null>(null), true)
}

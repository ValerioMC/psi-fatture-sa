<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { RouterView, useRoute } from 'vue-router'
import AppSidebar from './AppSidebar.vue'
import CommandPalette from './CommandPalette.vue'
import { useSmoothScroll } from '@/composables/useSmoothScroll'
import { hasPrimaryModifier } from '@/utils/platform'

const paletteOpen = ref(false)
const scrollWrapperRef = ref<HTMLElement | null>(null)
const scrollContentRef = ref<HTMLElement | null>(null)
const lenis = useSmoothScroll({ wrapper: scrollWrapperRef, content: scrollContentRef })

const route = useRoute()
watch(
  () => route.path,
  () => {
    paletteOpen.value = false
    if (lenis.value !== null) lenis.value.scrollTo(0, { immediate: true })
    else scrollWrapperRef.value?.scrollTo({ top: 0 })
  },
)

function onKeydown(event: KeyboardEvent): void {
  if (event.key.toLowerCase() === 'k' && hasPrimaryModifier(event)) {
    event.preventDefault()
    paletteOpen.value = !paletteOpen.value
  }
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="flex h-screen overflow-hidden">
    <AppSidebar @open-palette="paletteOpen = true" />
    <main ref="scrollWrapperRef" class="min-w-0 flex-1 overflow-y-auto">
      <div ref="scrollContentRef" class="min-h-full">
        <RouterView v-slot="{ Component, route: current }">
          <Transition name="pane" mode="out-in">
            <component :is="Component" :key="current.path" />
          </Transition>
        </RouterView>
      </div>
    </main>
    <CommandPalette :open="paletteOpen" @close="paletteOpen = false" />
  </div>
</template>

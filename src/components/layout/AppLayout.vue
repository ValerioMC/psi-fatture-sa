<script setup lang="ts">
import { ref, watch } from 'vue'
import { RouterView, useRoute } from 'vue-router'
import AppSidebar from './AppSidebar.vue'
import { useSmoothScroll } from '@/composables/useSmoothScroll'

const scrollWrapperRef = ref<HTMLElement | null>(null)
const scrollContentRef = ref<HTMLElement | null>(null)
const lenis = useSmoothScroll({ wrapper: scrollWrapperRef, content: scrollContentRef })

const route = useRoute()
watch(
  () => route.fullPath,
  () => {
    if (lenis.value !== null) lenis.value.scrollTo(0, { immediate: true })
    else scrollWrapperRef.value?.scrollTo({ top: 0 })
  },
)
</script>

<template>
  <div
    class="flex h-screen overflow-hidden"
    style="background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%)"
  >
    <AppSidebar />
    <main ref="scrollWrapperRef" class="flex-1 overflow-y-auto">
      <div ref="scrollContentRef" class="min-h-full">
        <RouterView />
      </div>
    </main>
  </div>
</template>

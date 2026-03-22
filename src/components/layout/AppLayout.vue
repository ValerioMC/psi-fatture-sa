<script setup lang="ts">
import { RouterView } from 'vue-router'
import AppSidebar from './AppSidebar.vue'
</script>

<template>
  <div
    class="flex h-screen overflow-hidden"
    style="background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%)"
  >
    <AppSidebar />
    <main class="flex-1 overflow-y-auto">
      <RouterView />
    </main>
  </div>
</template>

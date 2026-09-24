import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import { initTheme } from './composables/useTheme'
import { markPlatform } from './utils/platform'
import './style.css'

// Theme and platform marks go on <html> before the first paint, so there is no flash.
initTheme()
markPlatform()

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')

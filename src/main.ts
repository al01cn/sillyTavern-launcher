import { createApp } from 'vue'
import 'vue-sonner/style.css'
import './style.css'
import App from './App.vue'
import router from './router'

if (import.meta.env.PROD) {
  // 禁用右键菜单
  document.addEventListener('contextmenu', (e) => {
    e.preventDefault()
  })

  // 禁用 F12 和一些常见的开发者工具快捷键
  document.addEventListener('keydown', (e) => {
    if (
      e.key === 'F12' ||
      (e.ctrlKey && e.shiftKey && e.key === 'I') ||
      (e.ctrlKey && e.shiftKey && e.key === 'J') ||
      (e.ctrlKey && e.key === 'U') ||
      (e.metaKey && e.altKey && e.key === 'I') ||
      (e.metaKey && e.altKey && e.key === 'J') ||
      (e.metaKey && e.altKey && e.key === 'U')
    ) {
      e.preventDefault()
    }
  })
}

const app = createApp(App)
app.use(router)
app.mount('#app')
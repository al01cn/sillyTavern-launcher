import { ref, watch } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'auto'

// 获取系统主题
function getSystemTheme(): 'light' | 'dark' {
  if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
    return 'dark'
  }
  return 'light'
}

// 从配置中获取主题设置
function getConfigTheme(): ThemeMode {
  try {
    const cachedConfig = localStorage.getItem('app_settings_config_cache')
    if (cachedConfig) {
      const config = JSON.parse(cachedConfig)
      return config.theme || 'auto'
    }
  } catch (e) {
    console.error('解析配置失败:', e)
  }
  return 'auto'
}

// 确定实际使用的主题
function getActualTheme(mode: ThemeMode): 'light' | 'dark' {
  if (mode === 'auto') {
    return getSystemTheme()
  }
  return mode
}

// 应用主题到 DOM
function applyTheme(theme: 'light' | 'dark') {
  const root = document.documentElement
  if (theme === 'dark') {
    root.classList.add('dark')
    root.setAttribute('data-theme', 'dark')
  } else {
    root.classList.remove('dark')
    root.setAttribute('data-theme', 'light')
  }
}

// 当前主题模式
export const themeMode = ref<ThemeMode>(getConfigTheme())

// 当前实际主题
export const currentTheme = ref<'light' | 'dark'>(getActualTheme(themeMode.value))

// 初始化主题
export function initTheme() {
  const mode = getConfigTheme()
  themeMode.value = mode
  currentTheme.value = getActualTheme(mode)
  applyTheme(currentTheme.value)

  // 监听系统主题变化
  if (window.matchMedia) {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    mediaQuery.addEventListener('change', e => {
      if (themeMode.value === 'auto') {
        currentTheme.value = e.matches ? 'dark' : 'light'
        applyTheme(currentTheme.value)
      }
    })
  }

  // 监听主题模式变化
  watch(themeMode, newMode => {
    currentTheme.value = getActualTheme(newMode)
    applyTheme(currentTheme.value)
  })
}

// 设置主题
export function setTheme(mode: ThemeMode) {
  themeMode.value = mode
  currentTheme.value = getActualTheme(mode)
  applyTheme(currentTheme.value)
}

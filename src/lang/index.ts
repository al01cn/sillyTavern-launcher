import { createI18n } from 'vue-i18n'
import zhCN from './zh-CN'
import enUS from './en-US'

// 获取系统语言
function getSystemLocale(): string {
  const lang = navigator.language || 'zh-CN'
  if (lang.startsWith('zh')) return 'zh-CN'
  if (lang.startsWith('en')) return 'en-US'
  return 'zh-CN'
}

// 从配置中获取语言设置
function getConfigLocale(): string {
  try {
    const cachedConfig = localStorage.getItem('app_settings_config_cache')
    if (cachedConfig) {
      const config = JSON.parse(cachedConfig)
      return config.lang || 'auto'
    }
  } catch (e) {
    console.error('Failed to parse config:', e)
  }
  return 'auto'
}

// 确定实际使用的语言
function getActualLocale(): string {
  const configLang = getConfigLocale()
  if (configLang === 'auto') {
    return getSystemLocale()
  }
  return configLang
}

const i18n = createI18n({
  legacy: false,
  locale: getActualLocale(),
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
})

export default i18n
export { getSystemLocale, getActualLocale }

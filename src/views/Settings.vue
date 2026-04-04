<script lang="ts">
// Global promise to prevent concurrent loads across component instances
let globalLoadConfigPromise: Promise<void> | null = null
</script>

<script setup lang="ts">
import { ref, onMounted, watch, nextTick, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { openUrl as open } from '@tauri-apps/plugin-opener'
import { toast } from 'vue-sonner'
import {
  PhCheck,
  PhArrowsClockwise,
  PhGlobe,
  PhPalette,
  PhGithubLogo,
  PhInfo,
  PhPackage,
  PhDownloadSimple,
  PhShield,
  PhShieldCheck,
  PhGitBranch,
  PhSliders,
  PhTerminalWindow,
  PhX,
  PhWarning,
  PhDesktop,
  PhWifiHigh,
  PhCheckCircle,
  PhXCircle,
  PhThumbsUp,
  PhStar,
  PhHeart,
} from '@phosphor-icons/vue'
import globalConfig from '../lib/config'
import { checkUpdate } from '../lib/updater'
import { setTheme } from '../lib/theme'
import { getSystemLocale } from '../lang'
import { startOneClickSetup, updateOneClickMessage, simulateClickEffect } from '../lib/useOneClick'
import { gitInstallState, nodeInstallState, cancelGitInstall, cancelNodeInstall } from '../lib/useNodeGitInstall'

const { t, locale } = useI18n()
const router = useRouter()
const route = useRoute()

interface GithubProxyConfig {
  enable: boolean
  url: string
}

interface GitInfo {
  version: string | null
  path: string | null
  source: 'system' | 'local' | 'none'
}

interface NodeInfo {
  version: string | null
  path: string | null
  source: 'system' | 'local' | 'none'
}

interface NpmInfo {
  version: string | null
  path: string | null
  source: 'system' | 'local' | 'none'
}

interface AppConfig {
  lang: string
  theme: 'light' | 'dark' | 'auto'
  rememberWindowPosition: boolean
  githubProxy: GithubProxyConfig
  npmRegistry: string
  scanCpuCores: number | null
  regionAutoConfigured: boolean
  initialSetupCompleted: boolean
  enableAnimations: boolean
  setupCheckpoint: string | null
  useSystemNode: boolean
  useSystemGit: boolean
  launchMode: 'normal' | 'desktop' | 'lan' | 'public' | 'debug'
  dataMode: 'global' | 'local'
  networkProxy: {
    mode: 'none' | 'system' | 'custom'
    host: string
    port: number
  }
}

interface ProxyItem {
  url: string
  latency: number
}

interface NpmRegistry {
  name: string
  url: string
}

const activeTab = ref<'general' | 'credits' | 'about'>('general')
const creditsData = globalConfig.ca
const loading = ref(false)
const proxyLoading = ref(false)
const proxyLastFetchTimeDisplay = ref('')
let isSyncing = false
const checkingUpdate = ref(false)

const handleCheckUpdate = async () => {
  if (checkingUpdate.value) return
  checkingUpdate.value = true
  await checkUpdate(true)
  checkingUpdate.value = false
}

const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

const npmRegistries = computed<NpmRegistry[]>(() => [
  { name: t('settings.npmRegistryOfficial'), url: 'https://registry.npmjs.org/' },
  { name: t('settings.npmRegistryTaobao'), url: 'https://registry.npmmirror.com/' },
  { name: t('settings.npmRegistryTencent'), url: 'https://mirrors.cloud.tencent.com/npm/' },
  { name: t('settings.npmRegistryHuawei'), url: 'https://repo.huaweicloud.com/repository/npm/' },
])

const config = ref<AppConfig>({
  lang: 'auto',
  theme: 'auto',
  rememberWindowPosition: false,
  githubProxy: {
    enable: false,
    url: 'https://ghfast.top/',
  },
  npmRegistry: 'https://registry.npmmirror.com/',
  scanCpuCores: null,
  regionAutoConfigured: false,
  initialSetupCompleted: false,
  enableAnimations: true,
  setupCheckpoint: null,
  useSystemNode: true,
  useSystemGit: true,
  launchMode: 'normal',
  dataMode: 'global',
  networkProxy: {
    mode: 'none',
    host: '127.0.0.1',
    port: 7890,
  },
})

const proxies = ref<ProxyItem[]>([])
const gitInfo = ref<GitInfo>({ version: null, path: null, source: 'none' })
const nodeInfo = ref<NodeInfo>({ version: null, path: null, source: 'none' })
const npmInfo = ref<NpmInfo>({ version: null, path: null, source: 'none' })

// 同时检测系统/内置 Node（用于切换 Toggle）
interface BothNodeInfo {
  system: NodeInfo | null
  local: NodeInfo | null
}
const bothNodeInfo = ref<BothNodeInfo>({ system: null, local: null })

// 同时检测系统/内置 Git（用于切换 Toggle）
interface BothGitInfo {
  system: GitInfo | null
  local: GitInfo | null
}
const bothGitInfo = ref<BothGitInfo>({ system: null, local: null })

const checkNodeBoth = async () => {
  try {
    const res = await invoke<BothNodeInfo>('check_nodejs_both')
    bothNodeInfo.value = res
  } catch (e) {
    console.error('check_nodejs_both failed:', e)
  }
}

// 切换 Node 来源（系统/内置），保存配置并重新检测
const switchNodeSource = async (useSystem: boolean) => {
  config.value.useSystemNode = useSystem
  await saveConfig()
  await checkNode()
  await checkNpm()
  await checkNodeBoth() // 切换后刷新，确保按钮文案正确反映内置 Node 是否存在
}

const checkGitBoth = async () => {
  try {
    const res = await invoke<BothGitInfo>('check_git_both')
    bothGitInfo.value = res
  } catch (e) {
    console.error('check_git_both failed:', e)
  }
}

// 切换 Git 来源（系统/内置），保存配置并重新检测
const switchGitSource = async (useSystem: boolean) => {
  config.value.useSystemGit = useSystem
  await saveConfig()
  await checkGit()
  await checkGitBoth()
}

const isElevated = ref(false)
const elevating = ref(false)
const systemCpuCores = ref<number>(parseInt(localStorage.getItem('app_system_cpu_cores') || '0', 10))
const logDialogVisible = ref(false)
const logDialogTitle = ref('')
const currentLogs = ref<string[]>([])
const logContainer = ref<HTMLElement | null>(null)

const showLogs = (type: 'git' | 'node') => {
  if (type === 'git') {
    logDialogTitle.value = t('settings.git') + ' ' + t('common.logs', '日志')
    currentLogs.value = gitInstallState.logs
  } else {
    logDialogTitle.value = t('settings.nodejs') + ' ' + t('common.logs', '日志')
    currentLogs.value = nodeInstallState.logs
  }
  logDialogVisible.value = true

  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  })
}

watch(
  currentLogs,
  () => {
    if (logDialogVisible.value) {
      nextTick(() => {
        if (logContainer.value) {
          logContainer.value.scrollTop = logContainer.value.scrollHeight
        }
      })
    }
  },
  { deep: true },
)

const isNodeVersionValid = computed(() => {
  if (!nodeInfo.value.version) return false
  // Version string usually looks like "v18.20.4"
  const match = nodeInfo.value.version.match(/v?(\d+)\./)
  if (match && match[1]) {
    const majorVersion = parseInt(match[1], 10)
    return majorVersion >= requiredNodeMajor.value
  }
  return false
})

// 根据当前选择的酒馆版本动态计算所需的最低 Node 主版本
// ST >= 1.17.0 需要 v20+，否则需要 v18+
const requiredNodeMajor = computed(() => {
  try {
    const configCache = localStorage.getItem('app_settings_config_cache')
    if (configCache) {
      const cached = JSON.parse(configCache)
      const tavernVer: string | null = cached?.sillytavern?.version?.version || null
      if (tavernVer) {
        const m = tavernVer.replace(/^v/, '').match(/^(\d+)\.(\d+)\./)
        if (m) {
          const major = parseInt(m[1], 10)
          const minor = parseInt(m[2], 10)
          if (major > 1 || (major === 1 && minor >= 17)) return 20
        }
      }
    }
  } catch (e) {
    /* ignore */
  }
  return 18
})

const loadConfig = async () => {
  if (globalLoadConfigPromise) return globalLoadConfigPromise

  globalLoadConfigPromise = (async () => {
    try {
      // 先尝试从缓存中读取，实现秒开
      const cachedConfig = localStorage.getItem('app_settings_config_cache')
      if (cachedConfig) {
        try {
          config.value = JSON.parse(cachedConfig)
          loading.value = false
        } catch (e) {
          console.error('缓存解析失败:', e)
          loading.value = true
        }
      } else {
        loading.value = true
      }

      // 后台静默获取最新配置
      const res = await invoke<AppConfig>('get_app_config')

      // 检查后端数据与当前状态是否不一致（只对比当前配置的关键字段）
      let isDifferent = false
      if (!cachedConfig) {
        isDifferent = true
      } else {
        for (const key in res) {
          if (JSON.stringify(res[key as keyof AppConfig]) !== JSON.stringify(config.value[key as keyof AppConfig])) {
            isDifferent = true
            break
          }
        }
      }

      if (isDifferent) {
        // 避免触发 watch 的自动保存
        isSyncing = true
        config.value = { ...config.value, ...res }

        // 立即应用主题和语言设置
        setTheme(res.theme)
        if (res.lang === 'auto') {
          locale.value = getSystemLocale()
        } else {
          locale.value = res.lang
        }

        // 更新缓存并保留其他模块追加的数据
        const currentCachedStr = localStorage.getItem('app_settings_config_cache')
        let mergedConfig = { ...res }
        if (currentCachedStr) {
          try {
            const cached = JSON.parse(currentCachedStr)
            mergedConfig = { ...cached, ...res }
          } catch (e) {}
        }
        localStorage.setItem('app_settings_config_cache', JSON.stringify(mergedConfig))
      } else {
        // 即使配置没有变化,也要确保主题和语言正确应用
        setTheme(config.value.theme)
        if (config.value.lang === 'auto') {
          locale.value = getSystemLocale()
        } else {
          locale.value = config.value.lang
        }
      }

      // 等待 DOM 更新后解除状态
      await nextTick()
      isSyncing = false
    } catch (error) {
      console.error('Failed to load config:', error)
      toast.error(t('settings.loadFailed'))
    } finally {
      loading.value = false
      globalLoadConfigPromise = null
    }
  })()

  return globalLoadConfigPromise
}

// 验证 URL 格式：必须是 http:// 或 https:// 开头
const isValidProxyUrl = (url: string): boolean => {
  if (!url || !url.trim()) return false
  try {
    const trimmed = url.trim()
    // 必须以 http:// 或 https:// 开头
    if (!trimmed.startsWith('http://') && !trimmed.startsWith('https://')) return false
    // 再做一次 URL 构造验证
    new URL(trimmed)
    return true
  } catch {
    return false
  }
}

const saveConfig = async () => {
  try {
    // 兜底：github_proxy.url 必须是合法 URL，否则恢复默认值
    if (!isValidProxyUrl(config.value.githubProxy.url)) {
      config.value.githubProxy.url = 'https://ghfast.top/'
    }

    await invoke('save_app_config', { config: config.value })

    // 保存成功后同时更新本地缓存，合并现有数据以免覆盖其他模块追加的数据(如 sillytavern.version)
    const cachedStr = localStorage.getItem('app_settings_config_cache')
    let mergedConfig = { ...config.value }
    if (cachedStr) {
      try {
        const cached = JSON.parse(cachedStr)
        mergedConfig = { ...cached, ...config.value }
      } catch (e) {}
    }
    localStorage.setItem('app_settings_config_cache', JSON.stringify(mergedConfig))

    // toast.success('设置已保存'); // Remove toast for real-time save to avoid spam
    console.log('Config saved')
  } catch (error) {
    console.error('Failed to save config:', error)
    toast.error(t('settings.saveFailed'))
  }
}

const fetchProxies = async (forceUpdate = false) => {
  const THREE_DAYS_MS = 3 * 24 * 60 * 60 * 1000
  const now = Date.now()
  const cachedProxies = localStorage.getItem('app_settings_proxies_cache')
  const lastFetchTime = localStorage.getItem('app_settings_proxies_last_fetch')

  if (cachedProxies) {
    try {
      const parsed = JSON.parse(cachedProxies)
      if (Array.isArray(parsed) && parsed.length > 0) {
        proxies.value = parsed
        if (lastFetchTime) {
          proxyLastFetchTimeDisplay.value = formatDate(new Date(Number(lastFetchTime)).toISOString())
        }

        // 如果不是强制刷新，并且距离上次获取还没超过3天，则不再请求接口
        if (!forceUpdate && lastFetchTime && now - Number(lastFetchTime) < THREE_DAYS_MS) {
          return
        }
      }
    } catch (e) {
      console.error('加速节点缓存解析失败:', e)
    }
  }

  try {
    proxyLoading.value = true
    const res = await invoke<ProxyItem[]>('fetch_github_proxies')
    const sortedProxies = res.sort((a, b) => a.latency - b.latency)

    const fetchedString = JSON.stringify(sortedProxies)
    if (cachedProxies !== fetchedString) {
      proxies.value = sortedProxies
      localStorage.setItem('app_settings_proxies_cache', fetchedString)
    }

    localStorage.setItem('app_settings_proxies_last_fetch', now.toString())
    proxyLastFetchTimeDisplay.value = formatDate(new Date(now).toISOString())
    // toast.success('获取加速列表成功'); // Remove toast on auto-fetch
  } catch (error) {
    console.error('Failed to fetch proxies:', error)
    toast.error(t('settings.saveFailed'))
  } finally {
    proxyLoading.value = false
  }
}

const selectProxy = (url: string) => {
  config.value.githubProxy.url = url
  // watch will handle saving
}

const checkGit = async () => {
  try {
    const cachedGit = localStorage.getItem('app_settings_git_cache')
    if (cachedGit) {
      try {
        gitInfo.value = JSON.parse(cachedGit)
      } catch (e) {}
    }

    const res = await invoke<GitInfo>('check_git')

    if (JSON.stringify(res) !== JSON.stringify(gitInfo.value)) {
      gitInfo.value = res
      localStorage.setItem('app_settings_git_cache', JSON.stringify(res))
    }
  } catch (error) {
    console.error('Failed to check git:', error)
  }
}

const checkNode = async () => {
  try {
    // 优先从缓存读取
    const cachedNode = localStorage.getItem('app_settings_node_cache')
    if (cachedNode) {
      try {
        nodeInfo.value = JSON.parse(cachedNode)
      } catch (e) {}
    }

    const res = await invoke<NodeInfo>('check_nodejs')

    // 如果与缓存不一致，则更新缓存和状态
    if (JSON.stringify(res) !== JSON.stringify(nodeInfo.value)) {
      nodeInfo.value = res
      localStorage.setItem('app_settings_node_cache', JSON.stringify(res))
    }
  } catch (error) {
    console.error('Failed to check nodejs:', error)
  }
}

const checkNpm = async () => {
  try {
    // 优先从缓存读取
    const cachedNpm = localStorage.getItem('app_settings_npm_cache')
    if (cachedNpm) {
      try {
        npmInfo.value = JSON.parse(cachedNpm)
      } catch (e) {}
    }

    const res = await invoke<NpmInfo>('check_npm')

    // 如果与缓存不一致，则更新缓存和状态
    if (JSON.stringify(res) !== JSON.stringify(npmInfo.value)) {
      npmInfo.value = res
      localStorage.setItem('app_settings_npm_cache', JSON.stringify(res))
    }
  } catch (error) {
    console.error('Failed to check npm:', error)
  }
}

const checkElevation = async () => {
  try {
    isElevated.value = await invoke<boolean>('is_elevated')
  } catch (error) {
    console.error('Failed to check elevation:', error)
  }
}

const checkSystemCpuCores = async () => {
  try {
    const cachedCores = localStorage.getItem('app_system_cpu_cores')
    if (cachedCores) {
      systemCpuCores.value = parseInt(cachedCores, 10)
    }
    const cores = await invoke<number>('get_system_cpu_cores')
    systemCpuCores.value = cores
    localStorage.setItem('app_system_cpu_cores', cores.toString())
  } catch (error) {
    console.error('Failed to get cpu cores:', error)
  }
}

const requestElevation = async () => {
  if (elevating.value) return
  elevating.value = true
  try {
    await invoke('elevate_process')
    // The app will restart, so we don't need to reset elevating.value
  } catch (error) {
    console.error('Failed to elevate:', error)
    toast.error(t('common.failed') + ': ' + error)
    elevating.value = false
  }
}

const installGit = async () => {
  if (gitInstallState.installing) return
  gitInstallState.installing = true
  gitInstallState.logs = [t('common.processing', 'Processing...')]
  gitInstallState.progress = { status: 'starting', progress: 0, log: t('common.processing') }

  if (route.query.action === 'one_click_setup') {
    updateOneClickMessage(t('oneClick.gitDetecting'))
    await invoke('save_app_config', { config: { ...config.value, setupCheckpoint: 'START' } })
  }

  try {
    await invoke('install_git')
    if (route.query.action === 'one_click_setup') {
      updateOneClickMessage(t('oneClick.gitSuccess'))
      await invoke('save_app_config', { config: { ...config.value, setupCheckpoint: 'GIT_DONE' } })
    } else {
      toast.success(t('settings.gitInstall') + ' ' + t('common.success'))
    }
    await checkGit()
    await checkGitBoth()
    // 安装完内置 Git 后自动切换到内置 Git 来源
    await switchGitSource(false)
  } catch (error) {
    const errStr = String(error)
    const isCancelled = errStr.includes('取消') || errStr.toLowerCase().includes('cancel')
    console.error('Failed to install git:', error)
    if (isCancelled) {
      toast.info(t('settings.gitInstallCancelled', '安装已取消'))
    } else if (route.query.action === 'one_click_setup') {
      updateOneClickMessage(t('common.failed') + ': ' + error)
    } else {
      toast.error(t('common.failed') + ': ' + error)
    }
    if (!isCancelled) throw error
  } finally {
    gitInstallState.installing = false
  }
}

const refreshGitEnv = async () => {
  await checkGit()
  await checkGitBoth()
}

const installNode = async () => {
  if (nodeInstallState.installing) return
  nodeInstallState.installing = true
  nodeInstallState.logs = [t('common.processing', 'Processing...')]
  nodeInstallState.progress = { status: 'starting', progress: 0, log: t('common.processing') }

  if (route.query.action === 'one_click_setup') {
    updateOneClickMessage(t('oneClick.nodeDetecting'))
  }

  try {
    await invoke('install_nodejs')
    // 安装完内置 Node 后自动切换到内置 Node 来源
    await switchNodeSource(false)
    if (route.query.action === 'one_click_setup') {
      updateOneClickMessage(t('oneClick.nodeSuccess'))
      await invoke('save_app_config', { config: { ...config.value, setupCheckpoint: 'NODE_DONE' } })
    } else {
      toast.success(t('settings.nodejsInstall') + ' ' + t('common.success'))
    }
  } catch (error) {
    const errStr = String(error)
    const isCancelled = errStr.includes('取消') || errStr.toLowerCase().includes('cancel')
    console.error('Failed to install nodejs:', error)
    if (isCancelled) {
      toast.info(t('settings.nodejsInstallCancelled', '安装已取消'))
    } else if (route.query.action === 'one_click_setup') {
      updateOneClickMessage(t('common.failed') + ': ' + error)
    } else {
      toast.error(t('common.failed') + ': ' + error)
    }
    if (!isCancelled) throw error
  } finally {
    nodeInstallState.installing = false
  }
}

const refreshNodeEnv = async () => {
  await checkNode()
  await checkNpm()
  await checkNodeBoth()
}

// Watch for config changes and save automatically
watch(
  config,
  () => {
    if (!loading.value && !isSyncing) {
      saveConfig()
    }
  },
  { deep: true },
)

// 监听语言变化
watch(
  () => config.value.lang,
  newLang => {
    if (newLang === 'auto') {
      locale.value = getSystemLocale()
    } else {
      locale.value = newLang
    }
  },
)

// 监听主题变化
watch(
  () => config.value.theme,
  newTheme => {
    setTheme(newTheme)
  },
)

onMounted(() => {
  loadConfig()
  fetchProxies()
  checkGit().then(() => {
    checkNode().then(() => {
      // 如果是通过一键安装NodeJS进来的，自动触发安装
      if (route.query.action === 'install_node' || route.query.action === 'one_click_setup') {
        if (route.query.action === 'one_click_setup') {
          startOneClickSetup(t('oneClick.startWait'))
        }

        const doNodeInstall = () => {
          setTimeout(() => {
            document.getElementById('node-settings')?.scrollIntoView({ behavior: 'smooth' })

            setTimeout(() => {
              if (!isNodeVersionValid.value || nodeInfo.value.source === 'local') {
                simulateClickEffect('btn-install-node')
                installNode()
                  .then(() => {
                    if (route.query.action === 'one_click_setup') {
                      setTimeout(() => {
                        router.push('/versions?action=one_click_setup_st')
                      }, 3000)
                    }
                  })
                  .catch(() => {
                    if (route.query.action === 'one_click_setup') {
                      setTimeout(() => {
                        router.push('/versions?action=one_click_setup_st')
                      }, 3000)
                    }
                  })
              } else if (route.query.action === 'one_click_setup') {
                updateOneClickMessage(t('oneClick.nodeInstalled'))
                setTimeout(() => {
                  router.push('/versions?action=one_click_setup_st')
                }, 3000)
              }
            }, 500) // 留出一点滚动动画的时间
          }, 300)
        }

        const checkAndInstallGit = () => {
          setTimeout(() => {
            document.getElementById('git-settings')?.scrollIntoView({ behavior: 'smooth' })

            setTimeout(() => {
              if (!gitInfo.value.version && gitInfo.value.source !== 'local') {
                simulateClickEffect('btn-install-git')
                installGit()
                  .then(() => {
                    setTimeout(doNodeInstall, 3000)
                  })
                  .catch(() => {
                    // Git安装失败也继续尝试安装Node，也许系统后续不需要Git
                    setTimeout(doNodeInstall, 3000)
                  })
              } else {
                updateOneClickMessage(t('oneClick.gitInstalled'))
                setTimeout(doNodeInstall, 3000)
              }
            }, 500) // 留出一点滚动动画的时间
          }, 300)
        }

        if (route.query.action === 'one_click_setup') {
          // 在开始整个自动化流程前等待3秒（此时弹窗中显示 AI自动化执行中 的初始缓冲文本）
          setTimeout(() => {
            checkAndInstallGit()
          }, 3000)
        } else if (route.query.action === 'install_node') {
          doNodeInstall()
          // 移除URL参数
          router.replace({ query: {} })
        }
      }
    })
  })
  checkNpm()
  checkNodeBoth()
  checkGitBoth()
  checkElevation()
  checkSystemCpuCores()
})

const openLink = (url: string) => {
  open(url).catch((err: any) => {
    console.error('Failed to open URL:', err)
    toast.error(t('settings.loadFailed'))
  })
}

// ─── 公网模式二次确认弹窗 ──────────────────────────────────────────────────
const SKIP_PUBLIC_CONFIRM_KEY = 'launch_public_skip_confirm'
const showPublicConfirm = ref(false)

const handleLaunchModeChange = (mode: AppConfig['launchMode']) => {
  if (mode === 'public') {
    const skip = localStorage.getItem(SKIP_PUBLIC_CONFIRM_KEY)
    if (skip === 'true') {
      config.value.launchMode = 'public'
    } else {
      showPublicConfirm.value = true
    }
  } else {
    config.value.launchMode = mode
  }
}

const confirmPublicMode = () => {
  config.value.launchMode = 'public'
  showPublicConfirm.value = false
}

const neverAskPublicMode = () => {
  localStorage.setItem(SKIP_PUBLIC_CONFIRM_KEY, 'true')
  config.value.launchMode = 'public'
  showPublicConfirm.value = false
}

const cancelPublicMode = () => {
  showPublicConfirm.value = false
}

// ─── 代理设置弹窗 ──────────────────────────────────────────────────────────
const showProxyDialog = ref(false)
// 弹窗内的临时编辑状态，打开时从 config 复制过来
const proxyDialogMode = ref<'none' | 'system' | 'custom'>('none')
const proxyDialogHost = ref('127.0.0.1')
const proxyDialogPort = ref(7890)

// 系统代理信息（从注册表读取）
const systemProxyInfo = ref<{ server: string; enabled: boolean } | null>(null)

// 测试状态
const proxyTestStatus = ref<'idle' | 'testing' | 'success' | 'failed'>('idle')
const proxyTestLatency = ref(0)
const proxyTestError = ref('')

// GitHub 测试结果类型
interface GithubTestResult {
  key: string
  name: string
  url: string
  success: boolean
  latency: number | null // Rust 返回 Option<u64>
  error: string | null
  warning: string | null // 警告消息，有值时表示加速地址可用但无法加速特定资源
}

// GitHub 加速测试状态（测试选中的加速地址）
const githubProxyTestStatus = ref<'idle' | 'testing' | 'success' | 'failed'>('idle')
const githubProxyTestResults = ref<GithubTestResult[]>([])
let githubProxyTestTimer: ReturnType<typeof setTimeout> | null = null

// GitHub 直连测试状态（受代理设置影响）
const githubTestStatus = ref<'idle' | 'testing' | 'success' | 'failed'>('idle')
const githubTestResults = ref<GithubTestResult[]>([])
let githubTestTimer: ReturnType<typeof setTimeout> | null = null

const openProxyDialog = async () => {
  proxyDialogMode.value = config.value.networkProxy.mode
  proxyDialogHost.value = config.value.networkProxy.host || '127.0.0.1'
  proxyDialogPort.value = config.value.networkProxy.port || 7890
  proxyTestStatus.value = 'idle'
  // 读取系统代理信息
  systemProxyInfo.value = await invoke<{ server: string; enabled: boolean } | null>('get_system_proxy_info')
  showProxyDialog.value = true
}

const closeProxyDialog = () => {
  showProxyDialog.value = false
}

const saveProxyDialog = () => {
  config.value.networkProxy.mode = proxyDialogMode.value
  config.value.networkProxy.host = proxyDialogHost.value
  config.value.networkProxy.port = proxyDialogPort.value
  showProxyDialog.value = false
  // watch(config) 会自动触发 saveConfig
}

// 系统代理的 host / port（从 systemProxyInfo.server 解析，仅供展示用）
const systemProxyDisplayHost = computed(() => {
  if (!systemProxyInfo.value?.server) return ''
  const s = systemProxyInfo.value.server
  // 可能含协议前缀 "http=host:port;..." 或直接 "host:port"
  const addr = s.includes('=')
    ? (s
        .split(';')
        .find(p => p.startsWith('https=') || p.startsWith('http='))
        ?.split('=')[1] ??
      s.split(';')[0]?.split('=')[1] ??
      s)
    : s
  return addr.split(':')[0] ?? addr
})
const systemProxyDisplayPort = computed(() => {
  if (!systemProxyInfo.value?.server) return ''
  const s = systemProxyInfo.value.server
  const addr = s.includes('=')
    ? (s
        .split(';')
        .find(p => p.startsWith('https=') || p.startsWith('http='))
        ?.split('=')[1] ??
      s.split(';')[0]?.split('=')[1] ??
      s)
    : s
  return addr.split(':')[1] ?? ''
})

// 翻译测试项名称
const translateTestItem = (key: string) => {
  const keyMap: Record<string, string> = {
    raw: 'settings.testItemFileAccess',
    homepage: 'settings.testItemHomepage',
    repo: 'settings.testItemRepo',
    api: 'settings.testItemApi',
    speed: 'settings.testItemSpeed',
  }
  return t(keyMap[key] || key)
}

// 翻译速度等级（保留数值）
const translateSpeedWarning = (warning: string) => {
  // 提取速度数值
  const speedMatch = warning.match(/\(([\d.]+)\s*MB\/s\)/)
  const speedValue = speedMatch ? speedMatch[1] : ''
  const speedUnit = ' MB/s'

  if (warning.includes('太慢')) return `${t('settings.speedTooSlow')} (${speedValue}${speedUnit})`
  if (warning.includes('速度慢')) return `${t('settings.speedSlow')} (${speedValue}${speedUnit})`
  if (warning.includes('速度正常')) return `${t('settings.speedNormal')} (${speedValue}${speedUnit})`
  if (warning.includes('速度很快')) return `${t('settings.speedFast')} (${speedValue}${speedUnit})`
  if (warning.includes('速度极快')) return `${t('settings.speedVeryFast')} (${speedValue}${speedUnit})`
  return warning
}

const testProxy = async () => {
  if (proxyTestStatus.value === 'testing') return
  proxyTestStatus.value = 'testing'
  proxyTestError.value = ''
  try {
    const latency = await invoke<number>('test_network_proxy', {
      mode: proxyDialogMode.value,
      host: proxyDialogHost.value,
      port: proxyDialogPort.value,
    })
    proxyTestLatency.value = latency
    proxyTestStatus.value = 'success'
  } catch (e: any) {
    proxyTestError.value = String(e)
    proxyTestStatus.value = 'failed'
  }
}

// 测试 GitHub 加速连接（使用选中的加速地址，不包含 API 测试）
const testGithubConnection = async () => {
  if (githubProxyTestStatus.value === 'testing') return
  githubProxyTestStatus.value = 'testing'
  githubProxyTestResults.value = []
  try {
    // 使用当前选中的加速地址
    const proxyUrl = config.value.githubProxy.url || 'https://ghfast.top/'
    const result = await invoke<GithubTestResult[]>('test_github_multi', {
      mode: 'accelerate',
      host: proxyUrl,
      port: 0,
      includeApi: false, // 加速区域不测试 API
    })
    githubProxyTestResults.value = result
    // 只要有一个成功就算成功
    const hasSuccess = githubProxyTestResults.value.some(r => r.success)
    githubProxyTestStatus.value = hasSuccess ? 'success' : 'failed'
  } catch (e: any) {
    githubProxyTestStatus.value = 'failed'
    // 检查是否是 Git 未安装的错误
    const errorMsg = String(e)
    if (errorMsg.includes('Git not found')) {
      toast.error(t('settings.gitNotFoundForTest'))
    }
  }
  // 5秒后重置状态
  if (githubProxyTestTimer) clearTimeout(githubProxyTestTimer)
  githubProxyTestTimer = setTimeout(() => {
    githubProxyTestStatus.value = 'idle'
  }, 5000)
}

// 测试直连（受代理设置影响：代理开则走代理，代理关则直连，包含 API 测试）
const testDirectConnection = async () => {
  if (githubTestStatus.value === 'testing') return
  githubTestStatus.value = 'testing'
  githubTestResults.value = []
  try {
    // 根据代理模式选择对应的测试模式
    let testMode = 'none'
    if (config.value.networkProxy.mode === 'system') {
      testMode = 'system'
    } else if (config.value.networkProxy.mode === 'custom') {
      testMode = 'custom'
    }
    const result = await invoke<GithubTestResult[]>('test_github_multi', {
      mode: testMode,
      host: config.value.networkProxy.host,
      port: config.value.networkProxy.port,
      includeApi: true, // 包含 API 测试
    })
    githubTestResults.value = result
    // 只要有一个成功就算成功
    const hasSuccess = githubTestResults.value.some(r => r.success)
    githubTestStatus.value = hasSuccess ? 'success' : 'failed'
  } catch (e: any) {
    console.error('直连测试失败:', e)
    githubTestStatus.value = 'failed'
  }
  // 5秒后重置状态
  if (githubTestTimer) clearTimeout(githubTestTimer)
  githubTestTimer = setTimeout(() => {
    githubTestStatus.value = 'idle'
  }, 5000)
}
</script>

<template>
  <div class="flex flex-col h-full">
    <h1 class="text-2xl font-bold mb-6 px-1">{{ t('settings.title') }}</h1>

    <!-- Tabs -->
    <div class="flex space-x-1 bg-slate-100 dark:bg-slate-800 p-1 rounded-xl w-fit mb-6 shrink-0">
      <button
        :class="[
          'px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 flex items-center gap-2',
          activeTab === 'general'
            ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-sm'
            : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-700/50',
        ]"
        @click="activeTab = 'general'"
      >
        <PhPalette :size="16" weight="duotone" />
        {{ t('settings.general') }}
      </button>
      <button
        :class="[
          'px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 flex items-center gap-2',
          activeTab === 'credits'
            ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-sm'
            : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-700/50',
        ]"
        @click="activeTab = 'credits'"
      >
        <PhHeart :size="16" weight="duotone" />
        {{ t('settings.credits') }}
      </button>
      <button
        :class="[
          'px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 flex items-center gap-2',
          activeTab === 'about'
            ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-sm'
            : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-700/50',
        ]"
        @click="activeTab = 'about'"
      >
        <PhInfo :size="16" weight="duotone" />
        {{ t('settings.about') }}
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto px-1 pb-10 min-h-0 relative">
      <!-- Loading State -->
      <div
        v-if="loading"
        class="absolute inset-0 flex flex-col items-center justify-center bg-slate-50/50 dark:bg-slate-900/50 backdrop-blur-sm z-10"
      >
        <PhArrowsClockwise :size="48" class="animate-spin mb-4 text-blue-500/80" weight="duotone" />
        <p class="text-sm font-medium text-slate-500 dark:text-slate-400 animate-pulse">
          {{ t('settings.loadingConfig') }}
        </p>
      </div>

      <!-- General Settings -->
      <div v-if="activeTab === 'general'" class="space-y-8 animate-in fade-in slide-in-from-bottom-2 duration-300">
        <!-- Basic Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2">
            <PhSliders :size="20" class="text-teal-500" weight="duotone" />
            {{ t('settings.basic') }}
          </h2>

          <div
            class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class="w-8 h-8 rounded-lg bg-teal-50 dark:bg-teal-900/30 flex items-center justify-center text-teal-500"
                >
                  <PhArrowsClockwise :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ t('settings.scanCpuCores') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    {{ t('settings.scanCpuCoresDesc') }}
                  </div>
                </div>
              </div>
              <select
                v-model="config.scanCpuCores"
                :disabled="systemCpuCores === 0"
                class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 min-w-[140px] outline-none transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <template v-if="systemCpuCores === 0">
                  <option :value="config.scanCpuCores">{{ t('common.loading') }}</option>
                </template>
                <template v-else>
                  <option :value="null">{{ t('settings.scanCpuCoresAuto') }}</option>
                  <option v-for="n in systemCpuCores" :key="n" :value="n">{{ n }}</option>
                </template>
              </select>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <!-- Launch Mode -->
            <div class="flex items-start justify-between gap-4">
              <div class="flex items-center gap-3 shrink-0">
                <div
                  class="w-8 h-8 rounded-lg bg-violet-50 dark:bg-violet-900/30 flex items-center justify-center text-violet-500"
                >
                  <PhTerminalWindow :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ t('settings.launchMode') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    {{ t('settings.launchModeDesc') }}
                  </div>
                </div>
              </div>
              <!-- Tag 单选选择器 -->
              <div
                class="flex items-center gap-1 bg-slate-100 dark:bg-slate-700 rounded-lg p-1 shrink-0 flex-wrap justify-end"
              >
                <button
                  :class="[
                    'px-3 py-1.5 rounded-md text-xs font-medium transition-all',
                    config.launchMode === 'normal'
                      ? 'bg-white dark:bg-slate-600 text-slate-800 dark:text-slate-100 shadow-sm'
                      : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200',
                  ]"
                  :title="t('settings.launchModeNormalDesc')"
                  @click="handleLaunchModeChange('normal')"
                >
                  {{ t('settings.launchModeNormal') }}
                </button>
                <button
                  :class="[
                    'px-3 py-1.5 rounded-md text-xs font-medium transition-all',
                    config.launchMode === 'desktop'
                      ? 'bg-white dark:bg-slate-600 text-slate-800 dark:text-slate-100 shadow-sm'
                      : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200',
                  ]"
                  :title="t('settings.launchModeDesktopDesc')"
                  @click="handleLaunchModeChange('desktop')"
                >
                  {{ t('settings.launchModeDesktop') }}
                </button>
                <button
                  :class="[
                    'px-3 py-1.5 rounded-md text-xs font-medium transition-all',
                    config.launchMode === 'lan'
                      ? 'bg-white dark:bg-slate-600 text-emerald-700 dark:text-emerald-400 shadow-sm'
                      : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200',
                  ]"
                  :title="t('settings.launchModeLanDesc')"
                  @click="handleLaunchModeChange('lan')"
                >
                  {{ t('settings.launchModeLan') }}
                </button>
                <button
                  :class="[
                    'px-3 py-1.5 rounded-md text-xs font-medium transition-all',
                    config.launchMode === 'public'
                      ? 'bg-white dark:bg-slate-600 text-red-600 dark:text-red-400 shadow-sm'
                      : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200',
                  ]"
                  :title="t('settings.launchModePublicDesc')"
                  @click="handleLaunchModeChange('public')"
                >
                  {{ t('settings.launchModePublic') }}
                </button>
                <button
                  :class="[
                    'px-3 py-1.5 rounded-md text-xs font-medium transition-all',
                    config.launchMode === 'debug'
                      ? 'bg-white dark:bg-slate-600 text-violet-600 dark:text-violet-400 shadow-sm'
                      : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200',
                  ]"
                  :title="t('settings.launchModeDebugDesc')"
                  @click="handleLaunchModeChange('debug')"
                >
                  {{ t('settings.launchModeDebug') }}
                </button>
              </div>
            </div>
            <!-- 当前选中模式的说明文字 -->
            <div class="pl-11 text-xs -mt-2">
              <span v-if="config.launchMode === 'normal'" class="text-slate-400 dark:text-slate-500">{{
                t('settings.launchModeNormalDesc')
              }}</span>
              <span v-else-if="config.launchMode === 'desktop'" class="text-violet-500">{{
                t('settings.launchModeDesktopDesc')
              }}</span>
              <span v-else-if="config.launchMode === 'lan'" class="text-emerald-600 dark:text-emerald-400">{{
                t('settings.launchModeLanDesc')
              }}</span>
              <span v-else-if="config.launchMode === 'public'" class="text-red-500 font-medium flex items-center gap-1"
                ><PhWarning :size="12" weight="fill" />{{ t('settings.launchModePublicDesc') }}</span
              >
              <span v-else-if="config.launchMode === 'debug'" class="text-amber-500">{{
                t('settings.launchModeDebugDesc')
              }}</span>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <!-- Data Mode -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class="w-8 h-8 rounded-lg bg-orange-50 dark:bg-orange-900/30 flex items-center justify-center text-orange-500"
                >
                  <PhPackage :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ t('settings.dataMode') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    {{ t('settings.dataModeDesc') }}
                  </div>
                </div>
              </div>
              <select
                v-model="config.dataMode"
                class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 min-w-[140px] outline-none transition-all"
              >
                <option value="global">{{ t('settings.dataModeGlobal') }}</option>
                <option value="local">{{ t('settings.dataModeLocal') }}</option>
              </select>
            </div>
          </div>
        </section>

        <!-- Interface Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2">
            <PhPalette :size="20" class="text-blue-500" weight="duotone" />
            {{ t('settings.interface') }}
          </h2>

          <div
            class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm"
          >
            <!-- Language -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class="w-8 h-8 rounded-lg bg-blue-50 dark:bg-blue-900/30 flex items-center justify-center text-blue-500"
                >
                  <PhGlobe :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ t('settings.language') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    {{ t('settings.languageDesc') }}
                  </div>
                </div>
              </div>
              <select
                v-model="config.lang"
                class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 min-w-[140px] outline-none transition-all"
              >
                <option value="auto">{{ t('settings.languageAuto') }}</option>
                <option value="zh-CN">{{ t('settings.languageZhCN') }}</option>
                <option value="en-US">{{ t('settings.languageEnUS') }}</option>
              </select>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <!-- Theme -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class="w-8 h-8 rounded-lg bg-purple-50 dark:bg-purple-900/30 flex items-center justify-center text-purple-500"
                >
                  <PhPalette :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ t('settings.theme') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    {{ t('settings.themeDesc') }}
                  </div>
                </div>
              </div>
              <select
                v-model="config.theme"
                class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 min-w-[140px] outline-none transition-all"
              >
                <option value="auto">{{ t('settings.themeAuto') }}</option>
                <option value="light">{{ t('settings.themeLight') }}</option>
                <option value="dark">{{ t('settings.themeDark') }}</option>
              </select>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class="w-8 h-8 rounded-lg bg-indigo-50 dark:bg-indigo-900/30 flex items-center justify-center text-indigo-500"
                >
                  <PhBrowserWindow :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ t('settings.rememberWindow') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    {{ t('settings.rememberWindowDesc') }}
                  </div>
                </div>
              </div>
              <label class="inline-flex items-center cursor-pointer">
                <input v-model="config.rememberWindowPosition" type="checkbox" class="sr-only peer" />
                <div
                  class="relative w-11 h-6 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"
                ></div>
              </label>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <!-- Animations Toggle -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class="w-8 h-8 rounded-lg bg-pink-50 dark:bg-pink-900/30 flex items-center justify-center text-pink-500"
                >
                  <PhPalette :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ t('settings.animations') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    {{ t('settings.animationsDesc') }}
                  </div>
                </div>
              </div>
              <label class="inline-flex items-center cursor-pointer">
                <input v-model="config.enableAnimations" type="checkbox" class="sr-only peer" />
                <div
                  class="relative w-11 h-6 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-pink-500"
                ></div>
              </label>
            </div>
          </div>
        </section>

        <!-- Git Settings -->
        <section id="git-settings" class="space-y-4">
          <div class="flex items-center justify-between">
            <h2
              class="text-lg font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2 w-fit relative group"
            >
              <PhGitBranch :size="20" class="text-orange-600" weight="duotone" />
              {{ t('settings.git') }}
              <PhInfo :size="16" class="text-slate-400 cursor-help ml-1" />
              <div
                class="absolute left-[calc(100%+0.5rem)] top-1/2 -translate-y-1/2 w-max max-w-[200px] sm:max-w-xs bg-slate-800 text-white text-xs rounded-lg p-3 shadow-lg opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all z-20 whitespace-normal text-left"
              >
                {{ t('settings.gitTooltip') }}
              </div>
            </h2>
            <button
              :disabled="gitInstallState.installing"
              class="p-1.5 text-slate-400 hover:text-orange-500 hover:bg-orange-50 dark:hover:bg-orange-900/30 rounded-md transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
              :title="t('settings.refreshEnv', '刷新环境检测')"
              @click="refreshGitEnv"
            >
              <PhArrowsClockwise :size="16" weight="duotone" />
            </button>
          </div>

          <div
            class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class="w-8 h-8 rounded-lg bg-orange-50 dark:bg-orange-900/30 flex items-center justify-center text-orange-600"
                >
                  <PhGitBranch :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ t('settings.gitEnv') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    <span v-if="gitInfo.version">
                      {{ t('settings.gitVersion') }}: {{ gitInfo.version }} ({{
                        gitInfo.source === 'local' ? t('settings.gitLocal') : t('settings.gitSystem')
                      }})
                      <div v-if="gitInfo.path" class="mt-1 text-slate-400 dark:text-slate-500 break-all select-all">
                        {{ t('settings.gitPath') }}: {{ gitInfo.path }}
                      </div>
                    </span>
                    <span v-else>{{ t('settings.gitNotFound') }}</span>
                  </div>
                </div>
              </div>

              <!-- Git 按钮：当前用系统 Git 且内置 Git 已存在时隐藏（用户主动选了系统 Git，不需要再装） -->
              <div v-if="!(gitInfo.source === 'system' && bothGitInfo.local)" class="flex items-center gap-2">
                <button
                  v-if="gitInstallState.installing"
                  class="p-1.5 text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200 bg-slate-100 hover:bg-slate-200 dark:bg-slate-700/50 dark:hover:bg-slate-700 rounded-md transition-colors"
                  :title="t('common.logs', '日志')"
                  @click="showLogs('git')"
                >
                  <PhTerminalWindow :size="16" weight="duotone" />
                </button>
                <!-- 安装中：取消按钮 -->
                <button
                  v-if="gitInstallState.installing"
                  class="px-3 py-1.5 text-xs font-medium bg-red-50 dark:bg-red-900/30 text-red-600 rounded-md hover:bg-red-100 dark:hover:bg-red-900/50 transition-colors flex items-center gap-1"
                  @click="cancelGitInstall"
                >
                  <PhX :size="14" />
                  {{ t('settings.gitInstallCancel', '取消安装') }}
                </button>
                <!-- 空闲中：安装/重装按钮 -->
                <button
                  v-else
                  id="btn-install-git"
                  class="px-3 py-1.5 text-xs font-medium bg-orange-50 dark:bg-orange-900/30 text-orange-600 rounded-md hover:bg-orange-100 dark:hover:bg-orange-900/50 transition-colors flex items-center gap-1"
                  @click="installGit"
                >
                  <PhDownloadSimple :size="14" />
                  {{
                    gitInfo.source === 'local'
                      ? t('settings.gitReinstall')
                      : gitInfo.source === 'system'
                        ? t('settings.gitInstallLocal')
                        : t('settings.gitInstall')
                  }}
                </button>
              </div>
            </div>

            <div
              v-if="gitInstallState.installing"
              class="space-y-2 pt-2 border-t border-slate-100 dark:border-slate-700"
            >
              <div class="flex justify-between text-xs text-slate-500 dark:text-slate-400">
                <span class="flex items-center gap-1.5">
                  <PhArrowsClockwise :size="12" class="animate-spin shrink-0" />
                  {{ gitInstallState.progress.log || t('settings.gitInstalling') }}
                </span>
                <span>{{ Math.round(gitInstallState.progress.progress * 100) }}%</span>
              </div>
              <div class="w-full bg-slate-100 dark:bg-slate-700 rounded-full h-1.5 overflow-hidden">
                <div
                  class="bg-orange-500 h-1.5 rounded-full transition-all duration-300"
                  :style="{ width: `${gitInstallState.progress.progress * 100}%` }"
                ></div>
              </div>
            </div>

            <!-- 系统/内置 Git 切换（系统和内置 Git 同时存在时才显示） -->
            <template v-if="bothGitInfo.local && bothGitInfo.system">
              <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <div
                    class="w-8 h-8 rounded-lg bg-orange-50 dark:bg-orange-900/30 flex items-center justify-center text-orange-600"
                  >
                    <PhSliders :size="18" weight="duotone" />
                  </div>
                  <div>
                    <div class="font-medium text-slate-700 dark:text-slate-300">
                      {{ t('settings.gitSourceToggle') }}
                    </div>
                    <div class="text-xs text-slate-500 dark:text-slate-400">
                      {{ t('settings.gitSourceToggleDesc') }}
                    </div>
                    <div class="mt-1 text-[10px] text-slate-400 dark:text-slate-500 space-y-0.5">
                      <div v-if="bothGitInfo.system" class="flex items-center gap-1">
                        <PhDesktop :size="11" class="shrink-0" />
                        <span
                          >{{ t('settings.gitSystem') }}: {{ bothGitInfo.system.version }} —
                          {{ bothGitInfo.system.path }}</span
                        >
                      </div>
                      <div v-if="bothGitInfo.local" class="flex items-center gap-1">
                        <PhPackage :size="11" class="shrink-0" />
                        <span
                          >{{ t('settings.gitLocal') }}: {{ bothGitInfo.local.version }} —
                          {{ bothGitInfo.local.path }}</span
                        >
                      </div>
                    </div>
                  </div>
                </div>
                <div class="flex items-center gap-1 bg-slate-100 dark:bg-slate-700 rounded-lg p-1 shrink-0">
                  <button
                    :class="[
                      'px-3 py-1.5 rounded-md text-xs font-medium transition-all',
                      config.useSystemGit
                        ? 'bg-white dark:bg-slate-600 text-slate-800 dark:text-slate-100 shadow-sm'
                        : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200',
                    ]"
                    @click="switchGitSource(true)"
                  >
                    {{ t('settings.gitUseSystem') }}
                  </button>
                  <button
                    :class="[
                      'px-3 py-1.5 rounded-md text-xs font-medium transition-all',
                      !config.useSystemGit
                        ? 'bg-white dark:bg-slate-600 text-slate-800 dark:text-slate-100 shadow-sm'
                        : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200',
                    ]"
                    @click="switchGitSource(false)"
                  >
                    {{ t('settings.gitUseLocal') }}
                  </button>
                </div>
              </div>
            </template>
          </div>
        </section>

        <!-- NodeJs Settings -->
        <section id="node-settings" class="space-y-4">
          <div class="flex items-center justify-between">
            <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2">
              <PhPackage :size="20" class="text-green-600" weight="duotone" />
              {{ t('settings.nodejs') }}
            </h2>
            <button
              :disabled="nodeInstallState.installing"
              class="p-1.5 text-slate-400 hover:text-green-500 hover:bg-green-50 dark:hover:bg-green-900/30 rounded-md transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
              :title="t('settings.refreshEnv', '刷新环境检测')"
              @click="refreshNodeEnv"
            >
              <PhArrowsClockwise :size="16" weight="duotone" />
            </button>
          </div>

          <div
            class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class="w-8 h-8 rounded-lg bg-green-50 dark:bg-green-900/30 flex items-center justify-center text-green-600"
                >
                  <PhPackage :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ t('settings.nodejsEnv') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    <span v-if="nodeInfo.version">
                      {{ t('settings.nodejsVersion') }}: {{ nodeInfo.version }} ({{
                        nodeInfo.source === 'local' ? t('settings.nodejsLocal') : t('settings.nodejsSystem')
                      }})
                      <div
                        v-if="isNodeVersionValid && nodeInfo.path"
                        class="mt-1 text-slate-400 dark:text-slate-500 break-all select-all"
                      >
                        {{ t('settings.nodejsPath') }}: {{ nodeInfo.path }}
                      </div>
                      <div v-if="!isNodeVersionValid" class="mt-1 text-red-500">
                        {{ t('settings.nodejsLowVersion', { required: `v${requiredNodeMajor}` }) }}
                      </div>
                    </span>
                    <span v-else>{{ t('settings.nodejsNotFound') }}</span>
                  </div>
                </div>
              </div>

              <!-- Node 按钮：
                   - 当前用系统 Node 且已有内置 Node → 隐藏（用户主动选了系统Node，不需要再装）
                   - 当前用内置 Node → 显示「重新安装」
                   - 无内置 Node，当前系统 Node → 显示「安装内置 Node」
                   - 完全没有 Node → 显示「立即安装」
              -->
              <div v-if="!(nodeInfo.source === 'system' && bothNodeInfo.local)" class="flex items-center gap-2">
                <button
                  v-if="nodeInstallState.installing"
                  class="p-1.5 text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200 bg-slate-100 hover:bg-slate-200 dark:bg-slate-700/50 dark:hover:bg-slate-700 rounded-md transition-colors"
                  :title="t('common.logs', '日志')"
                  @click="showLogs('node')"
                >
                  <PhTerminalWindow :size="16" weight="duotone" />
                </button>
                <!-- 安装中：取消按钮 -->
                <button
                  v-if="nodeInstallState.installing"
                  class="px-3 py-1.5 text-xs font-medium bg-red-50 dark:bg-red-900/30 text-red-600 rounded-md hover:bg-red-100 dark:hover:bg-red-900/50 transition-colors flex items-center gap-1"
                  @click="cancelNodeInstall"
                >
                  <PhX :size="14" />
                  {{ t('settings.nodejsInstallCancel', '取消安装') }}
                </button>
                <!-- 空闲中：安装/重装按钮 -->
                <button
                  v-else
                  id="btn-install-node"
                  class="px-3 py-1.5 text-xs font-medium bg-green-50 dark:bg-green-900/30 text-green-600 rounded-md hover:bg-green-100 dark:hover:bg-green-900/50 transition-colors flex items-center gap-1"
                  @click="installNode"
                >
                  <PhDownloadSimple :size="14" />
                  {{
                    nodeInfo.source === 'local'
                      ? t('settings.nodejsReinstall')
                      : nodeInfo.source === 'system'
                        ? t('settings.nodejsInstallLocal')
                        : nodeInfo.version
                          ? t('settings.nodejsReinstall')
                          : t('settings.nodejsInstall')
                  }}
                </button>
              </div>
            </div>

            <div
              v-if="nodeInstallState.installing"
              class="space-y-2 pt-2 border-t border-slate-100 dark:border-slate-700"
            >
              <div class="flex justify-between text-xs text-slate-500 dark:text-slate-400">
                <span class="flex items-center gap-1.5">
                  <PhArrowsClockwise :size="12" class="animate-spin shrink-0" />
                  {{ nodeInstallState.progress.log || t('settings.nodejsInstalling') }}
                </span>
                <span>{{ Math.round(nodeInstallState.progress.progress * 100) }}%</span>
              </div>
              <div class="w-full bg-slate-100 dark:bg-slate-700 rounded-full h-1.5 overflow-hidden">
                <div
                  class="bg-green-500 h-1.5 rounded-full transition-all duration-300"
                  :style="{ width: `${nodeInstallState.progress.progress * 100}%` }"
                ></div>
              </div>
            </div>

            <!-- 系统/内置 Node 切换（系统和内置 Node 同时存在时才显示） -->
            <template v-if="bothNodeInfo.local && bothNodeInfo.system">
              <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <div
                    class="w-8 h-8 rounded-lg bg-teal-50 dark:bg-teal-900/30 flex items-center justify-center text-teal-600"
                  >
                    <PhSliders :size="18" weight="duotone" />
                  </div>
                  <div>
                    <div class="font-medium text-slate-700 dark:text-slate-300">
                      {{ t('settings.nodejsSourceToggle') }}
                    </div>
                    <div class="text-xs text-slate-500 dark:text-slate-400">
                      {{ t('settings.nodejsSourceToggleDesc2') }}
                    </div>
                    <div class="mt-1 text-[10px] text-slate-400 dark:text-slate-500 space-y-0.5">
                      <div v-if="bothNodeInfo.system" class="flex items-center gap-1">
                        <PhDesktop :size="11" class="shrink-0" />
                        <span
                          >{{ t('settings.nodejsSystem') }}: {{ bothNodeInfo.system.version }} —
                          {{ bothNodeInfo.system.path }}</span
                        >
                      </div>
                      <div v-if="bothNodeInfo.local" class="flex items-center gap-1">
                        <PhPackage :size="11" class="shrink-0" />
                        <span
                          >{{ t('settings.nodejsLocal') }}: {{ bothNodeInfo.local.version }} —
                          {{ bothNodeInfo.local.path }}</span
                        >
                      </div>
                    </div>
                  </div>
                </div>
                <div class="flex items-center gap-1 bg-slate-100 dark:bg-slate-700 rounded-lg p-1 shrink-0">
                  <button
                    :class="[
                      'px-3 py-1.5 rounded-md text-xs font-medium transition-all',
                      config.useSystemNode
                        ? 'bg-white dark:bg-slate-600 text-slate-800 dark:text-slate-100 shadow-sm'
                        : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200',
                    ]"
                    @click="switchNodeSource(true)"
                  >
                    {{ t('settings.nodejsUseSystem') }}
                  </button>
                  <button
                    :class="[
                      'px-3 py-1.5 rounded-md text-xs font-medium transition-all',
                      !config.useSystemNode
                        ? 'bg-white dark:bg-slate-600 text-slate-800 dark:text-slate-100 shadow-sm'
                        : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200',
                    ]"
                    @click="switchNodeSource(false)"
                  >
                    {{ t('settings.nodejsUseLocal') }}
                  </button>
                </div>
              </div>
            </template>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <!-- NPM Info -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class="w-8 h-8 rounded-lg bg-red-50 dark:bg-red-900/30 flex items-center justify-center text-red-500"
                >
                  <PhPackage :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ t('settings.npmEnv') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    <span v-if="npmInfo.version">
                      {{ t('settings.npmVersion') }}: {{ npmInfo.version }} ({{
                        npmInfo.source === 'local' ? t('settings.nodejsLocal') : t('settings.nodejsSystem')
                      }})
                      <div v-if="npmInfo.path" class="mt-1 text-slate-400 dark:text-slate-500 break-all select-all">
                        {{ t('settings.nodejsPath') }}: {{ npmInfo.path }}
                      </div>
                    </span>
                    <span v-else>{{ t('settings.npmNotFound') }}</span>
                  </div>
                </div>
              </div>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <!-- NPM Registry Selection -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class="w-8 h-8 rounded-lg bg-orange-50 dark:bg-orange-900/30 flex items-center justify-center text-orange-500"
                >
                  <PhGlobe :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ t('settings.npmRegistry') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    {{ t('settings.npmRegistryDesc') }}
                  </div>
                </div>
              </div>
              <select
                v-model="config.npmRegistry"
                class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-xs rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-48 p-2"
                @change="saveConfig"
              >
                <option v-for="registry in npmRegistries" :key="registry.url" :value="registry.url">
                  {{ registry.name }}
                </option>
              </select>
            </div>
            <div class="text-[10px] text-slate-400 pl-11">
              {{ t('settings.currentAddress') }}: {{ config.npmRegistry }}
            </div>
          </div>
        </section>

        <!-- Privilege Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2">
            <PhShield :size="20" class="text-indigo-600" weight="duotone" />
            {{ t('settings.privilege') }}
          </h2>

          <div
            class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 shadow-sm"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  :class="[
                    'w-8 h-8 rounded-lg flex items-center justify-center',
                    isElevated
                      ? 'bg-indigo-50 dark:bg-indigo-900/30 text-indigo-600'
                      : 'bg-slate-50 dark:bg-slate-900/30 text-slate-400',
                  ]"
                >
                  <PhShieldCheck v-if="isElevated" :size="18" weight="duotone" />
                  <PhShield v-else :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ isElevated ? t('settings.isElevated') : t('settings.nonElevated') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    {{ t('settings.elevateDesc') }}
                  </div>
                </div>
              </div>

              <div v-if="!isElevated">
                <button
                  :disabled="elevating"
                  class="px-3 py-1.5 text-xs font-medium bg-indigo-50 dark:bg-indigo-900/30 text-indigo-600 rounded-md hover:bg-indigo-100 dark:hover:bg-indigo-900/50 transition-colors flex items-center gap-1 disabled:opacity-50"
                  @click="requestElevation"
                >
                  <PhArrowsClockwise v-if="elevating" :size="14" class="animate-spin" />
                  <PhShield v-else :size="14" />
                  {{ elevating ? t('settings.elevating') : t('settings.elevateBtn') }}
                </button>
              </div>
            </div>
          </div>
        </section>

        <!-- Github Proxy Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2">
            <PhGithubLogo :size="20" class="text-slate-700 dark:text-slate-400" weight="duotone" />
            {{ t('settings.github') }}
          </h2>

          <div
            class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm"
          >
            <!-- Toggle -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class="w-8 h-8 rounded-lg bg-slate-100 dark:bg-slate-700 flex items-center justify-center text-slate-600 dark:text-slate-400"
                >
                  <PhGithubLogo :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ t('settings.githubToggle') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    {{ t('settings.githubToggleDesc') }}
                  </div>
                </div>
              </div>
              <label class="inline-flex items-center cursor-pointer">
                <input v-model="config.githubProxy.enable" type="checkbox" class="sr-only peer" />
                <div
                  class="relative w-11 h-6 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"
                ></div>
              </label>
            </div>

            <!-- 代理冲突警告提示 -->
            <div
              v-if="config.githubProxy.enable && config.networkProxy.mode !== 'none'"
              class="flex items-start gap-2 p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg"
            >
              <PhWarning :size="18" class="text-amber-600 dark:text-amber-500 flex-shrink-0 mt-0.5" />
              <div class="text-xs text-amber-800 dark:text-amber-300 leading-relaxed">
                {{ t('settings.githubProxyConflictWarning') }}
              </div>
            </div>

            <!-- Current URL Display -->
            <div
              class="flex items-center gap-3 p-3 bg-slate-50 dark:bg-slate-900/50 rounded-lg border border-slate-100 dark:border-slate-700"
            >
              <div class="text-sm font-medium text-slate-500 dark:text-slate-400 whitespace-nowrap">
                {{ t('settings.currentAddress') }}:
              </div>
              <div
                v-if="isValidProxyUrl(config.githubProxy.url)"
                class="text-sm text-slate-800 dark:text-slate-300 font-mono truncate select-all"
              >
                {{ config.githubProxy.url }}
              </div>
              <div v-else class="text-sm text-slate-400 dark:text-slate-500 font-mono italic">
                https://ghfast.top/ ({{ t('settings.defaultAddress') }})
              </div>
            </div>

            <!-- GitHub 连接测试 -->
            <div class="flex items-center justify-between pt-2">
              <div class="flex items-center gap-3">
                <h3 class="text-sm font-medium text-slate-700 dark:text-slate-300">
                  {{ t('settings.githubProxyList') }}
                </h3>
                <span
                  v-if="proxyLastFetchTimeDisplay"
                  class="text-[10px] text-slate-400 bg-slate-50 dark:bg-slate-900/50 px-2 py-0.5 rounded-full border border-slate-100 dark:border-slate-700 flex items-center gap-1"
                >
                  {{ t('settings.githubLastSync') }}: {{ proxyLastFetchTimeDisplay }}
                </span>
              </div>
              <div class="flex items-center gap-2">
                <!-- 测试 GitHub 加速连接按钮 -->
                <button
                  :disabled="githubProxyTestStatus === 'testing'"
                  class="text-xs flex items-center gap-1.5 px-3 py-1.5 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  :class="
                    githubProxyTestStatus === 'testing'
                      ? 'bg-slate-100 dark:bg-slate-700 text-slate-500'
                      : githubProxyTestStatus === 'success'
                        ? 'bg-emerald-50 dark:bg-emerald-900/30 text-emerald-600'
                        : githubProxyTestStatus === 'failed'
                          ? 'bg-red-50 dark:bg-red-900/30 text-red-500'
                          : 'bg-slate-50 dark:bg-slate-700 text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-600'
                  "
                  @click="testGithubConnection"
                >
                  <PhArrowsClockwise v-if="githubProxyTestStatus === 'testing'" :size="14" class="animate-spin" />
                  <PhCheckCircle v-else-if="githubProxyTestStatus === 'success'" :size="14" weight="fill" />
                  <PhXCircle v-else-if="githubProxyTestStatus === 'failed'" :size="14" weight="fill" />
                  <PhGlobe v-else :size="14" />
                  <span v-if="githubProxyTestStatus === 'testing'">{{ t('settings.githubTesting') }}</span>
                  <span v-else-if="githubProxyTestStatus === 'idle'">{{ t('settings.githubConnectionTest') }}</span>
                  <span v-else>
                    {{
                      t('settings.testResultCount', {
                        success: githubProxyTestResults.filter(r => r.success).length,
                        total: githubProxyTestResults.length,
                      })
                    }}
                  </span>
                </button>
                <button
                  :disabled="proxyLoading"
                  class="text-xs flex items-center gap-1.5 px-3 py-1.5 bg-blue-50 dark:bg-blue-900/30 text-blue-600 rounded-md hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  @click="fetchProxies(true)"
                >
                  <PhArrowsClockwise :class="{ 'animate-spin': proxyLoading }" :size="14" />
                  {{ proxyLoading ? t('settings.githubRefreshing') : t('settings.githubRefresh') }}
                </button>
              </div>
            </div>
            <!-- 加速测试结果列表 -->
            <div v-if="githubProxyTestResults.length > 0" class="space-y-1.5 mt-2">
              <div
                v-for="result in githubProxyTestResults"
                :key="result.key"
                class="flex items-center justify-between text-xs px-2 py-1 rounded bg-slate-50 dark:bg-slate-900/50"
              >
                <span class="text-slate-600 dark:text-slate-400">{{ translateTestItem(result.key) }}</span>
                <!-- 下载速度测试：根据速度级别显示不同图标 -->
                <span
                  v-if="result.key === 'speed' && result.warning"
                  class="flex items-center gap-1"
                  :title="translateSpeedWarning(result.warning)"
                >
                  <template v-if="result.warning.includes('太慢')">
                    <PhWarning :size="12" weight="fill" class="text-amber-500" />
                    <span class="text-amber-600 dark:text-amber-400">{{ translateSpeedWarning(result.warning) }}</span>
                  </template>
                  <template v-else-if="result.warning.includes('极快')">
                    <PhStar :size="12" weight="fill" class="text-amber-500" />
                    <span class="text-amber-600 dark:text-amber-400">{{ translateSpeedWarning(result.warning) }}</span>
                  </template>
                  <template v-else-if="result.warning.includes('很快')">
                    <PhThumbsUp :size="12" weight="fill" class="text-blue-500" />
                    <span class="text-blue-600 dark:text-blue-400">{{ translateSpeedWarning(result.warning) }}</span>
                  </template>
                  <template v-else>
                    <PhCheckCircle :size="12" weight="fill" class="text-emerald-500" />
                    <span class="text-emerald-600 dark:text-emerald-400">{{
                      translateSpeedWarning(result.warning)
                    }}</span>
                  </template>
                </span>
                <!-- 有警告但不是下载速度测试时显示警告图标 -->
                <span v-else-if="result.warning" class="text-amber-500 flex items-center gap-1" :title="result.warning">
                  <PhWarning :size="12" weight="fill" />
                  {{ result.latency }}ms
                </span>
                <span v-else-if="result.success" class="text-emerald-600 dark:text-emerald-400 flex items-center gap-1">
                  <PhCheckCircle :size="12" weight="fill" />
                  {{ result.latency }}ms
                </span>
                <span v-else class="text-red-500 flex items-center gap-1">
                  <PhXCircle :size="12" weight="fill" />
                  {{ t('settings.testFailed') }}
                </span>
              </div>
            </div>

            <!-- Proxy List -->
            <div v-if="proxies.length > 0" class="space-y-2 max-h-60 overflow-y-auto custom-scrollbar pr-1">
              <div
                v-for="proxy in proxies"
                :key="proxy.url"
                :class="[
                  'flex items-center justify-between p-3 rounded-lg border cursor-pointer transition-all hover:shadow-sm',
                  config.githubProxy.url === proxy.url
                    ? 'bg-blue-50 dark:bg-blue-900/30 border-blue-200 dark:border-blue-800 ring-1 ring-blue-200 dark:ring-blue-800'
                    : 'bg-white dark:bg-slate-900/50 border-slate-100 dark:border-slate-700 hover:border-slate-300 dark:hover:border-slate-600',
                ]"
                @click="selectProxy(proxy.url)"
              >
                <div class="flex items-center gap-3 overflow-hidden">
                  <div
                    :class="[
                      'w-4 h-4 rounded-full flex items-center justify-center shrink-0',
                      config.githubProxy.url === proxy.url ? 'text-blue-600' : 'text-transparent',
                    ]"
                  >
                    <PhCheck :size="12" weight="bold" />
                  </div>
                  <div class="text-sm font-mono truncate text-slate-600 dark:text-slate-400">
                    {{ proxy.url }}
                  </div>
                </div>
                <div class="flex items-center gap-2 shrink-0">
                  <span
                    :class="[
                      'text-xs font-medium px-2 py-0.5 rounded',
                      proxy.latency < 200
                        ? 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400'
                        : proxy.latency < 500
                          ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400'
                          : 'bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400',
                    ]"
                  >
                    {{ proxy.latency }}ms
                  </span>
                </div>
              </div>
            </div>

            <div
              v-else-if="!proxyLoading"
              class="text-center py-8 text-slate-400 text-sm bg-slate-50 dark:bg-slate-900/50 rounded-lg border border-dashed border-slate-200 dark:border-slate-700"
            >
              {{ t('settings.githubEmpty') }}
            </div>

            <div v-else class="py-8 flex justify-center">
              <div class="animate-pulse flex space-x-4 w-full px-4">
                <div class="flex-1 space-y-3 py-1">
                  <div class="h-10 bg-slate-100 dark:bg-slate-700 rounded"></div>
                  <div class="h-10 bg-slate-100 dark:bg-slate-700 rounded"></div>
                  <div class="h-10 bg-slate-100 dark:bg-slate-700 rounded"></div>
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- Network Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2">
            <PhWifiHigh :size="20" class="text-sky-500" weight="duotone" />
            {{ t('settings.network') }}
          </h2>

          <div
            class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-6 space-y-4 shadow-sm"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class="w-8 h-8 rounded-lg bg-sky-50 dark:bg-sky-900/30 flex items-center justify-center text-sky-500"
                >
                  <PhGlobe :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ t('settings.networkProxy') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    <span v-if="config.networkProxy.mode === 'none'">{{ t('settings.proxyStatusNone') }}</span>
                    <span v-else-if="config.networkProxy.mode === 'system'" class="text-sky-500">{{
                      t('settings.proxyStatusSystem')
                    }}</span>
                    <span v-else class="text-sky-500"
                      >{{ t('settings.proxyStatusCustom') }}: {{ config.networkProxy.host }}:{{
                        config.networkProxy.port
                      }}</span
                    >
                  </div>
                </div>
              </div>
              <button
                class="px-3 py-1.5 text-xs font-medium bg-sky-50 dark:bg-sky-900/30 text-sky-600 rounded-md hover:bg-sky-100 dark:hover:bg-sky-900/50 transition-colors flex items-center gap-1"
                @click="openProxyDialog"
              >
                <PhSliders :size="14" />
                {{ t('settings.networkProxyOpen') }}
              </button>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <!-- 直连测试 -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div
                  class="w-8 h-8 rounded-lg bg-slate-100 dark:bg-slate-700 flex items-center justify-center text-slate-600 dark:text-slate-400"
                >
                  <PhGithubLogo :size="18" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">
                    {{ t('settings.githubConnection') }}
                  </div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    {{ t('settings.testDirectConnectionDesc') }}
                  </div>
                </div>
              </div>
              <button
                :disabled="githubTestStatus === 'testing'"
                class="text-xs flex items-center gap-1.5 px-3 py-1.5 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                :class="
                  githubTestStatus === 'testing'
                    ? 'bg-slate-100 dark:bg-slate-700 text-slate-500'
                    : githubTestStatus === 'success'
                      ? 'bg-emerald-50 dark:bg-emerald-900/30 text-emerald-600'
                      : githubTestStatus === 'failed'
                        ? 'bg-red-50 dark:bg-red-900/30 text-red-500'
                        : 'bg-blue-50 dark:bg-blue-900/30 text-blue-600 hover:bg-blue-100 dark:hover:bg-blue-900/50'
                "
                @click="testDirectConnection"
              >
                <PhArrowsClockwise v-if="githubTestStatus === 'testing'" :size="14" class="animate-spin" />
                <PhCheckCircle v-else-if="githubTestStatus === 'success'" :size="14" weight="fill" />
                <PhXCircle v-else-if="githubTestStatus === 'failed'" :size="14" weight="fill" />
                <PhGlobe v-else :size="14" />
                <span v-if="githubTestStatus === 'testing'">{{ t('settings.githubTesting') }}</span>
                <span v-else-if="githubTestStatus === 'idle'">{{ t('settings.testDirectConnection') }}</span>
                <span v-else>
                  {{
                    t('settings.testResultCount', {
                      success: githubTestResults.filter(r => r.success).length,
                      total: githubTestResults.length,
                    })
                  }}
                </span>
              </button>
            </div>
            <!-- 直连测试结果列表 -->
            <div v-if="githubTestResults.length > 0" class="space-y-1.5 mt-2">
              <div
                v-for="result in githubTestResults"
                :key="result.key"
                class="flex items-center justify-between text-xs px-2 py-1 rounded bg-slate-50 dark:bg-slate-900/50"
              >
                <span class="text-slate-600 dark:text-slate-400">{{ translateTestItem(result.key) }}</span>
                <!-- 下载速度测试：优先显示速度信息 -->
                <span v-if="result.key === 'speed' && result.warning" class="flex items-center gap-1">
                  <template v-if="result.warning.includes('太慢')">
                    <PhWarning :size="12" weight="fill" class="text-amber-500" />
                    <span class="text-amber-600 dark:text-amber-400">{{ translateSpeedWarning(result.warning) }}</span>
                  </template>
                  <template v-else-if="result.warning.includes('极快')">
                    <PhStar :size="12" weight="fill" class="text-amber-500" />
                    <span class="text-amber-600 dark:text-amber-400">{{ translateSpeedWarning(result.warning) }}</span>
                  </template>
                  <template v-else-if="result.warning.includes('很快')">
                    <PhThumbsUp :size="12" weight="fill" class="text-blue-500" />
                    <span class="text-blue-600 dark:text-blue-400">{{ translateSpeedWarning(result.warning) }}</span>
                  </template>
                  <template v-else>
                    <PhCheckCircle :size="12" weight="fill" class="text-emerald-500" />
                    <span class="text-emerald-600 dark:text-emerald-400">{{
                      translateSpeedWarning(result.warning)
                    }}</span>
                  </template>
                </span>
                <!-- 其他测试：正常显示 -->
                <span v-else-if="result.success" class="text-emerald-600 dark:text-emerald-400 flex items-center gap-1">
                  <PhCheckCircle :size="12" weight="fill" />
                  {{ result.latency }}ms
                </span>
                <span v-else-if="result.warning" class="text-amber-500 flex items-center gap-1">
                  <PhWarning :size="12" weight="fill" />
                  {{ result.warning }}
                </span>
                <span v-else class="text-red-500 flex items-center gap-1">
                  <PhXCircle :size="12" weight="fill" />
                  {{ t('settings.testFailed') }}
                </span>
              </div>
            </div>
          </div>
        </section>
      </div>

      <!-- Credits Settings -->
      <div v-if="activeTab === 'credits'" class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
        <div class="text-sm text-slate-500 dark:text-slate-400 mb-4">
          {{ t('settings.creditsDesc') }}
        </div>

        <div
          v-for="category in creditsData.categories"
          :key="category.name"
          class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 shadow-sm overflow-hidden"
        >
          <div class="px-4 py-3 bg-slate-50 dark:bg-slate-700/50 border-b border-slate-200 dark:border-slate-600">
            <h3 class="font-semibold text-slate-700 dark:text-slate-200">{{ category.name }}</h3>
          </div>
          <div class="divide-y divide-slate-100 dark:divide-slate-700">
            <div
              v-for="item in category.items"
              :key="item.name"
              class="px-4 py-3 flex items-center justify-between hover:bg-slate-50 dark:hover:bg-slate-700/30 transition-colors"
            >
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="font-medium text-slate-800 dark:text-slate-200">{{ item.name }}</span>
                  <span
                    v-if="item.version"
                    class="text-xs px-1.5 py-0.5 bg-slate-100 dark:bg-slate-600 text-slate-500 dark:text-slate-400 rounded"
                  >
                    v{{ item.version }}
                  </span>
                </div>
                <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5 truncate">
                  {{ t('settings.creditsItems.' + item.key) }}
                </p>
              </div>
              <a
                :href="item.url"
                target="_blank"
                rel="noopener noreferrer"
                class="ml-3 text-blue-500 hover:text-blue-600 dark:text-blue-400 dark:hover:text-blue-300 text-sm flex items-center gap-1 shrink-0"
              >
                <PhGlobe :size="14" />
                {{ t('settings.creditsLink') }}
              </a>
            </div>
          </div>
        </div>
      </div>

      <!-- About Settings -->
      <div v-if="activeTab === 'about'" class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
        <div
          class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-8 shadow-sm flex flex-col items-center text-center space-y-4"
        >
          <div class="w-20 h-20 bg-slate-100 dark:bg-slate-700 rounded-2xl flex items-center justify-center mb-2">
            <img :src="globalConfig.appIcon" alt="Logo" class="w-12 h-12 opacity-80" />
          </div>
          <div>
            <h2 class="text-xl font-bold text-slate-800 dark:text-slate-200">
              {{ locale === 'zh-CN' ? globalConfig.appName : globalConfig.appNameEn }}
            </h2>
            <p class="text-slate-500 dark:text-slate-400 text-sm mt-1">
              {{ t('settings.version') }} {{ globalConfig.appVersion }}
            </p>
          </div>
          <button
            :disabled="checkingUpdate"
            class="mt-2 px-4 py-2 bg-blue-50 dark:bg-blue-900/30 text-blue-600 rounded-lg text-sm font-medium hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
            @click="handleCheckUpdate"
          >
            <PhArrowsClockwise :class="{ 'animate-spin': checkingUpdate }" :size="16" weight="bold" />
            {{ checkingUpdate ? t('settings.checking') : t('settings.checkUpdate') }}
          </button>
          <p class="text-slate-600 dark:text-slate-400 max-w-md text-sm leading-relaxed mt-4">
            {{ locale === 'zh-CN' ? globalConfig.appDescription : globalConfig.appDescriptionEn }}
          </p>

          <div
            class="max-w-md w-full mt-6 p-4 rounded-xl bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800/50 flex flex-col items-center gap-2"
          >
            <div class="flex items-center gap-2 text-amber-800 dark:text-amber-400 font-bold text-sm">
              <PhInfo :size="18" weight="fill" />
              {{ t('settings.importantNotice') }}
            </div>
            <p class="text-xs text-amber-700 dark:text-amber-500/90 leading-relaxed text-center">
              {{ t('settings.freeSoftwareNotice') }}
            </p>
          </div>

          <div class="flex gap-4 mt-6 pt-6 border-t border-slate-100 dark:border-slate-700 w-full justify-center">
            <button
              class="flex items-center gap-1.5 text-sm font-medium text-slate-600 dark:text-slate-400 hover:text-blue-500 dark:hover:text-blue-400 transition-colors bg-slate-50 dark:bg-slate-700/50 px-3 py-1.5 rounded-lg border border-slate-100 dark:border-slate-700/50 cursor-pointer"
              @click="openLink(globalConfig.git.github)"
            >
              <PhGithubLogo :size="18" weight="duotone" />
              GitHub
            </button>
            <button
              class="flex items-center gap-1.5 text-sm font-medium text-slate-600 dark:text-slate-400 hover:text-red-500 dark:hover:text-red-400 transition-colors bg-slate-50 dark:bg-slate-700/50 px-3 py-1.5 rounded-lg border border-slate-100 dark:border-slate-700/50 cursor-pointer"
              @click="openLink(globalConfig.git.gitee)"
            >
              <PhGlobe :size="18" weight="duotone" />
              Gitee
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Log Dialog -->
    <div
      v-if="logDialogVisible"
      class="fixed inset-0 z-50 flex items-center justify-center animate-in fade-in duration-200"
    >
      <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-sm" @click="logDialogVisible = false"></div>
      <div
        class="relative w-full max-w-2xl bg-white dark:bg-slate-800 rounded-xl shadow-2xl overflow-hidden flex flex-col border border-slate-200 dark:border-slate-700 mx-4 max-h-[80vh] animate-in zoom-in-95 duration-200"
      >
        <div
          class="flex items-center justify-between px-4 py-3 border-b border-slate-100 dark:border-slate-700 bg-slate-50 dark:bg-slate-800/50"
        >
          <h3 class="font-medium text-slate-800 dark:text-slate-200 flex items-center gap-2">
            <PhTerminalWindow :size="18" class="text-slate-500" />
            {{ logDialogTitle }}
          </h3>
          <button
            class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 p-1 rounded-md hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors"
            @click="logDialogVisible = false"
          >
            <PhX :size="16" weight="bold" />
          </button>
        </div>
        <div
          ref="logContainer"
          class="flex-1 overflow-y-auto p-4 bg-[#1e1e1e] custom-scrollbar font-mono text-sm leading-relaxed"
        >
          <div v-for="(log, idx) in currentLogs" :key="idx" class="text-green-400 mb-1 break-all">
            <span class="text-slate-500 mr-2">></span>{{ log }}
          </div>
          <div v-if="currentLogs.length === 0" class="text-slate-500 italic">暂无日志</div>
        </div>
      </div>
    </div>
  </div>

  <!-- 公网服务二次确认弹窗 -->
  <Teleport to="body">
    <div v-if="showPublicConfirm" class="fixed inset-0 z-[999] flex items-center justify-center p-4">
      <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" @click="cancelPublicMode"></div>
      <div
        class="relative bg-white dark:bg-slate-800 rounded-2xl shadow-2xl max-w-md w-full p-6 animate-in zoom-in-95 duration-200 border border-red-200 dark:border-red-900/50"
      >
        <!-- 标题 -->
        <div class="flex items-start gap-3 mb-4">
          <div
            class="w-10 h-10 rounded-xl bg-red-100 dark:bg-red-900/40 flex items-center justify-center text-red-600 dark:text-red-400 shrink-0"
          >
            <PhWarning :size="22" weight="duotone" />
          </div>
          <div>
            <h3 class="font-bold text-slate-900 dark:text-slate-100 text-base">
              {{ t('settings.publicModeWarningTitle') }}
            </h3>
            <p class="text-sm text-slate-500 dark:text-slate-400 mt-1">
              {{ t('settings.publicModeWarningDesc') }}
            </p>
          </div>
        </div>
        <!-- 风险列表 -->
        <ul class="space-y-2 mb-4">
          <li
            v-for="riskKey in ['publicModeRisk1', 'publicModeRisk2', 'publicModeRisk3', 'publicModeRisk4']"
            :key="riskKey"
            class="flex items-start gap-2 text-xs text-slate-600 dark:text-slate-400"
          >
            <span class="text-red-500 mt-0.5 shrink-0">•</span>
            <span>{{ t(`settings.${riskKey}`) }}</span>
          </li>
        </ul>
        <!-- 建议 -->
        <div
          class="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-xl p-3 mb-5 text-xs text-amber-700 dark:text-amber-400"
        >
          {{ t('settings.publicModeAdvice') }}
        </div>
        <!-- 三个按钮 -->
        <div class="flex items-center gap-2">
          <button
            class="flex-1 py-2.5 rounded-xl border border-slate-200 dark:border-slate-600 text-slate-600 dark:text-slate-400 text-sm font-medium hover:bg-slate-50 dark:hover:bg-slate-700 transition-all active:scale-95"
            @click="cancelPublicMode"
          >
            {{ t('settings.publicModeCancel') }}
          </button>
          <button
            class="flex-1 py-2.5 rounded-xl border border-slate-300 dark:border-slate-600 text-slate-500 dark:text-slate-400 text-sm font-medium hover:bg-slate-50 dark:hover:bg-slate-700 transition-all active:scale-95"
            @click="neverAskPublicMode"
          >
            {{ t('settings.publicModeNeverAsk') }}
          </button>
          <button
            class="flex-1 py-2.5 rounded-xl bg-red-600 hover:bg-red-700 text-white text-sm font-bold transition-all active:scale-95 shadow-md shadow-red-500/20"
            @click="confirmPublicMode"
          >
            {{ t('settings.publicModeConfirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <!-- 代理设置弹窗 -->
  <Teleport to="body">
    <div v-if="showProxyDialog" class="fixed inset-0 z-[999] flex items-center justify-center p-4">
      <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" @click="closeProxyDialog"></div>
      <div
        class="relative bg-white dark:bg-slate-800 rounded-2xl shadow-2xl max-w-md w-full p-6 animate-in zoom-in-95 duration-200 border border-slate-200 dark:border-slate-700"
      >
        <!-- 标题 -->
        <div class="flex items-center justify-between mb-5">
          <h3 class="font-bold text-slate-900 dark:text-slate-100 text-base flex items-center gap-2">
            <PhWifiHigh :size="20" class="text-sky-500" weight="duotone" />
            {{ t('settings.proxyDialogTitle') }}
          </h3>
          <button
            class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 p-1 rounded-md hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors"
            @click="closeProxyDialog"
          >
            <PhX :size="16" weight="bold" />
          </button>
        </div>

        <!-- 模式选择器（仿 launchMode Tag 选择器） -->
        <div class="mb-5">
          <div class="text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
            {{ t('settings.proxyMode') }}
          </div>
          <div class="flex items-center gap-1 bg-slate-100 dark:bg-slate-700 rounded-lg p-1">
            <button
              :class="[
                'flex-1 px-3 py-2 rounded-md text-xs font-medium transition-all text-center',
                proxyDialogMode === 'none'
                  ? 'bg-white dark:bg-slate-600 text-slate-800 dark:text-slate-100 shadow-sm'
                  : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200',
              ]"
              @click="proxyDialogMode = 'none'"
            >
              {{ t('settings.proxyModeNone') }}
            </button>
            <button
              :class="[
                'flex-1 px-3 py-2 rounded-md text-xs font-medium transition-all text-center',
                proxyDialogMode === 'system'
                  ? 'bg-white dark:bg-slate-600 text-sky-600 dark:text-sky-400 shadow-sm'
                  : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200',
              ]"
              @click="proxyDialogMode = 'system'"
            >
              {{ t('settings.proxyModeSystem') }}
            </button>
            <button
              :class="[
                'flex-1 px-3 py-2 rounded-md text-xs font-medium transition-all text-center',
                proxyDialogMode === 'custom'
                  ? 'bg-white dark:bg-slate-600 text-sky-600 dark:text-sky-400 shadow-sm'
                  : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200',
              ]"
              @click="proxyDialogMode = 'custom'"
            >
              {{ t('settings.proxyModeCustom') }}
            </button>
          </div>
          <!-- 当前模式描述 -->
          <div class="mt-2 text-xs pl-1">
            <span v-if="proxyDialogMode === 'none'" class="text-slate-400 dark:text-slate-500">{{
              t('settings.proxyModeNoneDesc')
            }}</span>
            <span v-else-if="proxyDialogMode === 'system'">
              <template v-if="systemProxyInfo">
                <span :class="systemProxyInfo.enabled ? 'text-sky-500' : 'text-amber-500'">
                  {{ systemProxyInfo.server }}
                  <span
                    class="ml-1 px-1 py-0.5 rounded text-[10px] font-medium"
                    :class="
                      systemProxyInfo.enabled
                        ? 'bg-sky-100 dark:bg-sky-900/40 text-sky-600'
                        : 'bg-amber-100 dark:bg-amber-900/40 text-amber-600'
                    "
                  >
                    {{ systemProxyInfo.enabled ? t('settings.systemProxyEnabled') : t('settings.systemProxyDisabled') }}
                  </span>
                </span>
              </template>
              <span v-else class="text-slate-400 dark:text-slate-500">{{ t('settings.proxyModeSystemDesc') }}</span>
            </span>
            <span v-else class="text-sky-500">{{ t('settings.proxyModeCustomDesc') }}</span>
          </div>
        </div>

        <!-- 地址/端口输入框 -->
        <div class="space-y-3 mb-5">
          <div class="flex gap-3">
            <!-- 代理地址 -->
            <div class="flex-1">
              <label class="block text-xs font-medium text-slate-600 dark:text-slate-400 mb-1">{{
                t('settings.proxyHost')
              }}</label>
              <input
                v-if="proxyDialogMode === 'custom'"
                v-model="proxyDialogHost"
                :placeholder="t('settings.proxyHostPlaceholder')"
                class="w-full px-3 py-2 text-sm rounded-lg border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-700 text-slate-800 dark:text-slate-200 placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-sky-400 transition-all"
              />
              <input
                v-else
                readonly
                :value="proxyDialogMode === 'system' ? systemProxyDisplayHost : ''"
                :placeholder="proxyDialogMode === 'system' ? t('settings.proxyHostPlaceholder') : ''"
                class="w-full px-3 py-2 text-sm rounded-lg border border-slate-200 dark:border-slate-600 bg-slate-50 dark:bg-slate-800 text-slate-400 dark:text-slate-500 placeholder-slate-300 cursor-not-allowed transition-all"
              />
            </div>
            <!-- 端口 -->
            <div class="w-28">
              <label class="block text-xs font-medium text-slate-600 dark:text-slate-400 mb-1">{{
                t('settings.proxyPort')
              }}</label>
              <input
                v-if="proxyDialogMode === 'custom'"
                v-model.number="proxyDialogPort"
                type="number"
                min="1"
                max="65535"
                :placeholder="t('settings.proxyPortPlaceholder')"
                class="w-full px-3 py-2 text-sm rounded-lg border border-slate-200 dark:border-slate-600 bg-white dark:bg-slate-700 text-slate-800 dark:text-slate-200 placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-sky-400 transition-all"
              />
              <input
                v-else
                readonly
                :value="proxyDialogMode === 'system' ? systemProxyDisplayPort : ''"
                :placeholder="proxyDialogMode === 'system' ? t('settings.proxyPortPlaceholder') : ''"
                class="w-full px-3 py-2 text-sm rounded-lg border border-slate-200 dark:border-slate-600 bg-slate-50 dark:bg-slate-800 text-slate-400 dark:text-slate-500 placeholder-slate-300 cursor-not-allowed transition-all"
              />
            </div>
          </div>

          <!-- 测试连接 -->
          <div class="flex items-center gap-3">
            <button
              :disabled="
                proxyDialogMode === 'none' ||
                proxyTestStatus === 'testing' ||
                (proxyDialogMode === 'custom' && (!proxyDialogHost || !proxyDialogPort))
              "
              class="px-4 py-2 text-xs font-medium rounded-lg transition-all flex items-center gap-1.5 disabled:opacity-40 disabled:cursor-not-allowed"
              :class="
                proxyTestStatus === 'testing'
                  ? 'bg-slate-100 dark:bg-slate-700 text-slate-500'
                  : 'bg-sky-50 dark:bg-sky-900/30 text-sky-600 hover:bg-sky-100 dark:hover:bg-sky-900/50'
              "
              @click="testProxy"
            >
              <PhArrowsClockwise :size="13" :class="{ 'animate-spin': proxyTestStatus === 'testing' }" />
              {{ proxyTestStatus === 'testing' ? t('settings.proxyTesting') : t('settings.proxyTest') }}
            </button>
            <!-- 测试结果 -->
            <div
              v-if="proxyTestStatus === 'success'"
              class="flex items-center gap-1 text-xs text-emerald-600 dark:text-emerald-400"
            >
              <PhCheckCircle :size="14" weight="fill" />
              {{ t('settings.proxyTestSuccess') }} · {{ t('settings.proxyTestLatency') }} {{ proxyTestLatency }}ms
            </div>
            <div v-else-if="proxyTestStatus === 'failed'" class="flex items-center gap-1 text-xs text-red-500">
              <PhXCircle :size="14" weight="fill" />
              {{ t('settings.proxyTestFailed') }}
            </div>
          </div>
          <!-- 失败详情 -->
          <div v-if="proxyTestStatus === 'failed' && proxyTestError" class="text-[11px] text-red-400 break-all pl-1">
            {{ proxyTestError }}
          </div>
        </div>

        <!-- 底部按钮 -->
        <div class="flex items-center justify-end gap-2 pt-4 border-t border-slate-100 dark:border-slate-700">
          <button
            class="px-4 py-2 rounded-xl border border-slate-200 dark:border-slate-600 text-slate-600 dark:text-slate-400 text-sm font-medium hover:bg-slate-50 dark:hover:bg-slate-700 transition-all active:scale-95"
            @click="closeProxyDialog"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            class="px-5 py-2 rounded-xl bg-sky-500 hover:bg-sky-600 text-white text-sm font-bold transition-all active:scale-95 shadow-sm shadow-sky-500/20"
            @click="saveProxyDialog"
          >
            {{ t('settings.proxySave') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: #cbd5e1;
  border-radius: 20px;
}
</style>

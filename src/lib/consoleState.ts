import { reactive, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'vue-sonner'
import i18n from '../lang'
import { initReleases } from './useReleases'
import { useRouter } from 'vue-router'

// 路由实例（由外部设置，避免循环依赖）
let _router: ReturnType<typeof useRouter> | null = null
export function setConsoleRouter(router: ReturnType<typeof useRouter>) {
  _router = router
}

/**
 * 根据酒馆版本计算所需的最低 Node.js 主版本号
 * - ST >= 1.17.0 需要 Node v20+
 * - ST < 1.17.0 需要 Node v18+
 */
function getRequiredNodeMajor(tavernVersion: string | null): number {
  if (!tavernVersion) return 18
  // 解析版本号，支持 "1.17.0" 或 "v1.17.0" 格式
  const match = tavernVersion.replace(/^v/, '').match(/^(\d+)\.(\d+)\.(\d+)/)
  if (!match) return 18
  const major = parseInt(match[1], 10)
  const minor = parseInt(match[2], 10)
  // >= 1.17.0
  if (major > 1 || (major === 1 && minor >= 17)) return 20
  return 18
}

/**
 * 检查当前 Node.js 版本是否满足酒馆版本要求
 * 返回 { ok, requiredMajor, currentMajor }
 */
function checkNodeCompatibility(): { ok: boolean; requiredMajor: number; currentMajor: number | null } {
  try {
    // 读取 Node 信息
    const nodeCache = localStorage.getItem('app_settings_node_cache')
    const nodeInfo = nodeCache ? JSON.parse(nodeCache) : null
    const nodeVersion: string | null = nodeInfo?.version || null
    const nodeSource: string = nodeInfo?.source || 'none'

    // 读取酒馆版本
    const configCache = localStorage.getItem('app_settings_config_cache')
    let tavernVersion: string | null = null
    if (configCache) {
      const config = JSON.parse(configCache)
      tavernVersion = config?.sillytavern?.version?.version || null
    }

    const requiredMajor = getRequiredNodeMajor(tavernVersion)

    if (!nodeVersion || nodeSource === 'none') {
      return { ok: false, requiredMajor, currentMajor: null }
    }

    const match = nodeVersion.match(/v?(\d+)\./)
    const currentMajor = match ? parseInt(match[1], 10) : null

    if (currentMajor === null) return { ok: false, requiredMajor, currentMajor: null }

    // 系统 Node 不满足要求时，视为不可用（将使用内置 Node）
    // 如果是内置 Node（local），也要检查版本
    return { ok: currentMajor >= requiredMajor, requiredMajor, currentMajor }
  } catch (e) {
    console.error('检查 Node 版本兼容性失败:', e)
    return { ok: true, requiredMajor: 18, currentMajor: null } // 出错时不阻止启动
  }
}

export type LogType = 'info' | 'success' | 'error' | 'output' | 'system'

export interface LogEntry {
  id: number
  type: LogType
  text: string
  time: number
}

// 0: 未启动, 1: 启动中, 2: 运行中, 3: 已停止, 4: 启动失败/异常
export const consoleStatus = ref(0)
export const consoleLogs = reactive<LogEntry[]>([])
export const serverUrl = ref('')
export const processPid = ref<number | null>(null)
// 网络服务模式信息（lan/public 模式启动成功后填充）
export const networkMode = ref<'lan' | 'public' | null>(null)
export const networkPort = ref<number>(8000)
// 公网服务 session 计数：每次 public 模式的酒馆启动成功时 +1，用于检测结果缓存
export const publicSessionId = ref<number>(0)
// 当前启动模式（从 config 读取，启动时更新，停止时重置）
export const launchMode = ref<string>('normal')

let nextId = 0
let isIntentionalStop = false

export function addLog(type: LogType, text: string) {
  // 移除可能存在的 ANSI 转义字符
  const cleanText = text.replace(/\x1B\[[0-9;]*[a-zA-Z]/g, '')
  consoleLogs.push({
    id: nextId++,
    type,
    text: cleanText,
    time: Date.now()
  })

  // 限制日志条数，防止内存泄漏（最多保留2000条日志）
  if (consoleLogs.length > 2000) {
    consoleLogs.splice(0, consoleLogs.length - 2000)
  }
}

export function clearLogs() {
  consoleLogs.splice(0, consoleLogs.length)
}

export async function startProcess() {
  if (consoleStatus.value === 1 || consoleStatus.value === 2) return

  // 检查 Node.js 版本是否满足当前酒馆版本要求
  const { ok, requiredMajor, currentMajor } = checkNodeCompatibility()
  if (!ok) {
    const msg = i18n.global.t('console.nodeVersionInsufficient', {
      required: `v${requiredMajor}`,
      current: currentMajor !== null ? `v${currentMajor}` : i18n.global.t('home.notInstalled'),
    })
    toast.warning(msg, { duration: 5000 })
    // 跳转到设置页触发内置 Node 安装
    if (_router) {
      setTimeout(() => {
        _router!.push('/settings?action=install_node')
      }, 800)
    }
    return
  }

  // 从 config 缓存读取当前启动模式
  const configCache = localStorage.getItem('app_settings_config_cache')
  if (configCache) {
    try {
      const cfg = JSON.parse(configCache)
      launchMode.value = cfg?.launchMode || 'normal'
    } catch {
      launchMode.value = 'normal'
    }
  } else {
    launchMode.value = 'normal'
  }

  clearLogs()
  consoleStatus.value = 1
  serverUrl.value = ''
  processPid.value = null
  isIntentionalStop = false
  networkMode.value = null
  networkPort.value = 8000
  addLog('system', i18n.global.t('console.initializing'))

  try {
    await invoke('start_sillytavern')
    addLog('success', i18n.global.t('console.startSent'))
  } catch (error: any) {
    consoleStatus.value = 4
    addLog('error', i18n.global.t('console.startError', { error }))
    toast.error(i18n.global.t('console.startError', { error }))
  }
}

export async function stopProcess() {
  if (consoleStatus.value === 0 || consoleStatus.value === 3 || consoleStatus.value === 4) return

  try {
    isIntentionalStop = true
    await invoke('stop_sillytavern')
    addLog('system', i18n.global.t('console.stopping'))
    consoleStatus.value = 3
    processPid.value = null
  } catch (error: any) {
    toast.error(i18n.global.t('console.stopError', { error }))
  }
}

// 预加载 GitHub 加速列表
const preloadGithubProxies = async () => {
  try {
    const THREE_DAYS_MS = 3 * 24 * 60 * 60 * 1000;
    const lastFetchTime = localStorage.getItem('app_settings_proxies_last_fetch');
    const now = Date.now();

    // 如果没有缓存或者缓存已过期，则预加载
    if (!lastFetchTime || (now - Number(lastFetchTime) >= THREE_DAYS_MS)) {
      await invoke('fetch_github_proxies');
      localStorage.setItem('app_settings_proxies_last_fetch', now.toString());
    }
  } catch (error) {
    // 静默失败，不影响启动
    console.error('预加载 GitHub 加速列表失败:', error);
  }
};

let isInitialized = false
export async function initConsoleState() {
  if (isInitialized) return
  isInitialized = true

  setTimeout(() => {
    initReleases();
    preloadGithubProxies();
  }, 2000);

  await listen<string>('process-log', (event) => {
    const text = event.payload
    let type: LogType = 'output'
    if (text.startsWith('ERROR:')) type = 'error'
    else if (text.startsWith('INFO:')) type = 'info'

    addLog(type, text.replace(/^(INFO|ERROR):\s*/, ''))

    // 如果看到成功启动的标志
    if (text.toLowerCase().includes('sillytavern is listening on') || text.includes('SillyTavern is running')) {
      consoleStatus.value = 2
    }

    // 提取访问链接
    const urlMatch = text.match(/http:\/\/(?:localhost|127\.0\.0\.1|0\.0\.0\.0):\d+/)
    if (urlMatch && !serverUrl.value) {
      serverUrl.value = urlMatch[0]
    }

    const pidMatch = text.match(/PID:\s*(\d+)/i)
    if (pidMatch) {
      processPid.value = parseInt(pidMatch[1], 10)
    }
  })

  // 桌面程序模式关闭子窗口时提前标记为主动停止，避免后续 process-exit 被误判为异常退出
  await listen('process-intentional-stop', () => {
    isIntentionalStop = true
  })

  // 局域网/公网服务模式：酒馆启动成功后，接收端口信息
  await listen<{ mode: string; port: number }>('tavern-network-ready', (event) => {
    const { mode, port } = event.payload
    networkMode.value = mode as 'lan' | 'public'
    networkPort.value = port
    if (mode === 'public') {
      publicSessionId.value += 1
    }
  })

  await listen('process-exit', () => {
    if (!isIntentionalStop && (consoleStatus.value === 1 || consoleStatus.value === 2)) {
      consoleStatus.value = 4
      addLog('error', i18n.global.t('console.processAbnormalExit'))
    } else {
      consoleStatus.value = 3
      addLog('system', i18n.global.t('console.processExited'))
    }
    processPid.value = null
    networkMode.value = null
    networkPort.value = 8000
    launchMode.value = 'normal'
  })
}

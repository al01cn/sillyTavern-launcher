import { reactive, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'vue-sonner'
import i18n from '../lang'
import { initReleases } from './useReleases'

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

  clearLogs()
  consoleStatus.value = 1
  serverUrl.value = ''
  processPid.value = null
  isIntentionalStop = false
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

let isInitialized = false
export async function initConsoleState() {
  if (isInitialized) return
  isInitialized = true

  setTimeout(() => {
    initReleases();
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

  await listen('process-exit', () => {
    if (!isIntentionalStop && (consoleStatus.value === 1 || consoleStatus.value === 2)) {
      consoleStatus.value = 4
      addLog('error', i18n.global.t('console.processAbnormalExit'))
    } else {
      consoleStatus.value = 3
      addLog('system', i18n.global.t('console.processExited'))
    }
    processPid.value = null
  })
}

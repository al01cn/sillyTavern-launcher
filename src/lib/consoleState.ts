import { reactive, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'vue-sonner'

export type LogType = 'info' | 'success' | 'error' | 'output' | 'system'

export interface LogEntry {
  id: number
  type: LogType
  text: string
  time: number
}

// 0: 未启动, 1: 启动中, 2: 运行中, 3: 启动失败/已停止
export const consoleStatus = ref(0)
export const consoleLogs = reactive<LogEntry[]>([])

let nextId = 0

export function addLog(type: LogType, text: string) {
  // 移除可能存在的 ANSI 转义字符
  const cleanText = text.replace(/\x1B\[[0-9;]*[a-zA-Z]/g, '')
  consoleLogs.push({
    id: nextId++,
    type,
    text: cleanText,
    time: Date.now()
  })
}

export function clearLogs() {
  consoleLogs.splice(0, consoleLogs.length)
}

export async function startProcess() {
  if (consoleStatus.value === 1 || consoleStatus.value === 2) return
  
  clearLogs()
  consoleStatus.value = 1
  addLog('system', '正在初始化酒馆环境...')
  
  try {
    await invoke('start_sillytavern')
    addLog('success', '启动命令已发送，等待酒馆服务就绪...')
  } catch (error: any) {
    consoleStatus.value = 3
    addLog('error', `启动失败: ${error}`)
    toast.error(`启动失败: ${error}`)
  }
}

export async function stopProcess() {
  if (consoleStatus.value === 0 || consoleStatus.value === 3) return
  
  try {
    await invoke('stop_sillytavern')
    addLog('system', '正在停止酒馆服务...')
    consoleStatus.value = 3
  } catch (error: any) {
    toast.error(`停止失败: ${error}`)
  }
}

let isInitialized = false
export async function initConsoleState() {
  if (isInitialized) return
  isInitialized = true

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
  })

  await listen('process-exit', () => {
    consoleStatus.value = 3
    addLog('system', '进程已退出')
  })
}

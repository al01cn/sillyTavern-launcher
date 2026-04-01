/**
 * 全局 Git / Node.js 安装状态管理
 *
 * 将安装进度状态提升到模块级 reactive，使其在页面切换时不丢失。
 * Settings.vue 只负责读取这里的状态并订阅事件，不再维护本地 ref。
 */
import { reactive } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

export interface DownloadProgress {
  status: string
  progress: number
  log: string
}

export interface InstallTaskState {
  installing: boolean
  progress: DownloadProgress
  logs: string[]
}

function createTask(): InstallTaskState {
  return {
    installing: false,
    progress: { status: '', progress: 0, log: '' },
    logs: [],
  }
}

/** Git 安装全局状态（模块级单例，页面切换不销毁） */
export const gitInstallState = reactive<InstallTaskState>(createTask())

/** Node.js 安装全局状态（模块级单例，页面切换不销毁） */
export const nodeInstallState = reactive<InstallTaskState>(createTask())

/** 重置 Git 安装状态 */
export function resetGitInstall() {
  gitInstallState.installing = false
  gitInstallState.progress = { status: '', progress: 0, log: '' }
  gitInstallState.logs = []
}

/** 重置 Node 安装状态 */
export function resetNodeInstall() {
  nodeInstallState.installing = false
  nodeInstallState.progress = { status: '', progress: 0, log: '' }
  nodeInstallState.logs = []
}

/** 取消当前 Git 安装/下载（通知 Rust 端置位取消标志） */
export async function cancelGitInstall() {
  try {
    await invoke('cancel_git_node_install')
  } catch (e) {
    console.error('cancelGitInstall error:', e)
  }
}

/** 取消当前 Node.js 安装/下载（通知 Rust 端置位取消标志） */
export async function cancelNodeInstall() {
  try {
    await invoke('cancel_git_node_install')
  } catch (e) {
    console.error('cancelNodeInstall error:', e)
  }
}

let _listenerInited = false

/**
 * 全局注册 `download-progress` 事件监听，只需调用一次（在 App.vue 的 onMounted 中）。
 * 此后无论在哪个页面，进度都会写入全局状态，页面切换不丢失。
 */
export async function initDownloadProgressListener() {
  if (_listenerInited) return
  _listenerInited = true

  await listen<DownloadProgress>('download-progress', event => {
    const p = event.payload
    if (nodeInstallState.installing) {
      nodeInstallState.progress = p
      nodeInstallState.logs.push(p.log)
    } else if (gitInstallState.installing) {
      gitInstallState.progress = p
      gitInstallState.logs.push(p.log)
    }
  })
}

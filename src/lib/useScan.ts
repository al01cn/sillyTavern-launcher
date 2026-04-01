import { reactive } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

export interface ScanProgress {
  key: string
  count: number
  found: number
  is_done: boolean
  current_path?: string
}

export interface LocalTavernItem {
  path: string
  version: string
  hasNodeModules: boolean
}

class ScanManager {
  public state = reactive({
    isScanning: false,
    progress: { key: '', count: 0, found: 0, is_done: false } as ScanProgress,
    localList: (() => {
      try {
        const specificCache = localStorage.getItem('app_scan_local_list_cache')
        if (specificCache) return JSON.parse(specificCache)

        const globalCache = localStorage.getItem('app_settings_config_cache')
        if (globalCache) {
          const config = JSON.parse(globalCache)
          if (config.localSillytavernList && Array.isArray(config.localSillytavernList)) {
            return config.localSillytavernList
          }
        }
      } catch (e) {
        console.error('Failed to load local list cache:', e)
      }
      return []
    })() as LocalTavernItem[],
    scanTimeText: '00:00',
    scanLogPaths: [] as string[],
  })

  private isInitialized = false
  private hasScannedOnce = localStorage.getItem('app_has_scanned_once') === 'true'

  async init() {
    if (this.isInitialized) return

    // Listen to progress
    await listen<ScanProgress>('scan-local-sillytavern-progress', event => {
      this.state.progress = event.payload
      if (event.payload.is_done) {
        this.state.isScanning = false
        // 标记首次扫描已完成（无论是否找到实例）
        this.markScannedOnce()
      } else {
        this.state.isScanning = true
        // 实时记录扫描路径（最多保留 500 条）
        if (event.payload.current_path) {
          this.state.scanLogPaths.push(event.payload.current_path)
          if (this.state.scanLogPaths.length > 500) {
            this.state.scanLogPaths.splice(0, this.state.scanLogPaths.length - 500)
          }
        }
      }
    })

    // Listen to timer update
    await listen<string>('scan-local-sillytavern-timer', event => {
      this.state.scanTimeText = event.payload
    })

    // Listen to found items
    await listen<LocalTavernItem>('scan-local-sillytavern-found', async event => {
      const item = event.payload
      const existingIndex = this.state.localList.findIndex(existing => existing.path === item.path)
      if (existingIndex === -1) {
        this.state.localList.push(item)
        await this.saveLocalList()
      } else {
        let changed = false
        if (this.state.localList[existingIndex].hasNodeModules !== item.hasNodeModules) {
          this.state.localList[existingIndex].hasNodeModules = item.hasNodeModules
          changed = true
        }
        if (this.state.localList[existingIndex].version !== item.version) {
          this.state.localList[existingIndex].version = item.version
          changed = true
        }
        if (changed) {
          await this.saveLocalList()
        }
      }
    })

    this.isInitialized = true
  }

  async loadConfig() {
    try {
      const config: any = await invoke('get_app_config')
      if (config.localSillytavernList && Array.isArray(config.localSillytavernList)) {
        this.state.localList = config.localSillytavernList

        // Double check dependencies status
        let changed = false
        for (let i = 0; i < this.state.localList.length; i++) {
          const item = this.state.localList[i]
          try {
            const hasDeps = await invoke<boolean>('check_local_tavern_dependencies', {
              path: item.path,
            })
            if (item.hasNodeModules !== hasDeps) {
              item.hasNodeModules = hasDeps
              changed = true
            }
          } catch (_e) {
            // ignore
          }
        }

        if (changed) {
          await this.saveLocalList()
        }

        localStorage.setItem('app_scan_local_list_cache', JSON.stringify(this.state.localList))
      }
      // 恢复 hasScannedOnce 状态
      if (config.hasScannedOnce) {
        this.hasScannedOnce = true
      }
    } catch (e) {
      console.error('Failed to load scan config:', e)
    }
  }

  async saveLocalList() {
    try {
      localStorage.setItem('app_scan_local_list_cache', JSON.stringify(this.state.localList))
      const config: any = await invoke('get_app_config')
      config.localSillytavernList = this.state.localList
      await invoke('save_app_config', { config })
    } catch (e) {
      console.error('Failed to save local list', e)
    }
  }

  async cancelScan() {
    try {
      await invoke('cancel_scan_local_sillytavern')
    } catch (e) {
      console.error('Failed to cancel scan:', e)
    }
  }

  async startScan(manual = false) {
    if (this.state.isScanning) return
    // 非手动扫描时：如果已经扫描过一次，或者已有本地列表，则不再自动触发
    if (!manual) {
      if (this.hasScannedOnce) return
      if (this.state.localList.length > 0) return
    }

    this.state.isScanning = true
    this.state.progress = { key: 'versions.scanPreparing', count: 0, found: 0, is_done: false }
    this.state.scanTimeText = '00:00'
    this.state.scanLogPaths = []

    try {
      await invoke('scan_local_sillytavern')
    } catch (e) {
      this.state.isScanning = false
      // 扫描出错也标记为已扫描，避免反复重试
      this.markScannedOnce()
      throw e
    }
  }

  private async markScannedOnce() {
    if (this.hasScannedOnce) return
    this.hasScannedOnce = true
    localStorage.setItem('app_has_scanned_once', 'true')
    try {
      const config: any = await invoke('get_app_config')
      config.hasScannedOnce = true
      await invoke('save_app_config', { config })
    } catch (e) {
      console.error('Failed to save hasScannedOnce flag:', e)
    }
  }
}

export const scanManager = new ScanManager()

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface ExtensionManifest {
  display_name?: string
  author?: string
  version?: string
  homePage?: string
  auto_update?: boolean
  minimum_client_version?: string
}

export interface ExtensionInfo {
  id: string
  manifest: ExtensionManifest
  dir_path: string
  enabled: boolean
  is_system: boolean
  scope: string
  has_git: boolean
}

// Global state
const extensions = ref<ExtensionInfo[]>([])
const loading = ref(false)
const lastFetchKey = ref<string | null>(null)

export function useExtensions() {
  const fetchExtensions = async (version: { version: string; path: string }, forceUpdate = false) => {
    const cacheKey = version.path || version.version

    // 1. Try to load from cache first if not forced
    if (!forceUpdate && lastFetchKey.value !== cacheKey) {
      const cacheDictStr = localStorage.getItem('extensions_cache_list')
      if (cacheDictStr) {
        try {
          const cacheDict = JSON.parse(cacheDictStr)
          const cached = cacheDict[cacheKey]
          if (Array.isArray(cached)) {
            extensions.value = cached
            lastFetchKey.value = cacheKey
          }
        } catch (e) {
          console.error('Failed to parse extensions cache:', e)
        }
      }
    }

    // 2. Fetch from backend
    loading.value = true
    try {
      const fetched = await invoke<ExtensionInfo[]>('get_extensions', { version })
      const fetchedString = JSON.stringify(fetched)

      // Get current cache dictionary
      let cacheDict: Record<string, any> = {}
      const currentCacheStr = localStorage.getItem('extensions_cache_list')
      if (currentCacheStr) {
        try {
          cacheDict = JSON.parse(currentCacheStr)
        } catch (_e) {}
      }

      const currentCache = cacheDict[cacheKey] ? JSON.stringify(cacheDict[cacheKey]) : null

      // Update if data changed or forced
      if (fetchedString !== currentCache || forceUpdate || lastFetchKey.value !== cacheKey) {
        extensions.value = fetched
        cacheDict[cacheKey] = fetched
        localStorage.setItem('extensions_cache_list', JSON.stringify(cacheDict))
        lastFetchKey.value = cacheKey
      }
    } catch (e) {
      console.error('Failed to fetch extensions:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  const updateExtensionCache = (version: { version: string; path: string }) => {
    const cacheKey = version.path || version.version
    const cacheDictStr = localStorage.getItem('extensions_cache_list')
    let cacheDict: Record<string, any> = {}
    if (cacheDictStr) {
      try {
        cacheDict = JSON.parse(cacheDictStr)
      } catch (_e) {}
    }
    cacheDict[cacheKey] = extensions.value
    localStorage.setItem('extensions_cache_list', JSON.stringify(cacheDict))
  }

  return {
    extensions,
    loading,
    fetchExtensions,
    updateExtensionCache,
  }
}

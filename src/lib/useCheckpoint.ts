import { invoke } from '@tauri-apps/api/core'

export type SetupCheckpoint =
  | 'START'
  | 'GIT_DONE'
  | 'NODE_DONE'
  | 'ST_DOWNLOADED' // This might include version info like ST_DOWNLOADED:v1.16.0
  | 'DONE'

export const updateCheckpoint = async (checkpoint: string | null) => {
  try {
    const config: any = await invoke('get_app_config')
    config.setupCheckpoint = checkpoint
    await invoke('save_app_config', { config })

    // Update local cache too
    const cachedStr = localStorage.getItem('app_settings_config_cache')
    if (cachedStr) {
      try {
        const cached = JSON.parse(cachedStr)
        cached.setupCheckpoint = checkpoint
        localStorage.setItem('app_settings_config_cache', JSON.stringify(cached))
      } catch (_e) {}
    }
  } catch (e) {
    console.error('Failed to update checkpoint:', e)
  }
}

export const clearCheckpoint = async () => {
  await updateCheckpoint(null)
}

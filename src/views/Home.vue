<template>
  <div class="h-full flex flex-col gap-6 text-slate-800 dark:text-slate-200">
    <!-- 顶部 Banner -->
    <div class="w-full h-48 sm:h-56 rounded-2xl overflow-hidden shadow-sm relative group shrink-0">
      <img src="../assets/images/banner.png" alt="Banner" class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105" />
      <div class="absolute inset-0 bg-gradient-to-t from-black/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
    </div>

    <!-- 中部 快捷目录和版本信息 -->
    <div class="flex-1 flex flex-col md:flex-row gap-6">
      
      <!-- 左侧：快捷目录 -->
      <div class="flex-[3] bg-white dark:bg-slate-800 p-6 rounded-2xl shadow-sm border border-slate-100 dark:border-slate-700 flex flex-col">
        <h2 class="text-lg font-bold mb-5 flex items-center text-slate-800 dark:text-slate-200 shrink-0">
          <FolderOpenIcon class="w-5 h-5 mr-2 text-primary" />
          {{ t('home.quickDirectories') }}
        </h2>
        <div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
          <button 
            v-for="btn in dirs" :key="btn.id"
            class="flex flex-col items-center justify-center gap-3 p-4 rounded-xl bg-slate-50 dark:bg-slate-900/50 border border-slate-100 dark:border-slate-700 hover:bg-blue-50 dark:hover:bg-blue-900/30 hover:border-blue-200 dark:hover:border-blue-800 hover:text-primary hover:-translate-y-1 hover:shadow-sm transition-all duration-300 group"
            @click="btn.action"
          >
            <component :is="btn.icon" class="w-8 h-8 text-slate-400 dark:text-slate-500 group-hover:text-primary transition-colors duration-300" />
            <span class="text-sm font-medium text-slate-700 dark:text-slate-300 group-hover:text-primary transition-colors">{{ btn.label }}</span>
          </button>
        </div>
      </div>

      <!-- 右侧：版本信息与一键启动 -->
      <div class="flex-[2] flex flex-col gap-6">
        
        <!-- 版本信息 -->
        <div class="flex-1 bg-white dark:bg-slate-800 p-6 rounded-2xl shadow-sm border border-slate-100 dark:border-slate-700 flex flex-col justify-center">
          <div class="flex items-center justify-between mb-5">
            <h2 class="text-lg font-bold flex items-center text-slate-800 dark:text-slate-200">
              <InfoIcon class="w-5 h-5 mr-2 text-primary" />
              {{ t('home.systemInfo') }}
            </h2>
            <button 
              v-if="status === 2 && serverUrl"
              @click="handleOpenServer"
              class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-emerald-600 bg-emerald-50 dark:bg-emerald-900/30 hover:bg-emerald-100 dark:hover:bg-emerald-900/50 border border-emerald-200 dark:border-emerald-800 transition-colors group"
            >
              <span>{{ t('home.visitTavern') }}</span>
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
            </button>
          </div>
          <div class="flex flex-col gap-4 text-sm">
            <div class="flex items-center justify-between p-3 rounded-xl bg-slate-50 dark:bg-slate-900/50 border border-slate-100 dark:border-slate-700 hover:bg-slate-100 dark:hover:bg-slate-900 transition-colors">
              <span class="text-slate-500 dark:text-slate-400 font-medium flex items-center gap-2">
                <BoxIcon class="w-4 h-4" /> {{ t('home.launcherVersion') }}
              </span>
              <span class="font-bold text-slate-700 dark:text-slate-300">{{ appVersion || t('home.fetching') }}</span>
            </div>
            <div class="flex items-center justify-between p-3 rounded-xl bg-slate-50 dark:bg-slate-900/50 border border-slate-100 dark:border-slate-700 hover:bg-slate-100 dark:hover:bg-slate-900 transition-colors">
              <span class="text-slate-500 dark:text-slate-400 font-medium flex items-center gap-2">
                <TerminalIcon class="w-4 h-4" /> {{ t('home.nodeVersion') }}
              </span>
              <span class="font-bold text-slate-700 dark:text-slate-300">{{ nodeVersion || t('home.notInstalled') }}</span>
            </div>
            <div class="flex items-center justify-between p-3 rounded-xl bg-slate-50 dark:bg-slate-900/50 border border-slate-100 dark:border-slate-700 hover:bg-slate-100 dark:hover:bg-slate-900 transition-colors">
              <span class="text-slate-500 dark:text-slate-400 font-medium flex items-center gap-2">
                <BeerIcon class="w-4 h-4" /> {{ t('home.tavernVersion') }}
              </span>
              <span class="font-bold text-slate-700 dark:text-slate-300">{{ tavernVersion || t('home.notInstalled') }}</span>
            </div>
          </div>
        </div>
        
        <!-- 一键启动按钮 -->
        <button 
          class="btn shrink-0 h-24 rounded-2xl shadow-md hover:shadow-lg transition-all duration-300 border-none text-white flex flex-col items-center justify-center gap-1 group relative overflow-hidden"
          :class="status === 1 || status === 2 ? 'bg-error hover:bg-error/90' : 'bg-primary hover:bg-primary/90'"
          @click="handleToggleProcess"
        >
          <div class="absolute inset-0 bg-white/20 translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-in-out"></div>
          
          <div class="flex items-center gap-2 z-10">
            <StopCircleIcon v-if="status === 1 || status === 2" class="w-7 h-7 fill-current animate-pulse" />
            <PlayIcon v-else class="w-7 h-7 fill-current" />
            <span class="text-2xl font-bold tracking-widest">{{ (status === 1 || status === 2) ? t('home.stopProcess') : t('home.startProcess') }}</span>
          </div>
          <span class="text-xs font-medium opacity-90 z-10">
            {{ (status === 1 || status === 2) ? t('home.stopDesc') : t('home.startDesc') }}
          </span>
        </button>

      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { 
  Play as PlayIcon, 
  StopCircle as StopCircleIcon,
  Folder as FolderIcon, 
  FolderOpen as FolderOpenIcon,
  FileText as FileTextIcon, 
  Beer as BeerIcon, 
  Box as BoxIcon,
  Database as DatabaseIcon,
  Puzzle as PuzzleIcon,
  Info as InfoIcon,
  Terminal as TerminalIcon
} from 'lucide-vue-next'
import { consoleStatus as status, serverUrl, startProcess, stopProcess } from '../lib/consoleState'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Dialog } from '../lib/useDialog'
import { toast } from 'vue-sonner'

const { t } = useI18n()
const router = useRouter()
const appVersion = ref('')
const nodeVersion = ref('')
const tavernVersion = ref('')
const nodePath = ref('')

const openDir = async (dirType: string) => {
  try {
    const customPath = dirType === 'node' && nodePath.value ? nodePath.value : null
    await invoke('open_directory', { dirType, customPath })
  } catch (error) {
    console.error(`Failed to open ${dirType} directory:`, error)
  }
}

const openExtensionFolder = async () => {
    Dialog.warning({
        title: t('home.selectDirectory'),
        msg: t('home.selectExtensionDir'),
        confirmText: t('home.currentUser'),
        thirdBtnText: t('home.global'),
        showCancel: false,
        onConfirm: async () => {
            try {
                // Read current configured version
                let version = tavernVersion.value;
                const cachedConfig = localStorage.getItem('app_settings_config_cache');
                if (cachedConfig) {
                    try {
                        const parsed = JSON.parse(cachedConfig);
                        if (parsed?.sillytavern?.version) {
                            version = parsed.sillytavern.version;
                        }
                    } catch(e) {}
                } else {
                    const config: any = await invoke('get_app_config');
                    if (config?.sillytavern?.version) {
                        version = config.sillytavern.version;
                    }
                }
                await invoke('open_extension_folder', { scope: 'user', version });
            } catch (e) {
                toast.error(t('home.openUserDirFailed') + ': ' + String(e));
            } finally {
                Dialog.close();
            }
        },
        onThirdBtn: async () => {
            try {
                let version = tavernVersion.value;
                const cachedConfig = localStorage.getItem('app_settings_config_cache');
                if (cachedConfig) {
                    try {
                        const parsed = JSON.parse(cachedConfig);
                        if (parsed?.sillytavern?.version) {
                            version = parsed.sillytavern.version;
                        }
                    } catch(e) {}
                } else {
                    const config: any = await invoke('get_app_config');
                    if (config?.sillytavern?.version) {
                        version = config.sillytavern.version;
                    }
                }
                if (!version || version === t('home.notInstalled')) {
                    toast.warning(t('home.noTavernInstalled'));
                    return;
                }
                await invoke('open_extension_folder', { scope: 'global', version });
            } catch (e) {
                toast.error(t('home.openGlobalDirFailed') + ': ' + String(e));
            } finally {
                Dialog.close();
            }
        },
        onCancel: () => {
            Dialog.close();
        },
        onClose: () => {
            Dialog.close();
        }
    });
};

const dirs = [
  { id: 'root', label: t('home.rootDir'), icon: FolderIcon, action: () => openDir('root') },
  { id: 'data', label: t('home.dataDir'), icon: DatabaseIcon, action: () => openDir('data') },
  { id: 'logs', label: t('home.logsDir'), icon: FileTextIcon, action: () => openDir('logs') },
  { id: 'tavern', label: t('home.tavernDir'), icon: BeerIcon, action: () => openDir('tavern') },
  { id: 'extension', label: t('home.extensionDir'), icon: PuzzleIcon, action: openExtensionFolder },
  { id: 'node', label: t('home.nodeDir'), icon: BoxIcon, action: () => openDir('node') },
]

const fetchVersions = async () => {
  // 优先从缓存读取
  const cachedAppVersion = localStorage.getItem('app_settings_app_version_cache');
  if (cachedAppVersion) appVersion.value = cachedAppVersion;

  const cachedNode = localStorage.getItem('app_settings_node_cache');
  if (cachedNode) {
    try {
      const parsedNode = JSON.parse(cachedNode);
      nodeVersion.value = parsedNode.version || t('home.notInstalled');
      nodePath.value = parsedNode.path || '';
    } catch(e) {}
  }

  const cachedConfig = localStorage.getItem('app_settings_config_cache');
  if (cachedConfig) {
    try {
      const parsedConfig = JSON.parse(cachedConfig);
      if (parsedConfig?.sillytavern?.version) {
        tavernVersion.value = parsedConfig.sillytavern.version;
      }
    } catch(e) {}
  }

  // 后台静默获取最新数据并更新缓存
  try {
    const appVer = await invoke<string>('get_app_version');
    if (appVer !== appVersion.value) {
      appVersion.value = appVer;
      localStorage.setItem('app_settings_app_version_cache', appVer);
    }
  } catch (e) {
    console.error(e);
  }

  try {
    const nodeInfo: any = await invoke('check_nodejs');
    const newVersion = nodeInfo.version || t('home.notInstalled');
    const newPath = nodeInfo.path || '';
    if (newVersion !== nodeVersion.value || newPath !== nodePath.value) {
      nodeVersion.value = newVersion;
      nodePath.value = newPath;
      localStorage.setItem('app_settings_node_cache', JSON.stringify(nodeInfo));
    }
  } catch (e) {
    if (nodeVersion.value !== t('home.notInstalled')) {
      nodeVersion.value = t('home.notInstalled');
    }
  }

  try {
    const tavernVer = await invoke<string>('get_tavern_version');
    if (tavernVer !== tavernVersion.value) {
      tavernVersion.value = tavernVer;
      
      // 合并到全局配置缓存中
      const currentCachedConfig = localStorage.getItem('app_settings_config_cache');
      let mergedConfig: any = { sillytavern: { version: tavernVer } };
      if (currentCachedConfig) {
        try {
          const parsed = JSON.parse(currentCachedConfig);
          mergedConfig = { ...parsed, sillytavern: { ...(parsed.sillytavern || {}), version: tavernVer } };
        } catch(e) {}
      }
      localStorage.setItem('app_settings_config_cache', JSON.stringify(mergedConfig));
    }
  } catch (e) {
    if (tavernVersion.value !== t('home.notInstalled')) {
      tavernVersion.value = t('home.notInstalled');
    }
  }
}

const handleToggleProcess = async () => {
  if (status.value === 1 || status.value === 2) {
    router.push('/console')
    await stopProcess()
  } else {
    router.push('/console')
    await startProcess()
  }
}

const handleOpenServer = async () => {
  if (serverUrl.value) {
    try {
      await openUrl(serverUrl.value)
    } catch (err) {
      toast.error(t('home.cannotOpenBrowser'))
    }
  }
}

onMounted(() => {
  fetchVersions()
})
</script>

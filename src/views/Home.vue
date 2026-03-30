<template>
  <div class="h-full flex flex-col gap-6 text-slate-800 dark:text-slate-200">
    <!-- 断点续传提示：只在一键安装中途中断（没有安装好酒馆）时才显示，且必须等数据加载完才渲染（避免中间态闪烁） -->
    <div v-if="isDataReady && setupCheckpoint && setupCheckpoint !== 'DONE' && !initialSetupCompleted && !hasAnyTavern" 
         class="mx-1 px-5 py-4 bg-blue-50 dark:bg-blue-900/40 border border-blue-200 dark:border-blue-800 rounded-2xl flex flex-col sm:flex-row items-center justify-between gap-4 animate-in slide-in-from-top-4 duration-500 shadow-sm"
    >
      <div class="flex items-center gap-4">
        <div class="w-12 h-12 rounded-xl bg-blue-100 dark:bg-blue-800/60 flex items-center justify-center text-blue-600 dark:text-blue-400 shrink-0">
          <HistoryIcon class="w-6 h-6 animate-pulse" />
        </div>
        <div>
          <h3 class="font-bold text-slate-800 dark:text-slate-100">{{ t('oneClick.resumeTitle') }}</h3>
          <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
            {{ t('oneClick.resumeDesc') }}: 
            <span class="font-medium text-blue-600 dark:text-blue-400">{{ getCheckpointName(setupCheckpoint) }}</span>
          </p>
        </div>
      </div>
      <div class="flex items-center gap-3 shrink-0">
        <button @click="clearSetupCheckpoint" class="text-xs font-medium text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors px-3 py-2">
          {{ t('common.ignore') }}
        </button>
        <button @click="handleResumeSetup" class="px-5 py-2.5 bg-blue-600 hover:bg-blue-700 text-white rounded-xl text-sm font-bold shadow-md shadow-blue-500/20 active:scale-95 transition-all">
          {{ t('oneClick.resumeButton') }}
        </button>
      </div>
    </div>

    <!-- 顶部 Banner -->
    <div
      class="w-full h-48 sm:h-56 rounded-2xl overflow-hidden shadow-sm relative group shrink-0"
    >
      <img src="../assets/images/banner.png" alt="Banner" class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105" />
      <div class="absolute inset-0 bg-gradient-to-t from-black/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
    </div>

    <!-- 中部 快捷目录和版本信息 -->
    <div
      class="flex-1 flex flex-col md:flex-row gap-6"
    >
      
      <!-- 左侧：快捷目录 -->
      <div class="flex-[3] bg-white dark:bg-slate-800 p-6 rounded-2xl shadow-sm border border-slate-100 dark:border-slate-700 flex flex-col">
        <h2 class="text-lg font-bold mb-5 flex items-center text-slate-800 dark:text-slate-200 shrink-0">
          <FolderOpenIcon class="w-5 h-5 mr-2 text-primary" />
          {{ t('home.quickDirectories') }}
        </h2>
        <div
          class="grid grid-cols-2 sm:grid-cols-3 gap-4"
        >
          <button
            v-for="(btn) in dirs" :key="btn.id"
            class="flex flex-col items-center justify-center gap-3 p-4 rounded-xl bg-slate-50 dark:bg-slate-900/50 border border-slate-100 dark:border-slate-700 hover:bg-blue-50 dark:hover:bg-blue-900/30 hover:border-blue-200 dark:hover:border-blue-800 hover:text-primary transition-colors group"
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
        <div
          class="flex-1 bg-white dark:bg-slate-800 p-6 rounded-2xl shadow-sm border border-slate-100 dark:border-slate-700 flex flex-col justify-center">
          <div class="flex items-center justify-between mb-5">
            <h2 class="text-lg font-bold flex items-center text-slate-800 dark:text-slate-200">
              <InfoIcon class="w-5 h-5 mr-2 text-primary" />
              {{ t('home.systemInfo') }}
            </h2>
            <button 
              v-if="status === 2 && serverUrl && networkMode === null && launchMode !== 'desktop'"
              @click="handleOpenServer"
              class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium text-emerald-600 bg-emerald-50 dark:bg-emerald-900/30 hover:bg-emerald-100 dark:hover:bg-emerald-900/50 border border-emerald-200 dark:border-emerald-800 transition-colors group"
            >
              <span>{{ t('home.visitTavern') }}</span>
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
            </button>
            <!-- 局域网/公网服务模式：显示网络链接按钮 -->
            <button
              v-else-if="status === 2 && networkMode !== null"
              @click="showNetworkDialog = true"
              :class="[
                'flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium border transition-colors',
                networkMode === 'lan'
                  ? 'text-emerald-700 bg-emerald-50 dark:bg-emerald-900/30 hover:bg-emerald-100 dark:hover:bg-emerald-900/50 border-emerald-300 dark:border-emerald-700'
                  : 'text-red-600 bg-red-50 dark:bg-red-900/30 hover:bg-red-100 dark:hover:bg-red-900/50 border-red-300 dark:border-red-700'
              ]"
            >
              <span>{{ networkMode === 'lan' ? t('home.lanLink') : t('home.publicLink') }}</span>
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
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
              <span class="font-bold text-slate-700 dark:text-slate-300">{{ tavernVersion === 'unknown' ? t('versions.unknownVersion') : (tavernVersion || t('home.notInstalled')) }}</span>
            </div>
          </div>
        </div>
        
        <!-- 一键启动按钮 / 安装NodeJs按钮 -->
        <button
          v-if="!initialSetupCompleted && !hasAnyTavern && (nodeVersion === t('home.notInstalled') || !nodeVersion)"
          :disabled="checkingEnv"
          class="btn shrink-0 min-h-[6rem] py-3 h-auto rounded-2xl shadow-md hover:shadow-lg border-none text-white flex flex-col items-center justify-center gap-1 group relative overflow-hidden bg-primary hover:bg-primary/90 disabled:opacity-70 disabled:cursor-not-allowed"
          @click="router.push('/settings?action=one_click_setup')"
        >
          <div class="absolute inset-0 bg-white/20 translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-in-out"></div>
          <div class="flex items-center gap-2 z-10">
            <PlayIcon class="w-7 h-7 fill-current" />
            <span class="text-2xl font-bold tracking-widest">{{ t('home.oneClickSetup') }}</span>
          </div>
          <span class="text-xs font-medium opacity-90 z-10">
            {{ t('home.oneClickSetupDesc') }}
          </span>
        </button>
        <button
          v-else-if="nodeVersion === t('home.notInstalled') || !nodeVersion"
          :disabled="checkingEnv"
          class="btn shrink-0 min-h-[6rem] py-3 h-auto rounded-2xl shadow-md hover:shadow-lg border-none text-white flex flex-col items-center justify-center gap-1 group relative overflow-hidden bg-orange-500 hover:bg-orange-600 disabled:opacity-70 disabled:cursor-not-allowed"
          @click="router.push('/settings?action=install_node')"
        >
          <div class="absolute inset-0 bg-white/20 translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-in-out"></div>
          <div class="flex items-center gap-2 z-10">
            <DownloadIcon class="w-7 h-7 fill-current" />
            <span class="text-2xl font-bold tracking-widest">{{ t('settings.nodejsInstall') }}</span>
          </div>
          <span class="text-xs font-medium opacity-90 z-10">
            {{ t('home.nodeRequiredDesc') }}
          </span>
        </button>
        <button
          v-else
          :disabled="checkingEnv"
          class="btn shrink-0 min-h-[6rem] py-3 h-auto rounded-2xl shadow-md hover:shadow-lg border-none text-white flex flex-col items-center justify-center gap-1 group relative overflow-hidden disabled:opacity-70 disabled:cursor-not-allowed"
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

  <!-- 局域网/公网连接弹窗 -->
  <NetworkLinkDialog
    v-if="networkMode !== null"
    :open="showNetworkDialog"
    :mode="networkMode"
    :port="networkPort"
    @close="showNetworkDialog = false"
  />
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
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
  Terminal as TerminalIcon,
  Download as DownloadIcon,
  GitBranch as GitIcon,
  History as HistoryIcon,
} from 'lucide-vue-next'

import { consoleStatus as status, serverUrl, networkMode, networkPort, startProcess, stopProcess, launchMode } from '../lib/consoleState'
import { scanManager } from '../lib/useScan'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Dialog } from '../lib/useDialog'
import { toast } from 'vue-sonner'
import NetworkLinkDialog from '../components/NetworkLinkDialog.vue'

const { t } = useI18n()
const router = useRouter()
const showNetworkDialog = ref(false)
const appVersion = ref('')
const nodeVersion = ref('')
const tavernVersion = ref('')
const nodePath = ref('')
const gitPath = ref('')
const initialSetupCompleted = ref(false)
const checkingEnv = ref(true)
const setupCheckpoint = ref<string | null>(null)
// 标记关键数据（在线已安装列表、本地列表）是否已全部加载完，防止断点续传提示在中间态一闪而过
const isDataReady = ref(false)

// 在线下载安装的版本列表（用于判断是否有任何已安装的酒馆）
const installedOnlineVersions = ref<string[]>([])

/**
 * 是否已有任何酒馆实例：
 * - 本地列表（手动添加 / 扫描到）有项目，或
 * - 在线下载页已安装了任意版本
 * 满足任意一条 → 视为"已有酒馆"，隐藏一键安装按钮和断点续传提示
 */
const hasAnyTavern = computed(() =>
  scanManager.state.localList.length > 0 || installedOnlineVersions.value.length > 0
)

const getCheckpointName = (cp: string) => {
    if (cp === 'START') return t('oneClick.gitDetecting');
    if (cp === 'GIT_DONE') return t('oneClick.nodeDetecting');
    if (cp === 'NODE_DONE') return t('oneClick.fetchingVersions');
    if (cp.startsWith('ST_INSTALLED')) return t('oneClick.installingDeps');
    return cp;
};

const clearSetupCheckpoint = async () => {
    try {
        const config: any = await invoke('get_app_config');
        config.setupCheckpoint = null;
        await invoke('save_app_config', { config });
        setupCheckpoint.value = null;
    } catch (e) {}
};

const handleResumeSetup = () => {
    if (!setupCheckpoint.value) return;
    if (setupCheckpoint.value === 'START' || setupCheckpoint.value === 'GIT_DONE') {
        router.push('/settings?action=one_click_setup');
    } else {
        router.push('/versions?action=one_click_setup_st');
    }
};

const openDir = async (dirType: string) => {
  try {
    let customPath = null;
    if (dirType === 'node' && nodePath.value) {
      customPath = nodePath.value;
    } else if (dirType === 'git' && gitPath.value) {
      customPath = gitPath.value;
    }
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
                let version: any = null;
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
                if (!version || version.version === t('home.notInstalled')) {
                    toast.warning(t('home.noTavernInstalled'));
                    return;
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
                let version: any = null;
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
                if (!version || version.version === t('home.notInstalled')) {
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
  { id: 'git', label: t('home.gitDir'), icon: GitIcon, action: () => openDir('git') },
]

const fetchVersions = async () => {
  checkingEnv.value = true;
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
      if (parsedConfig?.sillytavern?.version?.version) {
        tavernVersion.value = parsedConfig.sillytavern.version.version;
      }
      if (parsedConfig?.initialSetupCompleted !== undefined) {
        initialSetupCompleted.value = parsedConfig.initialSetupCompleted;
      } else {
        initialSetupCompleted.value = false;
      }
      setupCheckpoint.value = parsedConfig?.setupCheckpoint || null;
    } catch(e) {}
  }

  // Fetch app config
  try {
    const config: any = await invoke('get_app_config');
    if (config?.initialSetupCompleted !== undefined) {
      initialSetupCompleted.value = config.initialSetupCompleted;
    } else {
      initialSetupCompleted.value = false;
    }
    setupCheckpoint.value = config.setupCheckpoint || null;

    // 从 config 读取本地列表（和 Versions.vue 使用相同数据源）
    if (config.localSillytavernList && Array.isArray(config.localSillytavernList)) {
      scanManager.state.localList = config.localSillytavernList;
    }
  } catch (e) {}

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
    const gitInfo: any = await invoke('check_git');
    gitPath.value = gitInfo.path || '';
  } catch (e) {
    console.error(e);
  }

  try {
    const tavernVerItem: any = await invoke('get_tavern_version');
    if (tavernVerItem.version !== tavernVersion.value) {
      tavernVersion.value = tavernVerItem.version;
      
      // 合并到全局配置缓存中
      const currentCachedConfig = localStorage.getItem('app_settings_config_cache');
      let mergedConfig: any = { sillytavern: { version: tavernVerItem } };
      if (currentCachedConfig) {
        try {
          const parsed = JSON.parse(currentCachedConfig);
          mergedConfig = { ...parsed, sillytavern: { ...(parsed.sillytavern || {}), version: tavernVerItem } };
        } catch(e) {}
      }
      localStorage.setItem('app_settings_config_cache', JSON.stringify(mergedConfig));
    }
  } catch (e) {
    if (tavernVersion.value !== t('home.notInstalled')) {
      tavernVersion.value = t('home.notInstalled');
    }
  }

  // 查询在线下载页面已安装的版本列表
  try {
    const installed: any[] = await invoke('get_installed_versions_info');
    installedOnlineVersions.value = installed.map(v => v.version);
  } catch (e) {
    installedOnlineVersions.value = [];
  }

  // 如果确认已有酒馆但 setupCheckpoint 仍是中间状态（残留/误判），自动清掉
  if (hasAnyTavern.value && setupCheckpoint.value && setupCheckpoint.value !== 'DONE') {
    setupCheckpoint.value = null;
    try {
      const cfg: any = await invoke('get_app_config');
      cfg.setupCheckpoint = null;
      cfg.initialSetupCompleted = true;
      await invoke('save_app_config', { config: cfg });
      initialSetupCompleted.value = true;
    } catch(e) {}
  }

  // 所有关键数据已就绪，现在才允许渲染断点续传提示（避免中间态一闪而过）
  isDataReady.value = true;
  checkingEnv.value = false;
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

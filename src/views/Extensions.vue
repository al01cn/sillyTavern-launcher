<template>
  <div class="space-y-6">
    <!-- Header Info -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-slate-800 dark:text-slate-100">{{ t('extensions.title') }}</h1>
        <p class="text-slate-600 dark:text-slate-400 text-sm mt-1">{{ t('extensions.subtitle') }}</p>
      </div>
      <div class="flex items-center gap-3">
        <button 
          @click="handleInstallPlugin"
          class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 bg-slate-900 dark:bg-slate-700 text-white hover:bg-slate-800 dark:hover:bg-slate-600 shadow-lg shadow-slate-200 dark:shadow-slate-900 active:scale-95"
        >
          <Download class="w-4 h-4" />
          {{ t('extensions.installExtension') }}
        </button>
        <button 
          @click="openExtensionFolder"
          class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-700 hover:text-slate-900 dark:hover:text-slate-100 shadow-sm"
        >
          <FolderOpen class="w-4 h-4" />
          {{ t('extensions.openExtensionFolder') }}
        </button>
      </div>
    </div>

    <!-- Version Selection -->
    <div class="bg-white dark:bg-slate-800 p-5 rounded-2xl shadow-sm border border-slate-100 dark:border-slate-700 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-blue-50 dark:bg-blue-900/30 text-blue-500 dark:text-blue-400 flex items-center justify-center">
          <Puzzle class="w-5 h-5" />
        </div>
        <div>
          <h3 class="font-bold text-slate-800 dark:text-slate-100">{{ t('extensions.currentTavernVersion') }}</h3>
          <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">{{ t('extensions.checkCompatibility') }}</p>
        </div>
      </div>
      <div class="flex items-center gap-3">
        <div class="px-4 py-2 bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 rounded-xl text-sm font-medium text-slate-700 dark:text-slate-300">
          {{ selectedVersion || t('versions.notSet') }}
        </div>
        <router-link 
          to="/versions"
          class="text-xs text-blue-500 hover:text-blue-600 dark:text-blue-400 dark:hover:text-blue-300 font-medium transition-colors ml-1"
        >
          {{ t('extensions.goSwitch') }}
        </router-link>
      </div>
    </div>

    <!-- Extensions List -->
    <div v-if="selectedVersion" class="bg-white dark:bg-slate-800 rounded-2xl shadow-sm border border-slate-100 dark:border-slate-700 overflow-hidden">
      <div class="p-5 border-b border-slate-100 dark:border-slate-700 flex items-center justify-between">
        <h3 class="font-bold text-slate-800 dark:text-slate-100 flex items-center gap-2">
          <Puzzle class="w-5 h-5 text-slate-400" />
          {{ t('extensions.installedExtensions') }}
        </h3>
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium text-slate-600 dark:text-slate-400">{{ t('extensions.showSystemExtensions') }}</span>
            <button 
              @click="showOfficial = !showOfficial"
              class="relative inline-flex h-5 w-9 items-center rounded-full transition-colors duration-200 ease-in-out focus:outline-none"
              :class="showOfficial ? 'bg-blue-500' : 'bg-slate-200'"
            >
              <span 
                class="inline-block h-3 w-3 transform rounded-full bg-white transition duration-200 ease-in-out shadow-sm"
                :class="showOfficial ? 'translate-x-5' : 'translate-x-1'"
              />
            </button>
          </div>
          <button @click="refresh(true)" class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 transition-colors p-2 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700">
            <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': loading }" />
          </button>
        </div>
      </div>
      
      <div v-if="loading && extensions.length === 0" class="p-10 text-center text-slate-400">
        <Loader2 class="w-8 h-8 animate-spin mx-auto mb-2" />
        <p>{{ t('extensions.scanningExtensions') }}</p>
      </div>

      <div v-else-if="filteredExtensions.length === 0" class="p-10 text-center text-slate-400 flex flex-col items-center">
        <Puzzle class="w-12 h-12 mb-3 text-slate-300" />
        <p>{{ t('extensions.noExtensionsFound') }}</p>
        <p class="text-xs mt-1">{{ t('extensions.noExtensionsHint') }}</p>
      </div>

      <div v-else class="flex flex-col">
        <div class="divide-y divide-slate-50 dark:divide-slate-700">
          <div v-for="ext in paginatedExtensions" :key="ext.id" class="p-5 hover:bg-slate-50/50 dark:hover:bg-slate-700/50 transition-colors group">
            <div class="flex items-start justify-between gap-4">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-3 mb-2">
                  <h4 class="font-bold text-slate-800 dark:text-slate-100 text-lg" :class="{'opacity-50 line-through': !ext.enabled}">{{ ext.manifest.display_name || ext.id }}</h4>
                  <span class="px-2 py-0.5 rounded-full bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-400 text-[10px] font-bold tracking-wide" :class="{'opacity-50': !ext.enabled}">
                    v{{ ext.manifest.version || t('versions.notSet') }}
                  </span>
                  <span v-if="ext.manifest.minimum_client_version !== undefined && ext.manifest.minimum_client_version !== null" class="px-2 py-0.5 rounded-full bg-blue-50 dark:bg-blue-900/30 text-blue-500 dark:text-blue-400 text-[10px] font-bold tracking-wide" :class="{'opacity-50': !ext.enabled}" :title="t('extensions.minClientVersion')">
                    ST &ge; {{ ext.manifest.minimum_client_version }}
                  </span>
                  <!-- Compatibility Warning -->
                  <span v-if="ext.manifest.minimum_client_version !== undefined && ext.manifest.minimum_client_version !== null && !isCompatible(ext.manifest.minimum_client_version)" class="px-2 py-0.5 rounded-full bg-red-50 dark:bg-red-900/30 text-red-500 dark:text-red-400 text-[10px] font-bold tracking-wide flex items-center gap-1" :class="{'opacity-50': !ext.enabled}" :title="t('extensions.maybeIncompatible')">
                    <AlertTriangle class="w-3 h-3" />
                    {{ t('extensions.maybeIncompatible') }}
                  </span>
                  <!-- Disabled Badge -->
                  <span v-if="!ext.enabled" class="px-2 py-0.5 rounded-full bg-slate-200 dark:bg-slate-700 text-slate-500 dark:text-slate-400 text-[10px] font-bold tracking-wide">
                    {{ t('extensions.disabled') }}
                  </span>
                  <!-- Scope Badge -->
                  <span class="px-2 py-0.5 rounded-full text-[10px] font-bold tracking-wide" :class="ext.scope === 'global' ? 'bg-purple-50 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400' : 'bg-emerald-50 dark:bg-emerald-900/30 text-emerald-600 dark:text-emerald-400'">
                    {{ ext.scope === 'global' ? t('extensions.scope.global') : t('extensions.scope.user') }}
                  </span>
                  <!-- System Badge -->
                  <span v-if="ext.is_system" class="px-2 py-0.5 rounded-full bg-amber-50 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400 text-[10px] font-bold tracking-wide flex items-center gap-1">
                    <ShieldCheck class="w-3 h-3" />
                    {{ t('extensions.system') }}
                  </span>
                </div>
                
                <div class="flex items-center gap-4 text-sm text-slate-500 dark:text-slate-400 mb-1">
                  <span class="flex items-center gap-1">
                     <User class="w-3.5 h-3.5" />
                     {{ ext.manifest.author || t('extensions.unknownAuthor') }}
                  </span>
                  <span class="text-slate-300">|</span>
                  <span class="flex items-center gap-1 font-mono text-xs">
                     <Folder class="w-3.5 h-3.5" />
                     {{ ext.id }}
                  </span>
                  <div class="flex items-center gap-2 ml-2">
                    <a 
                      v-if="ext.manifest.homePage && ext.manifest.homePage !== 'None' && ext.manifest.homePage.trim() !== ''"
                      :href="ext.manifest.homePage" 
                      target="_blank" 
                      @click.prevent="openUrl(ext.manifest.homePage)"
                      class="px-2 py-1 text-[10px] font-medium text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-700 hover:bg-slate-200 dark:hover:bg-slate-600 hover:text-slate-700 dark:hover:text-slate-200 rounded-md transition-colors flex items-center gap-1"
                      :title="t('extensions.visitHomepage')"
                    >
                      <Globe class="w-3 h-3" />
                      {{ t('extensions.visitHomepage') }}
                    </a>
                    <button 
                      @click="openSpecificExtensionFolder(ext.dir_path)"
                      class="px-2 py-1 text-[10px] font-medium text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-700 hover:bg-slate-200 dark:hover:bg-slate-600 hover:text-slate-700 dark:hover:text-slate-200 rounded-md transition-colors flex items-center gap-1"
                      :title="t('extensions.openDirectory')"
                    >
                      <FolderOpen class="w-3 h-3" />
                      {{ t('extensions.openDirectory') }}
                    </button>
                  </div>
                </div>
              </div>

              <div class="flex flex-col items-end gap-3 shrink-0">
                 <div v-if="!ext.is_system" class="flex items-center gap-2">
                   <!-- Enable/Disable Switch -->
                   <div class="flex items-center gap-2">
                     <span class="text-sm font-medium" :class="ext.enabled ? 'text-slate-600 dark:text-slate-400' : 'text-slate-400'">{{ ext.enabled ? t('extensions.enabled') : t('extensions.disabled') }}</span>
                     <button 
                       @click="toggleEnable(ext)"
                       class="relative inline-flex h-5 w-9 items-center rounded-full transition-colors duration-200 ease-in-out focus:outline-none"
                       :class="ext.enabled ? 'bg-blue-500' : 'bg-slate-200'"
                     >
                       <span 
                         class="inline-block h-3 w-3 transform rounded-full bg-white transition duration-200 ease-in-out shadow-sm"
                         :class="ext.enabled ? 'translate-x-5' : 'translate-x-1'"
                       />
                     </button>
                   </div>
                   
                   <!-- Delete Button -->
                   <button 
                     @click="deleteExtension(ext)"
                     class="p-1.5 text-slate-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-lg transition-colors"
                     :title="t('extensions.deleteExtension')"
                   >
                     <Trash2 class="w-4 h-4" />
                   </button>
                 </div>
                 
                 <!-- Auto Update Switch -->
                 <div v-if="ext.manifest.auto_update !== undefined && ext.manifest.auto_update !== null" class="flex items-center gap-2" :class="{ 'pr-[34px]': !ext.is_system }">
                   <span class="text-sm text-slate-500 dark:text-slate-400 font-medium">{{ t('extensions.autoUpdate') }}</span>
                   <button 
                     @click="toggleAutoUpdate(ext)"
                     class="relative inline-flex h-5 w-9 items-center rounded-full transition-colors duration-200 ease-in-out focus:outline-none"
                     :class="ext.manifest.auto_update ? 'bg-emerald-500' : 'bg-slate-200'"
                   >
                     <span 
                       class="inline-block h-3 w-3 transform rounded-full bg-white transition duration-200 ease-in-out shadow-sm"
                       :class="ext.manifest.auto_update ? 'translate-x-5' : 'translate-x-1'"
                     />
                   </button>
                 </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Pagination Controls -->
        <div v-if="totalPages > 1" class="p-4 border-t border-slate-100 flex items-center justify-between bg-slate-50/50">
          <span class="text-sm text-slate-500">
            共 {{ filteredExtensions.length }} 个扩展
          </span>
          <div class="flex items-center gap-2">
            <button 
              @click="prevPage" 
              :disabled="currentPage === 1"
              class="p-1.5 rounded-lg border border-slate-200 text-slate-600 hover:bg-white hover:text-slate-900 disabled:opacity-50 disabled:cursor-not-allowed transition-colors bg-slate-50"
            >
              <ChevronLeft class="w-4 h-4" />
            </button>
            <span class="text-sm font-medium text-slate-700 min-w-[3rem] text-center">
              {{ currentPage }} / {{ totalPages }}
            </span>
            <button 
              @click="nextPage" 
              :disabled="currentPage === totalPages"
              class="p-1.5 rounded-lg border border-slate-200 text-slate-600 hover:bg-white hover:text-slate-900 disabled:opacity-50 disabled:cursor-not-allowed transition-colors bg-slate-50"
            >
              <ChevronRight class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </div>
    
    <div v-else class="bg-white dark:bg-slate-800 p-10 rounded-2xl shadow-sm border border-slate-100 dark:border-slate-700 text-center text-slate-400 dark:text-slate-500">
      <Puzzle class="w-12 h-12 mx-auto mb-3 text-slate-300 dark:text-slate-600" />
      <p>{{ t('extensions.selectVersionFirst') }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { openUrl as tauriOpenUrl } from '@tauri-apps/plugin-opener';
import { useI18n } from 'vue-i18n';
import { 
    Download, FolderOpen, Puzzle, RefreshCw, Loader2, 
    User, Globe, Folder, AlertTriangle, Trash2,
    ChevronLeft, ChevronRight, ShieldCheck
} from 'lucide-vue-next';
import { toast } from 'vue-sonner';
import { Dialog } from '../lib/useDialog';
import { openInstallExtensionDialog } from '../lib/useExtensionInstall';

const { t } = useI18n();

interface ExtensionManifest {
    display_name?: string;
    author?: string;
    version?: string;
    homePage?: string;
    auto_update?: boolean;
    minimum_client_version?: string;
}

interface ExtensionInfo {
    id: string;
    manifest: ExtensionManifest;
    dir_path: string;
    enabled: boolean;
    is_system: boolean;
    scope: string;
}

interface InstalledVersionInfo {
    version: string;
    has_node_modules: boolean;
}

const extensions = ref<ExtensionInfo[]>([]);
const installedVersions = ref<InstalledVersionInfo[]>([]);
const selectedVersion = ref('');
const loading = ref(false);
const showOfficial = ref(false);

// Filtered and sorted extensions
const filteredExtensions = computed(() => {
    let result = extensions.value;
    if (!showOfficial.value) {
        result = result.filter(ext => !ext.is_system);
    }
    
    // Sort logic: 
    // 1. User scope extensions first
    // 2. Global third-party extensions next
    // 3. System extensions last
    return [...result].sort((a, b) => {
        const getPriority = (ext: ExtensionInfo) => {
            if (ext.scope === 'user') return 1;
            if (ext.scope === 'global' && !ext.is_system) return 2;
            if (ext.is_system) return 3;
            return 4;
        };
        
        return getPriority(a) - getPriority(b);
    });
});

// Pagination
const currentPage = ref(1);
const itemsPerPage = 5;

const totalPages = computed(() => {
    return Math.max(1, Math.ceil(filteredExtensions.value.length / itemsPerPage));
});

const paginatedExtensions = computed(() => {
    const start = (currentPage.value - 1) * itemsPerPage;
    const end = start + itemsPerPage;
    return filteredExtensions.value.slice(start, end);
});

const prevPage = () => {
    if (currentPage.value > 1) {
        currentPage.value--;
    }
};

const nextPage = () => {
    if (currentPage.value < totalPages.value) {
        currentPage.value++;
    }
};

// A simple semver compare to check minimum_client_version
const isCompatible = (minVersion?: string) => {
    if (!minVersion || !selectedVersion.value) return true;
    
    // basic semver compare
    const v1 = selectedVersion.value.replace(/[^0-9.]/g, '').split('.').map(Number);
    const v2 = minVersion.replace(/[^0-9.]/g, '').split('.').map(Number);
    
    for (let i = 0; i < Math.max(v1.length, v2.length); i++) {
        const num1 = v1[i] || 0;
        const num2 = v2[i] || 0;
        if (num1 > num2) return true;
        if (num1 < num2) return false;
    }
    return true; // Equal
};

const loadVersions = async () => {
    try {
        installedVersions.value = await invoke('get_installed_versions_info');
        
        let versionFromConfig = '';
        const cachedConfig = localStorage.getItem('app_settings_config_cache');
        if (cachedConfig) {
            try {
                const parsed = JSON.parse(cachedConfig);
                if (parsed?.sillytavern?.version) {
                    versionFromConfig = parsed.sillytavern.version;
                }
            } catch(e) {}
        }
        
        if (!versionFromConfig) {
            const config: any = await invoke('get_app_config');
            if (config?.sillytavern?.version) {
                versionFromConfig = config.sillytavern.version;
            }
        }
        
        if (versionFromConfig) {
            // Set to current version if it exists in installed list
            if (installedVersions.value.some(v => v.version === versionFromConfig)) {
                selectedVersion.value = versionFromConfig;
            } else if (installedVersions.value.length > 0) {
                selectedVersion.value = installedVersions.value[0].version;
            }
        } else if (installedVersions.value.length > 0) {
            selectedVersion.value = installedVersions.value[0].version;
        }
    } catch (e) {
        console.error(e);
    }
};

const refresh = async (forceUpdate = false) => {
    if (!selectedVersion.value) return;
    
    const cacheKey = `extensions_cache_${selectedVersion.value}`;
    
    // 如果不是强制更新，优先尝试从缓存中加载扩展列表，实现秒开
    if (!forceUpdate) {
        const cachedExtensions = localStorage.getItem(cacheKey);
        if (cachedExtensions) {
            try {
                const parsed = JSON.parse(cachedExtensions);
                if (Array.isArray(parsed)) {
                    extensions.value = parsed;
                }
            } catch (e) {
                console.error('扩展缓存解析失败:', e);
            }
        }
    }

    loading.value = true;
    try {
        const fetchedExtensions = await invoke<ExtensionInfo[]>('get_extensions', { version: selectedVersion.value });
        const fetchedString = JSON.stringify(fetchedExtensions);
        const currentCache = localStorage.getItem(cacheKey);
        
        // 如果数据有变化或者是强制更新，才更新列表和缓存
        if (fetchedString !== currentCache || forceUpdate) {
            extensions.value = fetchedExtensions;
            localStorage.setItem(cacheKey, fetchedString);
            // 只有当数据真正发生变化时，才考虑重置页码（可选，或者保留当前页）
            if (forceUpdate) {
                currentPage.value = 1;
            }
        }
    } catch (e) {
        console.error(e);
        toast.error(t('extensions.fetchFailed') + ': ' + String(e));
    } finally {
        loading.value = false;
    }
};

const openExtensionFolder = async () => {
    Dialog.warning({
        title: t('extensions.selectDirectory'),
        msg: t('extensions.selectDirectoryMsg'),
        confirmText: t('home.currentUser'),
        thirdBtnText: t('home.global'),
        showCancel: false,
        onConfirm: async () => {
            try {
                await invoke('open_extension_folder', { scope: 'user', version: selectedVersion.value });
            } catch (e) {
                toast.error(t('extensions.openUserDirFailed') + ': ' + String(e));
            } finally {
                Dialog.close();
            }
        },
        onThirdBtn: async () => {
            if (!selectedVersion.value) {
                toast.warning(t('extensions.selectVersionWarning'));
                return;
            }
            try {
                await invoke('open_extension_folder', { scope: 'global', version: selectedVersion.value });
            } catch (e) {
                toast.error(t('extensions.openGlobalDirFailed') + ': ' + String(e));
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

const openSpecificExtensionFolder = async (dirPath: string) => {
    try {
        await invoke('open_specific_extension_folder', { dirPath });
    } catch (e) {
        toast.error(t('extensions.openDirFailed') + ': ' + String(e));
    }
};

const handleInstallPlugin = () => {
    openInstallExtensionDialog(selectedVersion.value, () => {
        refresh(true);
    });
};

const openUrl = async (url?: string) => {
    if (!url) return;
    try {
        await tauriOpenUrl(url);
    } catch (e) {
        toast.error(t('tools.failedToOpen') + ': ' + String(e));
    }
};

const updateCache = () => {
    if (!selectedVersion.value) return;
    const cacheKey = `extensions_cache_${selectedVersion.value}`;
    localStorage.setItem(cacheKey, JSON.stringify(extensions.value));
};

const toggleEnable = async (ext: ExtensionInfo) => {
    const newStatus = !ext.enabled;
    // Optimistic update
    ext.enabled = newStatus;
    try {
        await invoke('toggle_extension_enable', {
            id: ext.id,
            enable: newStatus,
            dirPath: ext.dir_path
        });
        updateCache();
        toast.success(t(newStatus ? 'extensions.extensionEnabled' : 'extensions.extensionDisabled', { name: ext.manifest.display_name || ext.id }));
    } catch (e) {
        // Revert on failure
        ext.enabled = !newStatus;
        toast.error(t('extensions.toggleEnableFailed') + ': ' + String(e));
    }
};

const deleteExtension = (ext: ExtensionInfo) => {
    const extName = ext.manifest.display_name || ext.id;
    Dialog.warning({
        title: t('extensions.deleteExtension'),
        msg: t('extensions.confirmDeleteExt', { name: extName }),
        confirmText: t('common.delete'),
        cancelText: t('common.cancel'),
        onConfirm: async () => {
            try {
                await invoke('delete_extension', { 
                    id: ext.id,
                    dirPath: ext.dir_path
                });
                toast.success(t('extensions.extensionDeleted', { name: extName }));
                // Remove from list
                extensions.value = extensions.value.filter(e => e.id !== ext.id);
                updateCache();
                // Adjust page if necessary
                if (paginatedExtensions.value.length === 0 && currentPage.value > 1) {
                    currentPage.value--;
                }
            } catch (e) {
                toast.error(t('extensions.deleteFailed') + ': ' + String(e));
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

const toggleAutoUpdate = async (ext: ExtensionInfo) => {
    const newValue = !ext.manifest.auto_update;
    // Optimistic update
    ext.manifest.auto_update = newValue;
    try {
        await invoke('toggle_extension_auto_update', { 
            id: ext.id, 
            autoUpdate: newValue,
            dirPath: ext.dir_path
        });
        updateCache();
        toast.success(t(newValue ? 'extensions.autoUpdateEnabled' : 'extensions.autoUpdateDisabled'));
    } catch (e) {
        // Revert on failure
        ext.manifest.auto_update = !newValue;
        toast.error(t('extensions.toggleAutoUpdateFailed') + ': ' + String(e));
    }
};

onMounted(async () => {
    await loadVersions();
    if (selectedVersion.value) {
        refresh();
    }
});
</script>

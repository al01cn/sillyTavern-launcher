<template>
  <div class="space-y-6">
    <!-- Header Info -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-slate-800">扩展管理</h1>
        <p class="text-slate-600 text-sm mt-1">管理酒馆已安装的第三方扩展插件</p>
      </div>
      <div class="flex items-center gap-3">
        <button 
          @click="handleInstallPlugin"
          class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 bg-slate-900 text-white hover:bg-slate-800 shadow-lg shadow-slate-200 active:scale-95"
        >
          <Download class="w-4 h-4" />
          安装插件
        </button>
        <button 
          @click="openExtensionFolder"
          class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 bg-white border border-slate-200 text-slate-600 hover:bg-slate-50 hover:text-slate-900 shadow-sm"
        >
          <FolderOpen class="w-4 h-4" />
          打开插件文件夹
        </button>
      </div>
    </div>

    <!-- Version Selection -->
    <div class="bg-white p-5 rounded-2xl shadow-sm border border-slate-100 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-blue-50 text-blue-500 flex items-center justify-center">
          <Puzzle class="w-5 h-5" />
        </div>
        <div>
          <h3 class="font-bold text-slate-800">当前选择的酒馆版本</h3>
          <p class="text-xs text-slate-500 mt-0.5">将根据此版本检查扩展兼容性</p>
        </div>
      </div>
      <div class="flex items-center gap-3">
        <div class="px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl text-sm font-medium text-slate-700">
          {{ selectedVersion || '未选择版本' }}
        </div>
        <router-link 
          to="/versions"
          class="text-xs text-blue-500 hover:text-blue-600 font-medium transition-colors ml-1"
        >
          去切换
        </router-link>
      </div>
    </div>

    <!-- Extensions List -->
    <div v-if="selectedVersion" class="bg-white rounded-2xl shadow-sm border border-slate-100 overflow-hidden">
      <div class="p-5 border-b border-slate-100 flex items-center justify-between">
        <h3 class="font-bold text-slate-800 flex items-center gap-2">
          <Puzzle class="w-5 h-5 text-slate-400" />
          已安装扩展
        </h3>
        <button @click="refresh" class="text-slate-400 hover:text-slate-600 transition-colors p-2 rounded-lg hover:bg-slate-50">
          <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': loading }" />
        </button>
      </div>
      
      <div v-if="loading && extensions.length === 0" class="p-10 text-center text-slate-400">
        <Loader2 class="w-8 h-8 animate-spin mx-auto mb-2" />
        <p>正在扫描扩展...</p>
      </div>

      <div v-else-if="extensions.length === 0" class="p-10 text-center text-slate-400 flex flex-col items-center">
        <Puzzle class="w-12 h-12 mb-3 text-slate-300" />
        <p>没有找到任何扩展</p>
        <p class="text-xs mt-1">您可以点击右上角按钮打开文件夹放入扩展</p>
      </div>

      <div v-else class="divide-y divide-slate-50">
        <div v-for="ext in extensions" :key="ext.id" class="p-5 hover:bg-slate-50/50 transition-colors group">
          <div class="flex items-start justify-between gap-4">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-3 mb-2">
                <h4 class="font-bold text-slate-800 text-lg">{{ ext.manifest.display_name || ext.id }}</h4>
                <span class="px-2 py-0.5 rounded-full bg-slate-100 text-slate-600 text-[10px] font-bold tracking-wide">
                  v{{ ext.manifest.version || '未知' }}
                </span>
                <span v-if="ext.manifest.minimum_client_version !== undefined && ext.manifest.minimum_client_version !== null" class="px-2 py-0.5 rounded-full bg-blue-50 text-blue-500 text-[10px] font-bold tracking-wide" title="最小支持客户端版本">
                  ST &ge; {{ ext.manifest.minimum_client_version }}
                </span>
                <!-- Compatibility Warning -->
                <span v-if="ext.manifest.minimum_client_version !== undefined && ext.manifest.minimum_client_version !== null && !isCompatible(ext.manifest.minimum_client_version)" class="px-2 py-0.5 rounded-full bg-red-50 text-red-500 text-[10px] font-bold tracking-wide flex items-center gap-1" title="当前版本可能不兼容">
                  <AlertTriangle class="w-3 h-3" />
                  可能不兼容
                </span>
              </div>
              
              <div class="flex items-center gap-4 text-sm text-slate-500 mb-1">
                <span class="flex items-center gap-1">
                   <User class="w-3.5 h-3.5" />
                   {{ ext.manifest.author || '未知作者' }}
                </span>
                <span class="text-slate-300">|</span>
                <span class="flex items-center gap-1 font-mono text-xs">
                   <Folder class="w-3.5 h-3.5" />
                   {{ ext.id }}
                </span>
              </div>
              
              <div v-if="ext.manifest.homePage" class="mt-2">
                <a 
                  :href="ext.manifest.homePage" 
                  target="_blank" 
                  @click.prevent="openUrl(ext.manifest.homePage)"
                  class="inline-flex items-center gap-1 text-sm text-blue-500 hover:text-blue-600 transition-colors"
                >
                  <Globe class="w-3.5 h-3.5" />
                  访问扩展主页
                  <ExternalLink class="w-3 h-3" />
                </a>
              </div>
            </div>

            <div class="flex flex-col items-end gap-3 shrink-0">
               <div v-if="ext.manifest.auto_update !== undefined && ext.manifest.auto_update !== null" class="flex items-center gap-2">
                 <span class="text-sm text-slate-500 font-medium">自动更新</span>
                 <button 
                   @click="toggleAutoUpdate(ext)"
                   class="relative inline-flex h-5 w-9 items-center rounded-full transition-colors duration-200 ease-in-out focus:outline-none"
                   :class="ext.manifest.auto_update ? 'bg-emerald-500' : 'bg-slate-200'"
                 >
                   <span 
                     class="inline-block h-3 w-3 transform rounded-full bg-white transition duration-200 ease-in-out"
                     :class="ext.manifest.auto_update ? 'translate-x-5' : 'translate-x-1'"
                   />
                 </button>
               </div>
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <div v-else class="bg-white p-10 rounded-2xl shadow-sm border border-slate-100 text-center text-slate-400">
      <Puzzle class="w-12 h-12 mx-auto mb-3 text-slate-300" />
      <p>请先在上方选择一个酒馆版本来查看扩展</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { openUrl as tauriOpenUrl } from '@tauri-apps/plugin-opener';
import { 
    Download, FolderOpen, Puzzle, RefreshCw, Loader2, 
    User, Globe, ExternalLink, Folder, AlertTriangle
} from 'lucide-vue-next';
import { toast } from 'vue-sonner';

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
}

interface InstalledVersionInfo {
    version: string;
    has_node_modules: boolean;
}

const extensions = ref<ExtensionInfo[]>([]);
const installedVersions = ref<InstalledVersionInfo[]>([]);
const selectedVersion = ref('');
const loading = ref(false);

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
        const config: any = await invoke('get_app_config');
        if (config.sillytavern && config.sillytavern.version) {
            // Set to current version if it exists in installed list
            if (installedVersions.value.some(v => v.version === config.sillytavern.version)) {
                selectedVersion.value = config.sillytavern.version;
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

const refresh = async () => {
    if (!selectedVersion.value) return;
    loading.value = true;
    try {
        extensions.value = await invoke('get_extensions');
    } catch (e) {
        console.error(e);
        toast.error('获取扩展列表失败: ' + String(e));
    } finally {
        loading.value = false;
    }
};

const openExtensionFolder = async () => {
    try {
        await invoke('open_extension_folder');
    } catch (e) {
        toast.error('打开文件夹失败: ' + String(e));
    }
};

const handleInstallPlugin = () => {
    toast.info('请在打开的文件夹中放入解压后的插件，然后刷新列表');
    openExtensionFolder();
};

const openUrl = async (url?: string) => {
    if (!url) return;
    try {
        await tauriOpenUrl(url);
    } catch (e) {
        toast.error('无法打开链接: ' + String(e));
    }
};

const toggleAutoUpdate = async (ext: ExtensionInfo) => {
    const newValue = !ext.manifest.auto_update;
    // Optimistic update
    ext.manifest.auto_update = newValue;
    try {
        await invoke('toggle_extension_auto_update', { 
            id: ext.id, 
            autoUpdate: newValue 
        });
        toast.success(`已${newValue ? '开启' : '关闭'}自动更新`);
    } catch (e) {
        // Revert on failure
        ext.manifest.auto_update = !newValue;
        toast.error('切换自动更新失败: ' + String(e));
    }
};

onMounted(async () => {
    await loadVersions();
    if (selectedVersion.value) {
        refresh();
    }
});
</script>

<template>
  <div class="space-y-6">
    <!-- Header Info -->
    <div class="grid grid-cols-2 gap-4">
      <div class="bg-white p-6 rounded-2xl shadow-sm border border-slate-100 flex items-center justify-between">
        <div>
          <p class="text-xs font-bold text-slate-400 uppercase tracking-wider mb-1">当前版本</p>
          <h2 class="text-2xl font-black text-slate-800">{{ currentVersion || '未设置' }}</h2>
        </div>
        <div class="w-12 h-12 rounded-xl bg-blue-50 text-blue-500 flex items-center justify-center">
          <CheckCircle2 class="w-6 h-6" />
        </div>
      </div>
      <div class="bg-white p-6 rounded-2xl shadow-sm border border-slate-100 flex items-center justify-between">
        <div>
          <p class="text-xs font-bold text-slate-400 uppercase tracking-wider mb-1">最新版本</p>
          <h2 class="text-2xl font-black text-slate-800">{{ latestVersion || '加载中...' }}</h2>
        </div>
        <div class="w-12 h-12 rounded-xl bg-purple-50 text-purple-500 flex items-center justify-center">
          <Sparkles class="w-6 h-6" />
        </div>
      </div>
    </div>

    <!-- Versions List -->
    <div class="bg-white rounded-2xl shadow-sm border border-slate-100 overflow-hidden">
      <div class="p-5 border-b border-slate-100 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <h3 class="font-bold text-slate-800 flex items-center gap-2">
            <History class="w-5 h-5 text-slate-400" />
            版本列表
          </h3>
          <span v-if="lastFetchTimeDisplay" class="text-[10px] text-slate-400 bg-slate-50 px-2 py-0.5 rounded-full border border-slate-100 flex items-center gap-1">
            <Clock class="w-3 h-3" />
            上次同步: {{ lastFetchTimeDisplay }}
          </span>
        </div>
        <button @click="refresh(true)" class="text-slate-400 hover:text-slate-600 transition-colors p-2 rounded-lg hover:bg-slate-50" title="强制刷新版本列表">
          <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': loading }" />
        </button>
      </div>
      
      <div v-if="loading && releases.length === 0" class="p-10 text-center text-slate-400">
        <Loader2 class="w-8 h-8 animate-spin mx-auto mb-2" />
        <p>正在获取版本信息...</p>
      </div>

      <div v-else class="divide-y divide-slate-50">
        <div v-for="release in releases" :key="release.id" class="p-5 hover:bg-slate-50/50 transition-colors group">
          <div class="flex items-start justify-between gap-4">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <h4 class="font-bold text-slate-800">{{ release.name || release.tag_name }}</h4>
                <span v-if="release.tag_name === latestVersion" class="px-2 py-0.5 rounded-full bg-purple-100 text-purple-600 text-[10px] font-bold uppercase tracking-wide">Latest</span>
                <span v-if="isInstalled(release.tag_name)" class="px-2 py-0.5 rounded-full bg-emerald-100 text-emerald-600 text-[10px] font-bold uppercase tracking-wide">Installed</span>
                <span v-if="currentVersion === release.tag_name" class="px-2 py-0.5 rounded-full bg-blue-100 text-blue-600 text-[10px] font-bold uppercase tracking-wide">Current</span>
              </div>
              
              <div class="flex items-center gap-4 text-xs text-slate-400 mb-3">
                <span class="flex items-center gap-1">
                   <Calendar class="w-3 h-3" />
                   发布于 {{ formatDate(release.published_at) }}
                </span>
                <span class="flex items-center gap-1">
                   <Clock class="w-3 h-3" />
                   创建于 {{ formatDate(release.created_at) }}
                </span>
              </div>

              <!-- Body with hover expansion -->
              <div class="text-sm text-slate-600 relative group/body cursor-default">
                 <div class="line-clamp-2 whitespace-pre-wrap group-hover/body:line-clamp-none transition-all duration-300">
                    {{ release.body }}
                 </div>
                 <div class="absolute -bottom-4 left-0 w-full h-4 bg-gradient-to-t from-white to-transparent group-hover/body:hidden"></div>
              </div>
            </div>

            <div class="flex flex-col gap-2 shrink-0 pt-1">
               <button 
                 v-if="isInstalled(release.tag_name) && !hasDependencies(release.tag_name)"
                 @click="handleInstallDependencies(release.tag_name)"
                 class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 w-32 justify-center bg-amber-500 text-white hover:bg-amber-600 shadow-sm"
               >
                 <Download class="w-4 h-4" />
                 安装依赖
               </button>

               <button 
                 v-else-if="isInstalled(release.tag_name)"
                 @click="handleSwitch(release.tag_name)"
                 :disabled="currentVersion === release.tag_name || switchingVersion !== null || deletingVersions.size > 0"
                 class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 w-32 justify-center"
                 :class="(currentVersion === release.tag_name || switchingVersion !== null || deletingVersions.size > 0)
                    ? 'bg-slate-100 text-slate-400 cursor-not-allowed' 
                    : 'bg-white border border-slate-200 text-slate-600 hover:bg-slate-50 hover:text-slate-900 shadow-sm'"
               >
                 <Power class="w-4 h-4" :class="{ 'animate-spin': switchingVersion === release.tag_name }" />
                 {{ currentVersion === release.tag_name ? '当前使用' : (switchingVersion === release.tag_name ? '切换中...' : '切换版本') }}
               </button>

               <button 
                 v-if="isInstalled(release.tag_name) && currentVersion !== release.tag_name"
                 @click="handleDelete(release.tag_name)"
                 :disabled="deletingVersions.has(release.tag_name) || switchingVersion !== null"
                 class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 w-32 justify-center bg-white border shadow-sm"
                 :class="(deletingVersions.has(release.tag_name) || switchingVersion !== null)
                    ? 'border-slate-100 text-slate-400 cursor-not-allowed'
                    : 'border-red-100 text-red-500 hover:bg-red-50 hover:text-red-600'"
               >
                 <Loader2 v-if="deletingVersions.has(release.tag_name)" class="w-4 h-4 animate-spin" />
                 <Trash2 v-else class="w-4 h-4" />
                 {{ deletingVersions.has(release.tag_name) ? '删除中...' : '删除版本' }}
               </button>

               <button 
                 v-else-if="!isInstalled(release.tag_name)"
                 @click="handleInstall(release)"
                 class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 w-32 justify-center bg-slate-900 text-white hover:bg-slate-800 shadow-lg shadow-slate-200 active:scale-95"
               >
                 <Download class="w-4 h-4" />
                 下载安装
               </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { 
    CheckCircle2, Sparkles, History, RefreshCw, Loader2, 
    Calendar, Clock, Download, Power, Trash2 
} from 'lucide-vue-next';
import { installState } from '../lib/useInstall';
import { Dialog } from '../lib/useDialog';
import { toast } from 'vue-sonner';

interface ReleaseAsset {
    name: string;
    browser_download_url: string;
}

interface Release {
    id: number;
    tag_name: string;
    name: string;
    body: string;
    created_at: string;
    published_at: string;
    zipball_url: string;
    assets: ReleaseAsset[];
}

interface InstalledVersionInfo {
    version: string;
    hasNodeModules: boolean;
}

const releases = ref<Release[]>([]);
const installedVersions = ref<InstalledVersionInfo[]>([]);
const currentVersion = ref('');
const loading = ref(false);
const switchingVersion = ref<string | null>(null);
const deletingVersions = ref<Set<string>>(new Set());
const lastFetchTimeDisplay = ref('');

const updateLastFetchTimeDisplay = () => {
    const lastFetch = localStorage.getItem('sillytavern_releases_last_fetch');
    if (lastFetch) {
        lastFetchTimeDisplay.value = formatDate(new Date(Number(lastFetch)).toISOString());
    }
};

const latestVersion = computed(() => {
    if (releases.value.length > 0) {
        return releases.value[0].tag_name;
    }
    return '';
});

const isInstalled = (tagName: string) => {
    return installedVersions.value.some(v => v.version === tagName);
};

const hasDependencies = (tagName: string) => {
    const v = installedVersions.value.find(v => v.version === tagName);
    return v ? v.hasNodeModules : false;
};

const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit'
    });
};

const refresh = async (forceUpdate = false) => {
    // 尝试从缓存中加载版本列表，实现秒开
    const cachedReleases = localStorage.getItem('sillytavern_releases_cache');
    const lastFetchTime = localStorage.getItem('sillytavern_releases_last_fetch');
    const ONE_WEEK_MS = 7 * 24 * 60 * 60 * 1000;
    const now = Date.now();
    
    let shouldUseCache = false;

    if (cachedReleases) {
        try {
            const parsed = JSON.parse(cachedReleases);
            if (Array.isArray(parsed) && parsed.length > 0) {
                releases.value = parsed;
                
                // 如果不是强制刷新，并且距离上次获取还没超过一周，则不再请求接口
                if (!forceUpdate && lastFetchTime && (now - Number(lastFetchTime) < ONE_WEEK_MS)) {
                    shouldUseCache = true;
                }
            }
        } catch (e) {
            console.error('缓存解析失败:', e);
        }
    }

    // 更新本地安装和配置状态（不需要等太久，每次进入都可以更新）
    try {
        const [installed] = await Promise.all([
            invoke('get_installed_versions_info')
        ]);
        
        installedVersions.value = installed as InstalledVersionInfo[];
        
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
        currentVersion.value = versionFromConfig;

        // 如果可以使用缓存并且不是强制刷新，则结束
        if (shouldUseCache) {
            return;
        }

        // 否则后台静默请求最新版本列表
        loading.value = true;
        const fetchedReleases = await invoke<Release[]>('fetch_sillytavern_releases');
        const fetchedString = JSON.stringify(fetchedReleases);
        
        // 只有当接口返回的数据与缓存不同，才更新列表和缓存
        if (cachedReleases !== fetchedString) {
            releases.value = fetchedReleases;
            localStorage.setItem('sillytavern_releases_cache', fetchedString);
        }
        // 更新最后获取时间
        localStorage.setItem('sillytavern_releases_last_fetch', now.toString());
        updateLastFetchTimeDisplay();
        
    } catch (e) {
        console.error(e);
        toast.error('获取数据失败: ' + String(e));
    } finally {
        loading.value = false;
    }
};

const handleSwitch = async (version: string) => {
    if (switchingVersion.value || deletingVersions.value.size > 0) return;
    switchingVersion.value = version;
    try {
        await invoke('switch_sillytavern_version', { version });
        currentVersion.value = version;
        
        // 更新全局缓存中的版本号，确保其他页面（如酒馆配置、扩展管理）能立即响应版本切换
        const cachedConfigStr = localStorage.getItem('app_settings_config_cache');
        if (cachedConfigStr) {
            try {
                const parsedConfig = JSON.parse(cachedConfigStr);
                if (!parsedConfig.sillytavern) {
                    parsedConfig.sillytavern = {};
                }
                parsedConfig.sillytavern.version = version;
                localStorage.setItem('app_settings_config_cache', JSON.stringify(parsedConfig));
            } catch (e) {
                console.error('更新本地缓存版本号失败:', e);
            }
        }
        
        toast.success(`已切换到版本 ${version}`);
    } catch (e) {
        toast.error('切换版本失败: ' + String(e));
    } finally {
        switchingVersion.value = null;
    }
};

const handleDelete = (version: string) => {
    Dialog.warning({
        title: '删除确认',
        msg: `确定要删除版本 ${version} 吗？此操作不可恢复。`,
        confirmText: '删除',
        cancelText: '取消',
        onConfirm: async () => {
            installState.show = true;
            installState.version = version;
            installState.status = 'deleting';
            installState.operation = 'delete';
            installState.progress = 0;
            installState.logs = [`开始删除版本 ${version}...`];

            deletingVersions.value.add(version);
            try {
                await invoke('delete_sillytavern_version', { version });
                installState.logs.push(`版本 ${version} 删除成功`);
                installState.status = 'done';
                installState.progress = 1;
                
                // Refresh list
                await refresh();
            } catch (e) {
                installState.status = 'error';
                installState.logs.push(`删除失败: ${String(e)}`);
            } finally {
                deletingVersions.value.delete(version);
                Dialog.close();
            }
        }
    });
};

const handleInstallDependencies = async (version: string) => {
    installState.show = true;
    installState.version = version;
    installState.status = 'downloading';
    installState.operation = 'install';
    installState.progress = 0;
    installState.logs = [`开始为版本 ${version} 安装依赖...`];

    try {
        await invoke('install_sillytavern_dependencies', { version });
        // 同理，这里的依赖安装也是后台异步执行，实际完成会通过 installState.status === 'done' 触发 watcher
    } catch (e) {
        installState.status = 'error';
        installState.logs.push(`错误: ${String(e)}`);
    }
};

const handleInstall = async (release: Release) => {
    installState.show = true;
    installState.version = release.tag_name;
    installState.status = 'downloading';
    installState.operation = 'install';
    installState.progress = 0;
    installState.logs = [`开始安装版本 ${release.tag_name}...`];

    // Get config to check for proxy
    let downloadUrl = `https://github.com/SillyTavern/SillyTavern/archive/refs/tags/${release.tag_name}.zip`;
    try {
        const config: any = await invoke('get_app_config');
        if (config.githubProxy && config.githubProxy.enable && config.githubProxy.url) {
            // Ensure proxy url ends with slash or handle it
            let proxyUrl = config.githubProxy.url;
            if (!proxyUrl.endsWith('/')) {
                proxyUrl += '/';
            }
            downloadUrl = `${proxyUrl}${downloadUrl}`;
            installState.logs.push(`使用加速代理: ${proxyUrl}`);
        }
    } catch (e) {
        console.error('Failed to get config for proxy:', e);
    }

    try {
        await invoke('install_sillytavern_version', { 
            version: release.tag_name,
            url: downloadUrl 
        });
        
        // 我们之前把这里获取安装列表的逻辑提前了，但由于后端是异步 spawn 执行 npm install，
        // 此时 node_modules 还没生成，所以获取到的 has_node_modules 会是 false。
        // 现在我们在下方添加了 watch 监听 installState.status === 'done' 来自动刷新，所以这里不需要获取。
        
    } catch (e) {
        installState.status = 'error';
        installState.logs.push(`错误: ${String(e)}`);
    }
};

onMounted(() => {
    updateLastFetchTimeDisplay();
    refresh();
});

// 监听安装状态，如果安装成功则自动刷新列表，更新依赖状态
watch(() => installState.status, (newStatus) => {
    if (newStatus === 'done') {
        // 当状态变为 done 时，说明 npm install 或者解压等已经彻底完成
        // 此时 node_modules 已经存在，我们重新获取列表以更新 hasNodeModules
        invoke('get_installed_versions_info').then(installed => {
            installedVersions.value = installed as InstalledVersionInfo[];
        }).catch(e => {
            console.error('Failed to update installed versions after install done:', e);
        });
    }
});
</script>

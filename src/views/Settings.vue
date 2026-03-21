<script lang="ts">
// Global promise to prevent concurrent loads across component instances
let globalLoadConfigPromise: Promise<void> | null = null;
</script>

<script setup lang="ts">
import { ref, onMounted, watch, computed, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { openUrl as open } from '@tauri-apps/plugin-opener';
import { toast } from 'vue-sonner';
import { PhCheck, PhArrowsClockwise, PhGlobe, PhPalette, PhGithubLogo, PhInfo, PhPackage, PhDownloadSimple } from '@phosphor-icons/vue';
import globalConfig from '../lib/config'
import { checkUpdate } from '../lib/updater'
import { setTheme } from '../lib/theme'
import { getSystemLocale } from '../lang'

const { t, locale } = useI18n();



interface GithubProxyConfig {
  enable: boolean;
  url: String;
}

interface NodeInfo {
  version: string | null;
  path: string | null;
  source: 'system' | 'local' | 'none';
}

interface NpmInfo {
  version: string | null;
  path: string | null;
  source: 'system' | 'local' | 'none';
}

interface DownloadProgress {
  status: string;
  progress: number;
  log: string;
}

interface AppConfig {
  lang: string;
  theme: 'light' | 'dark' | 'auto';
  rememberWindowPosition: boolean;
  githubProxy: GithubProxyConfig;
  npmRegistry: string;
}

interface ProxyItem {
  url: string;
  latency: number;
}

interface NpmRegistry {
  name: string;
  url: string;
}

const activeTab = ref<'general' | 'about'>('general');
const loading = ref(false);
const proxyLoading = ref(false);
const proxyLastFetchTimeDisplay = ref('');
let isSyncing = false;
const checkingUpdate = ref(false);

const handleCheckUpdate = async () => {
  if (checkingUpdate.value) return;
  checkingUpdate.value = true;
  await checkUpdate(true);
  checkingUpdate.value = false;
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

const npmRegistries: NpmRegistry[] = [
  { name: t('settings.npmRegistryOfficial'), url: 'https://registry.npmjs.org/' },
  { name: t('settings.npmRegistryTaobao'), url: 'https://registry.npmmirror.com/' },
  { name: t('settings.npmRegistryTencent'), url: 'https://mirrors.cloud.tencent.com/npm/' },
  { name: t('settings.npmRegistryHuawei'), url: 'https://repo.huaweicloud.com/repository/npm/' }
];

const config = ref<AppConfig>({
  lang: 'auto',
  theme: 'auto',
  rememberWindowPosition: false,
  githubProxy: {
    enable: false,
    url: 'https://ghfast.top/'
  },
  npmRegistry: 'https://registry.npmmirror.com/'
});

const proxies = ref<ProxyItem[]>([]);
const nodeInfo = ref<NodeInfo>({ version: null, path: null, source: 'none' });
const npmInfo = ref<NpmInfo>({ version: null, path: null, source: 'none' });
const installingNode = ref(false);
const nodeProgress = ref<DownloadProgress>({ status: '', progress: 0, log: '' });

const isNodeVersionValid = computed(() => {
  if (!nodeInfo.value.version) return false;
  // Version string usually looks like "v18.20.4"
  const match = nodeInfo.value.version.match(/v?(\d+)\./);
  if (match && match[1]) {
    const majorVersion = parseInt(match[1], 10);
    return majorVersion >= 18;
  }
  return false;
});

const loadConfig = async () => {
  if (globalLoadConfigPromise) return globalLoadConfigPromise;

  globalLoadConfigPromise = (async () => {
    try {
      // 先尝试从缓存中读取，实现秒开
      const cachedConfig = localStorage.getItem('app_settings_config_cache');
      if (cachedConfig) {
        try {
          config.value = JSON.parse(cachedConfig);
          loading.value = false;
        } catch (e) {
          console.error('缓存解析失败:', e);
          loading.value = true;
        }
      } else {
        loading.value = true;
      }

      // 后台静默获取最新配置
      const res = await invoke<AppConfig>('get_app_config');

      // 检查后端数据与当前状态是否不一致（只对比当前配置的关键字段）
      let isDifferent = false;
      if (!cachedConfig) {
        isDifferent = true;
      } else {
        for (const key in res) {
          if (JSON.stringify(res[key as keyof AppConfig]) !== JSON.stringify(config.value[key as keyof AppConfig])) {
            isDifferent = true;
            break;
          }
        }
      }

      if (isDifferent) {
        // 避免触发 watch 的自动保存
        isSyncing = true;
        config.value = { ...config.value, ...res };

        // 立即应用主题和语言设置
        setTheme(res.theme);
        if (res.lang === 'auto') {
          locale.value = getSystemLocale();
        } else {
          locale.value = res.lang;
        }

        // 更新缓存并保留其他模块追加的数据
        const currentCachedStr = localStorage.getItem('app_settings_config_cache');
        let mergedConfig = { ...res };
        if (currentCachedStr) {
          try {
            const cached = JSON.parse(currentCachedStr);
            mergedConfig = { ...cached, ...res };
          } catch (e) { }
        }
        localStorage.setItem('app_settings_config_cache', JSON.stringify(mergedConfig));
      } else {
        // 即使配置没有变化,也要确保主题和语言正确应用
        setTheme(config.value.theme);
        if (config.value.lang === 'auto') {
          locale.value = getSystemLocale();
        } else {
          locale.value = config.value.lang;
        }
      }

      // 等待 DOM 更新后解除状态
      await nextTick();
      isSyncing = false;
    } catch (error) {
      console.error('Failed to load config:', error);
      toast.error(t('settings.loadFailed'));
    } finally {
      loading.value = false;
      globalLoadConfigPromise = null;
    }
  })();

  return globalLoadConfigPromise;
};

const saveConfig = async () => {
  try {
    await invoke('save_app_config', { config: config.value });

    // 保存成功后同时更新本地缓存，合并现有数据以免覆盖其他模块追加的数据(如 sillytavern.version)
    const cachedStr = localStorage.getItem('app_settings_config_cache');
    let mergedConfig = { ...config.value };
    if (cachedStr) {
      try {
        const cached = JSON.parse(cachedStr);
        mergedConfig = { ...cached, ...config.value };
      } catch (e) { }
    }
    localStorage.setItem('app_settings_config_cache', JSON.stringify(mergedConfig));

    // toast.success('设置已保存'); // Remove toast for real-time save to avoid spam
    console.log('Config saved');
  } catch (error) {
    console.error('Failed to save config:', error);
    toast.error(t('settings.saveFailed'));
  }
};

const fetchProxies = async (forceUpdate = false) => {
  const THREE_DAYS_MS = 3 * 24 * 60 * 60 * 1000;
  const now = Date.now();
  const cachedProxies = localStorage.getItem('app_settings_proxies_cache');
  const lastFetchTime = localStorage.getItem('app_settings_proxies_last_fetch');

  if (cachedProxies) {
    try {
      const parsed = JSON.parse(cachedProxies);
      if (Array.isArray(parsed) && parsed.length > 0) {
        proxies.value = parsed;
        if (lastFetchTime) {
          proxyLastFetchTimeDisplay.value = formatDate(new Date(Number(lastFetchTime)).toISOString());
        }

        // 如果不是强制刷新，并且距离上次获取还没超过3天，则不再请求接口
        if (!forceUpdate && lastFetchTime && (now - Number(lastFetchTime) < THREE_DAYS_MS)) {
          return;
        }
      }
    } catch (e) {
      console.error('加速节点缓存解析失败:', e);
    }
  }

  try {
    proxyLoading.value = true;
    const res = await invoke<ProxyItem[]>('fetch_github_proxies');
    const sortedProxies = res.sort((a, b) => a.latency - b.latency);

    const fetchedString = JSON.stringify(sortedProxies);
    if (cachedProxies !== fetchedString) {
      proxies.value = sortedProxies;
      localStorage.setItem('app_settings_proxies_cache', fetchedString);
    }

    localStorage.setItem('app_settings_proxies_last_fetch', now.toString());
    proxyLastFetchTimeDisplay.value = formatDate(new Date(now).toISOString());
    // toast.success('获取加速列表成功'); // Remove toast on auto-fetch
  } catch (error) {
    console.error('Failed to fetch proxies:', error);
    toast.error(t('settings.saveFailed'));
  } finally {
    proxyLoading.value = false;
  }
};

const selectProxy = (url: string) => {
  config.value.githubProxy.url = url;
  // watch will handle saving
};

const checkNode = async () => {
  try {
    // 优先从缓存读取
    const cachedNode = localStorage.getItem('app_settings_node_cache');
    if (cachedNode) {
      try {
        nodeInfo.value = JSON.parse(cachedNode);
      } catch (e) { }
    }

    const res = await invoke<NodeInfo>('check_nodejs');

    // 如果与缓存不一致，则更新缓存和状态
    if (JSON.stringify(res) !== JSON.stringify(nodeInfo.value)) {
      nodeInfo.value = res;
      localStorage.setItem('app_settings_node_cache', JSON.stringify(res));
    }
  } catch (error) {
    console.error('Failed to check nodejs:', error);
  }
};

const checkNpm = async () => {
  try {
    // 优先从缓存读取
    const cachedNpm = localStorage.getItem('app_settings_npm_cache');
    if (cachedNpm) {
      try {
        npmInfo.value = JSON.parse(cachedNpm);
      } catch (e) { }
    }

    const res = await invoke<NpmInfo>('check_npm');

    // 如果与缓存不一致，则更新缓存和状态
    if (JSON.stringify(res) !== JSON.stringify(npmInfo.value)) {
      npmInfo.value = res;
      localStorage.setItem('app_settings_npm_cache', JSON.stringify(res));
    }
  } catch (error) {
    console.error('Failed to check npm:', error);
  }
};

const installNode = async () => {
  if (installingNode.value) return;
  installingNode.value = true;
  nodeProgress.value = { status: 'starting', progress: 0, log: t('common.processing') };

  try {
    await invoke('install_nodejs');
    toast.success(t('settings.nodejsInstall') + ' ' + t('common.success'));
    await checkNode();
    await checkNpm();
  } catch (error) {
    console.error('Failed to install nodejs:', error);
    toast.error(t('common.failed') + ': ' + error);
  } finally {
    installingNode.value = false;
  }
};

// Watch for config changes and save automatically
watch(config, () => {
  if (!loading.value && !isSyncing) {
    saveConfig();
  }
}, { deep: true });

// 监听语言变化
watch(() => config.value.lang, (newLang) => {
  if (newLang === 'auto') {
    locale.value = getSystemLocale()
  } else {
    locale.value = newLang
  }
})

// 监听主题变化
watch(() => config.value.theme, (newTheme) => {
  setTheme(newTheme)
})

onMounted(async () => {
  await loadConfig();
  fetchProxies();
  checkNode();
  checkNpm();

  await listen<DownloadProgress>('download-progress', (event) => {
    if (installingNode.value) {
      nodeProgress.value = event.payload;
    }
  });
});

const openLink = (url: string) => {
  open(url).catch((err: any) => {
    console.error('Failed to open URL:', err);
    toast.error(t('settings.loadFailed'));
  });
};
</script>

<template>
  <div class="flex flex-col h-full">
    <h1 class="text-2xl font-bold mb-6 px-1">{{ t('settings.title') }}</h1>

    <!-- Tabs -->
    <div class="flex space-x-1 bg-slate-100 dark:bg-slate-800 p-1 rounded-xl w-fit mb-6 shrink-0">
      <button @click="activeTab = 'general'" :class="[
        'px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 flex items-center gap-2',
        activeTab === 'general'
          ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-sm'
          : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-700/50'
      ]">
        <PhPalette :size="16" weight="duotone" />
        {{ t('settings.general') }}
      </button>
      <button @click="activeTab = 'about'" :class="[
        'px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 flex items-center gap-2',
        activeTab === 'about'
          ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-sm'
          : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-700/50'
      ]">
        <PhInfo :size="16" weight="duotone" />
        {{ t('settings.about') }}
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto px-1 pb-10 min-h-0 relative">

      <!-- Loading State -->
      <div v-if="loading"
        class="absolute inset-0 flex flex-col items-center justify-center bg-slate-50/50 dark:bg-slate-900/50 backdrop-blur-sm z-10">
        <PhArrowsClockwise :size="48" class="animate-spin mb-4 text-blue-500/80" weight="duotone" />
        <p class="text-sm font-medium text-slate-500 dark:text-slate-400 animate-pulse">{{ t('settings.loadingConfig') }}</p>
      </div>

      <!-- General Settings -->
      <div v-if="activeTab === 'general'" class="space-y-8 animate-in fade-in slide-in-from-bottom-2 duration-300">

        <!-- Interface Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2">
            <PhPalette :size="20" class="text-blue-500" weight="duotone" />
            {{ t('settings.interface') }}
          </h2>

          <div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm">
            <!-- Language -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-blue-50 dark:bg-blue-900/30 flex items-center justify-center text-blue-500">
                  <PhGlobe :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('settings.language') }}</div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.languageDesc') }}</div>
                </div>
              </div>
              <select v-model="config.lang"
                class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 min-w-[140px] outline-none transition-all">
                <option value="zh-CN">{{ t('settings.languageZhCN') }}</option>
                <option value="en-US">{{ t('settings.languageEnUS') }}</option>
              </select>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <!-- Theme -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-purple-50 dark:bg-purple-900/30 flex items-center justify-center text-purple-500">
                  <PhPalette :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('settings.theme') }}</div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.themeDesc') }}</div>
                </div>
              </div>
              <select v-model="config.theme"
                class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 min-w-[140px] outline-none transition-all">
                <option value="auto">{{ t('settings.themeAuto') }}</option>
                <option value="light">{{ t('settings.themeLight') }}</option>
                <option value="dark">{{ t('settings.themeDark') }}</option>
              </select>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-indigo-50 dark:bg-indigo-900/30 flex items-center justify-center text-indigo-500">
                  <PhPalette :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('settings.rememberWindow') }}</div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.rememberWindowDesc') }}</div>
                </div>
              </div>
              <label class="inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="config.rememberWindowPosition" class="sr-only peer">
                <div
                  class="relative w-11 h-6 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600">
                </div>
              </label>
            </div>
          </div>
        </section>

        <!-- NodeJs Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2">
            <PhPackage :size="20" class="text-green-600" weight="duotone" />
            {{ t('settings.nodejs') }}
          </h2>

          <div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-green-50 dark:bg-green-900/30 flex items-center justify-center text-green-600">
                  <PhPackage :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('settings.nodejsEnv') }}</div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    <span v-if="nodeInfo.version">
                      {{ t('settings.nodejsVersion') }}: {{ nodeInfo.version }} ({{ nodeInfo.source === 'local' ? t('settings.nodejsLocal') : t('settings.nodejsSystem') }})
                      <div v-if="isNodeVersionValid && nodeInfo.path" class="mt-1 text-slate-400 dark:text-slate-500 break-all select-all">
                        {{ t('settings.nodejsPath') }}: {{ nodeInfo.path }}
                      </div>
                      <div v-if="!isNodeVersionValid" class="mt-1 text-red-500">
                        {{ t('settings.nodejsLowVersion') }}
                      </div>
                    </span>
                    <span v-else>{{ t('settings.nodejsNotFound') }}</span>
                  </div>
                </div>
              </div>

              <div v-if="!isNodeVersionValid || nodeInfo.source === 'local'" class="flex items-center gap-2">
                <button @click="installNode" :disabled="installingNode"
                  class="px-3 py-1.5 text-xs font-medium bg-green-50 dark:bg-green-900/30 text-green-600 rounded-md hover:bg-green-100 dark:hover:bg-green-900/50 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1">
                  <PhArrowsClockwise v-if="installingNode" :size="14" class="animate-spin" />
                  <PhDownloadSimple v-else :size="14" />
                  {{ installingNode ? t('settings.nodejsInstalling') : (nodeInfo.version ? t('settings.nodejsReinstall') : t('settings.nodejsInstall')) }}
                </button>
              </div>
            </div>

            <div v-if="installingNode" class="space-y-2 pt-2 border-t border-slate-100 dark:border-slate-700">
              <div class="flex justify-between text-xs text-slate-500 dark:text-slate-400">
                <span>{{ nodeProgress.log }}</span>
                <span>{{ Math.round(nodeProgress.progress * 100) }}%</span>
              </div>
              <div class="w-full bg-slate-100 dark:bg-slate-700 rounded-full h-1.5 overflow-hidden">
                <div class="bg-green-500 h-1.5 rounded-full transition-all duration-300"
                  :style="{ width: `${nodeProgress.progress * 100}%` }"></div>
              </div>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <!-- NPM Info -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-red-50 dark:bg-red-900/30 flex items-center justify-center text-red-500">
                  <PhPackage :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('settings.npmEnv') }}</div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">
                    <span v-if="npmInfo.version">
                      {{ t('settings.npmVersion') }}: {{ npmInfo.version }} ({{ npmInfo.source === 'local' ? t('settings.nodejsLocal') : t('settings.nodejsSystem') }})
                      <div v-if="npmInfo.path" class="mt-1 text-slate-400 dark:text-slate-500 break-all select-all">
                        {{ t('settings.nodejsPath') }}: {{ npmInfo.path }}
                      </div>
                    </span>
                    <span v-else>{{ t('settings.npmNotFound') }}</span>
                  </div>
                </div>
              </div>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <!-- NPM Registry Selection -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-orange-50 dark:bg-orange-900/30 flex items-center justify-center text-orange-500">
                  <PhGlobe :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('settings.npmRegistry') }}</div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.npmRegistryDesc') }}</div>
                </div>
              </div>
              <select v-model="config.npmRegistry" @change="saveConfig"
                class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-xs rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-48 p-2">
                <option v-for="registry in npmRegistries" :key="registry.url" :value="registry.url">
                  {{ registry.name }}
                </option>
              </select>
            </div>
            <div class="text-[10px] text-slate-400 pl-11">
              {{ t('settings.currentAddress') }}: {{ config.npmRegistry }}
            </div>
          </div>
        </section>

        <!-- Github Proxy Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2">
            <PhGithubLogo :size="20" class="text-slate-700 dark:text-slate-400" weight="duotone" />
            {{ t('settings.github') }}
          </h2>

          <div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm">
            <!-- Toggle -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-slate-100 dark:bg-slate-700 flex items-center justify-center text-slate-600 dark:text-slate-400">
                  <PhGithubLogo :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('settings.githubToggle') }}</div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('settings.githubToggleDesc') }}</div>
                </div>
              </div>
              <label class="inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="config.githubProxy.enable" class="sr-only peer">
                <div
                  class="relative w-11 h-6 bg-slate-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600">
                </div>
              </label>
            </div>

            <!-- Current URL Display -->
            <div class="flex items-center gap-3 p-3 bg-slate-50 dark:bg-slate-900/50 rounded-lg border border-slate-100 dark:border-slate-700">
              <div class="text-sm font-medium text-slate-500 dark:text-slate-400 whitespace-nowrap">{{ t('settings.currentAddress') }}:</div>
              <div class="text-sm text-slate-800 dark:text-slate-300 font-mono truncate select-all">{{ config.githubProxy.url }}</div>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <!-- Proxy List Header -->
            <div class="flex items-center justify-between pt-2">
              <div class="flex items-center gap-3">
                <h3 class="text-sm font-medium text-slate-700 dark:text-slate-300">{{ t('settings.githubProxyList') }}</h3>
                <span v-if="proxyLastFetchTimeDisplay"
                  class="text-[10px] text-slate-400 bg-slate-50 dark:bg-slate-900/50 px-2 py-0.5 rounded-full border border-slate-100 dark:border-slate-700 flex items-center gap-1">
                  {{ t('settings.githubLastSync') }}: {{ proxyLastFetchTimeDisplay }}
                </span>
              </div>
              <button @click="fetchProxies(true)" :disabled="proxyLoading"
                class="text-xs flex items-center gap-1.5 px-3 py-1.5 bg-blue-50 dark:bg-blue-900/30 text-blue-600 rounded-md hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                <PhArrowsClockwise :class="{ 'animate-spin': proxyLoading }" :size="14" />
                {{ proxyLoading ? t('settings.githubRefreshing') : t('settings.githubRefresh') }}
              </button>
            </div>

            <!-- Proxy List -->
            <div v-if="proxies.length > 0" class="space-y-2 max-h-60 overflow-y-auto custom-scrollbar pr-1">
              <div v-for="proxy in proxies" :key="proxy.url" @click="selectProxy(proxy.url)" :class="[
                'flex items-center justify-between p-3 rounded-lg border cursor-pointer transition-all hover:shadow-sm',
                config.githubProxy.url === proxy.url
                  ? 'bg-blue-50 dark:bg-blue-900/30 border-blue-200 dark:border-blue-800 ring-1 ring-blue-200 dark:ring-blue-800'
                  : 'bg-white dark:bg-slate-900/50 border-slate-100 dark:border-slate-700 hover:border-slate-300 dark:hover:border-slate-600'
              ]">
                <div class="flex items-center gap-3 overflow-hidden">
                  <div :class="[
                    'w-4 h-4 rounded-full flex items-center justify-center shrink-0',
                    config.githubProxy.url === proxy.url ? 'text-blue-600' : 'text-transparent'
                  ]">
                    <PhCheck :size="12" weight="bold" />
                  </div>
                  <div class="text-sm font-mono truncate text-slate-600 dark:text-slate-400">{{ proxy.url }}</div>
                </div>
                <div class="flex items-center gap-2 shrink-0">
                  <span :class="[
                    'text-xs font-medium px-2 py-0.5 rounded',
                    proxy.latency < 200 ? 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400' :
                      proxy.latency < 500 ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400' :
                        'bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400'
                  ]">
                    {{ proxy.latency }}ms
                  </span>
                </div>
              </div>
            </div>

            <div v-else-if="!proxyLoading"
              class="text-center py-8 text-slate-400 text-sm bg-slate-50 dark:bg-slate-900/50 rounded-lg border border-dashed border-slate-200 dark:border-slate-700">
              {{ t('settings.githubEmpty') }}
            </div>

            <div v-else class="py-8 flex justify-center">
              <div class="animate-pulse flex space-x-4 w-full px-4">
                <div class="flex-1 space-y-3 py-1">
                  <div class="h-10 bg-slate-100 dark:bg-slate-700 rounded"></div>
                  <div class="h-10 bg-slate-100 dark:bg-slate-700 rounded"></div>
                  <div class="h-10 bg-slate-100 dark:bg-slate-700 rounded"></div>
                </div>
              </div>
            </div>

          </div>
        </section>

      </div>

      <!-- About Settings -->
      <div v-if="activeTab === 'about'" class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
        <div
          class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-8 shadow-sm flex flex-col items-center text-center space-y-4">
          <div class="w-20 h-20 bg-slate-100 dark:bg-slate-700 rounded-2xl flex items-center justify-center mb-2">
            <img :src="globalConfig.appIcon" alt="Logo" class="w-12 h-12 opacity-80" />
          </div>
          <div>
            <h2 class="text-xl font-bold text-slate-800 dark:text-slate-200">{{ globalConfig.appName }}({{ globalConfig.appNameEn }})</h2>
            <p class="text-slate-500 dark:text-slate-400 text-sm mt-1">{{ t('settings.version') }} {{ globalConfig.appVersion }}</p>
          </div>
          <button @click="handleCheckUpdate" :disabled="checkingUpdate"
            class="mt-2 px-4 py-2 bg-blue-50 dark:bg-blue-900/30 text-blue-600 rounded-lg text-sm font-medium hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-colors flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed">
            <PhArrowsClockwise :class="{ 'animate-spin': checkingUpdate }" :size="16" weight="bold" />
            {{ checkingUpdate ? t('settings.checking') : t('settings.checkUpdate') }}
          </button>
          <p class="text-slate-600 dark:text-slate-400 max-w-md text-sm leading-relaxed mt-4">
            {{ (locale === 'zh-CN') ? globalConfig.appDescription : globalConfig.appDescriptionEn }}
          </p>

          <div class="max-w-md w-full mt-6 p-4 rounded-xl bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800/50 flex flex-col items-center gap-2">
            <div class="flex items-center gap-2 text-amber-800 dark:text-amber-400 font-bold text-sm">
              <PhInfo :size="18" weight="fill" />
              {{ locale === 'zh-CN' ? '重要提醒' : 'Important Notice' }}
            </div>
            <p class="text-xs text-amber-700 dark:text-amber-500/90 leading-relaxed text-center">
              {{ locale === 'zh-CN' 
                ? '本软件完全免费且开源。如果您是通过付费、打赏或在各平台（如某宝、某鱼等）购买到的，请立即申请退款并举报相应商家。' 
                : 'This software is completely free and open-source. If you paid for this or bought it from any marketplace, please request a refund immediately and report the seller.' 
              }}
            </p>
          </div>

          <div class="flex gap-4 mt-6 pt-6 border-t border-slate-100 dark:border-slate-700 w-full justify-center">
            <button @click="openLink(globalConfig.git.github)"
              class="flex items-center gap-1.5 text-sm font-medium text-slate-600 dark:text-slate-400 hover:text-blue-500 dark:hover:text-blue-400 transition-colors bg-slate-50 dark:bg-slate-700/50 px-3 py-1.5 rounded-lg border border-slate-100 dark:border-slate-700/50 cursor-pointer">
              <PhGithubLogo :size="18" weight="duotone" />
              GitHub
            </button>
            <button @click="openLink(globalConfig.git.gitee)"
              class="flex items-center gap-1.5 text-sm font-medium text-slate-600 dark:text-slate-400 hover:text-red-500 dark:hover:text-red-400 transition-colors bg-slate-50 dark:bg-slate-700/50 px-3 py-1.5 rounded-lg border border-slate-100 dark:border-slate-700/50 cursor-pointer">
              <PhGlobe :size="18" weight="duotone" />
              Gitee
            </button>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: #cbd5e1;
  border-radius: 20px;
}
</style>

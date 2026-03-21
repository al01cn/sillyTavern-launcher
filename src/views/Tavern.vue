<script setup lang="ts">
import { ref, onBeforeUnmount, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'vue-sonner';
import { PhFileCode, PhGlobe, PhLockKey, PhBrowser, PhListNumbers } from '@phosphor-icons/vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const loading = ref(true);
const version = ref<string>('');
const configError = ref<string | null>(null);
const saveInProgress = ref(false);
const saveQueued = ref(false);
let saveTimer: ReturnType<typeof setTimeout> | null = null;

type TavernConfigPayload = {
  port: number;
  listen: boolean;
  listenAddress: {
    ipv4: string;
    ipv6: string;
  };
  protocol: {
    ipv4: boolean;
    ipv6: boolean;
  };
  basicAuthMode: boolean;
  enableUserAccounts: boolean;
  enableDiscreetLogin: boolean;
  perUserBasicAuth: boolean;
  basicAuthUser: {
    username: string;
    password: string;
  };
  whitelistMode: boolean;
  whitelist: string[];
  cors: {
    enabled: boolean;
    origin: string[];
    methods: string[];
    allowedHeaders: string[];
    exposedHeaders: string[];
    credentials: boolean;
    maxAge: number | null;
  };
  requestProxy: {
    enabled: boolean;
    url: string;
    bypass: string[];
  };
  backups: {
    common: {
      numberOfBackups: number;
    };
    chat: {
      enabled: boolean;
      checkIntegrity: boolean;
      maxTotalBackups: number;
      throttleInterval: number;
    };
  };
  thumbnails: {
    enabled: boolean;
    format: string;
    quality: number;
    dimensions: {
      bg: number[];
      avatar: number[];
      persona: number[];
    };
  };
  browserLaunchEnabled: boolean;
  browserType: string;
};

const tavernConfig = ref<TavernConfigPayload>({
  port: 8000,
  listen: false,
  listenAddress: {
    ipv4: '0.0.0.0',
    ipv6: '[::]'
  },
  protocol: {
    ipv4: true,
    ipv6: false
  },
  basicAuthMode: false,
  enableUserAccounts: false,
  enableDiscreetLogin: false,
  perUserBasicAuth: false,
  basicAuthUser: {
    username: 'user',
    password: 'password'
  },
  whitelistMode: true,
  whitelist: [] as string[],
  cors: {
    enabled: true,
    origin: ['null'],
    methods: ['OPTIONS'],
    allowedHeaders: [],
    exposedHeaders: [],
    credentials: false,
    maxAge: null
  },
  requestProxy: {
    enabled: false,
    url: '',
    bypass: []
  },
  backups: {
    common: {
      numberOfBackups: 50
    },
    chat: {
      enabled: true,
      checkIntegrity: true,
      maxTotalBackups: -1,
      throttleInterval: 10000
    }
  },
  thumbnails: {
    enabled: true,
    format: 'jpg',
    quality: 95,
    dimensions: {
      bg: [160, 90],
      avatar: [96, 144],
      persona: [96, 144]
    }
  },
  browserLaunchEnabled: true,
  browserType: 'default'
});

const isInitialLoad = ref(true);

const loadConfig = async () => {
  try {
    // 尝试从缓存加载，实现秒开
    const cachedConfig = localStorage.getItem('tavern_config_data_cache');
    
    let hasCache = false;
    
    // 从 app_settings_config_cache 获取当前选择的版本号作为唯一真理
    let currentVersionName = '';
    const cachedAppConfig = localStorage.getItem('app_settings_config_cache');
    if (cachedAppConfig) {
      try {
        const parsed = JSON.parse(cachedAppConfig);
        if (parsed?.sillytavern?.version) {
          currentVersionName = parsed.sillytavern.version;
        }
      } catch (e) {}
    }
    
    if (!currentVersionName) {
      const appConfig = await invoke<any>('get_app_config');
      currentVersionName = appConfig.sillytavern?.version || '';
    }

    if (!currentVersionName) {
      configError.value = t('tavern.noVersionSelected');
      loading.value = false;
      return;
    }
    
    if (cachedConfig) {
      try {
        const parsedConfig = JSON.parse(cachedConfig);
        // 检查缓存的配置是否真的是当前选中版本的配置
        // 由于 tavern_config_data_cache 中本身并没有显式存储版本号，我们通过界面状态先假设它是
        version.value = currentVersionName;
        tavernConfig.value = parsedConfig;
        loading.value = false;
        isInitialLoad.value = false; // 有缓存则立即结束初次加载状态，允许用户立刻操作
        hasCache = true;
      } catch (e) {
        console.error('缓存解析失败:', e);
        loading.value = true;
        isInitialLoad.value = true;
      }
    } else {
      loading.value = true;
      isInitialLoad.value = true;
    }

    configError.value = null;

    const fetchedConfig = await invoke<TavernConfigPayload>('get_sillytavern_config_options', {
      version: currentVersionName
    });
    
    const fetchedConfigStr = JSON.stringify(fetchedConfig);
    
    // 如果接口数据与缓存不一致，则静默更新界面和缓存
    if (currentVersionName !== version.value || fetchedConfigStr !== cachedConfig) {
      // 临时屏蔽 watch，防止因应用后端数据而触发无意义的自动保存
      isInitialLoad.value = true;
      
      version.value = currentVersionName;
      tavernConfig.value = fetchedConfig;
      
      localStorage.setItem('tavern_config_data_cache', fetchedConfigStr);
      
      // 延迟恢复 watch，确保数据更新完成
      setTimeout(() => {
        isInitialLoad.value = false;
      }, 50);
    } else if (!hasCache) {
      // 如果没有缓存且数据一致（说明是首次加载完毕），也要解除初始状态
      isInitialLoad.value = false;
    }

  } catch (error: any) {
    console.error('Failed to load tavern config:', error);
    configError.value = typeof error === 'string' ? error : t('tavern.loadConfigFailed');
  } finally {
    loading.value = false;
  }
};

const saveConfig = async () => {
  if (!version.value || isInitialLoad.value || loading.value || configError.value) return;
  if (saveInProgress.value) {
    saveQueued.value = true;
    return;
  }

  saveInProgress.value = true;
  try {
    await invoke('update_sillytavern_config_options', {
      version: version.value,
      config: tavernConfig.value
    });
  } catch (error) {
    console.error('Failed to save tavern config:', error);
    toast.error(t('tavern.saveFailed'));
  } finally {
    saveInProgress.value = false;
    if (saveQueued.value) {
      saveQueued.value = false;
      await saveConfig();
    }
  }
};

const scheduleSave = () => {
  if (saveTimer) {
    clearTimeout(saveTimer);
  }
  saveTimer = setTimeout(() => {
    void saveConfig();
  }, 300);
};

const openConfigFile = async () => {
  if (!version.value) return;
  try {
    await invoke('open_sillytavern_config_file', { version: version.value });
  } catch (error) {
    console.error('Failed to open config file:', error);
    toast.error(t('tavern.cannotOpenFile'));
  }
};

const addWhitelistItem = () => {
  tavernConfig.value.whitelist.push('');
};

const removeWhitelistItem = (index: number) => {
  tavernConfig.value.whitelist.splice(index, 1);
};

const addCorsOrigin = () => {
  tavernConfig.value.cors.origin.push('');
};

const removeCorsOrigin = (index: number) => {
  tavernConfig.value.cors.origin.splice(index, 1);
};

const addCorsMethod = () => {
  tavernConfig.value.cors.methods.push('');
};

const removeCorsMethod = (index: number) => {
  tavernConfig.value.cors.methods.splice(index, 1);
};

const addCorsAllowedHeader = () => {
  tavernConfig.value.cors.allowedHeaders.push('');
};

const removeCorsAllowedHeader = (index: number) => {
  tavernConfig.value.cors.allowedHeaders.splice(index, 1);
};

const addCorsExposedHeader = () => {
  tavernConfig.value.cors.exposedHeaders.push('');
};

const removeCorsExposedHeader = (index: number) => {
  tavernConfig.value.cors.exposedHeaders.splice(index, 1);
};

const onCorsMaxAgeInput = (event: Event) => {
  const value = (event.target as HTMLInputElement).value.trim();
  if (value === '') {
    tavernConfig.value.cors.maxAge = null;
    return;
  }
  const parsed = Number(value);
  tavernConfig.value.cors.maxAge = Number.isNaN(parsed) ? null : parsed;
};

const addRequestProxyBypass = () => {
  tavernConfig.value.requestProxy.bypass.push('');
};

const removeRequestProxyBypass = (index: number) => {
  tavernConfig.value.requestProxy.bypass.splice(index, 1);
};

// Watch for config changes and auto-save
watch(tavernConfig, () => {
  if (loading.value || configError.value || isInitialLoad.value) {
    return;
  }
  
  scheduleSave();
}, { deep: true });

onBeforeUnmount(() => {
  if (saveTimer) {
    clearTimeout(saveTimer);
  }
});

onMounted(() => {
  loadConfig();
});
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="flex items-center justify-between mb-6 px-1 shrink-0">
      <div>
        <h1 class="text-2xl font-bold dark:text-slate-100">{{ t('tavern.title') }}</h1>
        <p class="text-slate-500 dark:text-slate-400 text-sm mt-1">{{ t('tavern.subtitle') }} ({{ version || t('tavern.notSelected') }})</p>
      </div>
      <button
        @click="openConfigFile"
        class="flex items-center gap-2 px-4 py-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-300 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700 hover:text-blue-600 dark:hover:text-blue-400 transition-colors shadow-sm text-sm font-medium"
        :disabled="!version || !!configError"
        :class="{ 'opacity-50 cursor-not-allowed': !version || !!configError }"
      >
        <PhFileCode :size="18" weight="duotone" />
        {{ t('tavern.openConfigFile') }}
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto px-1 pb-10 min-h-0 relative">
      
      <!-- Loading State -->
      <div v-if="loading" class="flex flex-col items-center justify-center py-20">
        <div class="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin mb-4"></div>
        <p class="text-sm font-medium text-slate-500 dark:text-slate-400">{{ t('tavern.readingConfig') }}</p>
      </div>

      <!-- Error State -->
      <div v-else-if="configError" class="bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 p-4 rounded-xl border border-red-100 dark:border-red-800 flex items-start gap-3">
        <div class="mt-0.5">⚠️</div>
        <div>
          <h3 class="font-semibold">{{ t('tavern.cannotLoadConfig') }}</h3>
          <p class="text-sm opacity-90 mt-1">{{ configError }}</p>
        </div>
      </div>

      <!-- Settings Form -->
      <div v-else class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
        
        <!-- Network Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 flex items-center gap-2">
            <PhGlobe :size="20" class="text-blue-500" weight="duotone" />
            {{ t('tavern.networkAndAccess') }}
          </h2>
          
          <div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm">
            <!-- Port -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-blue-50 dark:bg-blue-900/30 flex items-center justify-center text-blue-500 dark:text-blue-400">
                  <PhListNumbers :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('tavern.serverPort') }}</div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('tavern.serverPortDesc') }}</div>
                </div>
              </div>
              <input 
                type="number" 
                v-model="tavernConfig.port"
                class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 w-24 outline-none transition-all"
              />
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <!-- Listen (Host) -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-green-50 dark:bg-green-900/30 flex items-center justify-center text-green-500 dark:text-green-400">
                  <PhGlobe :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('tavern.allowLanAccess') }}</div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('tavern.allowLanAccessDesc') }}</div>
                </div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.listen" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
              </label>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <div class="space-y-3">
              <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.listenAddress') }}</div>
              <div class="flex items-center gap-2">
                <span class="text-xs text-slate-500 dark:text-slate-400 w-14">IPv4</span>
                <input
                  type="text"
                  v-model="tavernConfig.listenAddress.ipv4"
                  class="flex-1 bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2 outline-none transition-all"
                />
              </div>
              <div class="flex items-center gap-2">
                <span class="text-xs text-slate-500 dark:text-slate-400 w-14">IPv6</span>
                <input
                  type="text"
                  v-model="tavernConfig.listenAddress.ipv6"
                  class="flex-1 bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2 outline-none transition-all"
                />
              </div>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <div class="space-y-3">
              <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.protocolSwitch') }}</div>
              <div class="flex items-center justify-between">
                <div class="text-sm text-slate-600 dark:text-slate-400">{{ t('tavern.enableIPv4') }}</div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" v-model="tavernConfig.protocol.ipv4" class="sr-only peer">
                  <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
                </label>
              </div>
              <div class="flex items-center justify-between">
                <div class="text-sm text-slate-600 dark:text-slate-400">{{ t('tavern.enableIPv6') }}</div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" v-model="tavernConfig.protocol.ipv6" class="sr-only peer">
                  <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
                </label>
              </div>
            </div>
          </div>
        </section>

        <!-- Security Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 flex items-center gap-2">
            <PhLockKey :size="20" class="text-purple-500" weight="duotone" />
            {{ t('tavern.securityWhitelist') }}
          </h2>
          
          <div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm">
            <!-- Whitelist Mode -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-purple-50 dark:bg-purple-900/30 flex items-center justify-center text-purple-500 dark:text-purple-400">
                  <PhLockKey :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('tavern.enableWhitelist') }}</div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('tavern.enableWhitelistDesc') }}</div>
                </div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.whitelistMode" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-purple-500"></div>
              </label>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <!-- Whitelist IPs -->
            <div class="space-y-3" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.whitelistMode }">
              <div class="flex items-center justify-between">
                <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.whitelistIPs') }}</div>
                <button 
                  @click="addWhitelistItem"
                  class="text-xs text-purple-600 dark:text-purple-400 hover:text-purple-700 dark:hover:text-purple-300 font-medium px-2 py-1 bg-purple-50 dark:bg-purple-900/30 rounded-md transition-colors"
                >
                  + {{ t('tavern.addIP') }}
                </button>
              </div>
              
              <div v-for="(_, index) in tavernConfig.whitelist" :key="index" class="flex items-center gap-2">
                <input 
                  type="text" 
                  v-model="tavernConfig.whitelist[index]"
                  :placeholder="t('tavern.ipPlaceholder')"
                  class="flex-1 bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-purple-500 focus:border-purple-500 block p-2 outline-none transition-all"
                />
                <button 
                  @click="removeWhitelistItem(index)"
                  class="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-lg transition-colors"
                  :title="t('common.remove')"
                >
                  ✕
                </button>
              </div>
              <div v-if="tavernConfig.whitelist.length === 0" class="text-xs text-slate-400 italic">
                {{ t('tavern.emptyList') }}
              </div>
            </div>
          </div>
        </section>

        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 flex items-center gap-2">
            <PhLockKey :size="20" class="text-indigo-500" weight="duotone" />
            {{ t('tavern.basicAuth') }}
          </h2>

          <div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-3 shadow-sm">
            <div class="flex items-center justify-between">
              <div>
                <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('tavern.enableBasicAuth') }}</div>
                <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('tavern.enableBasicAuthDesc') }}</div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.basicAuthMode" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500"></div>
              </label>
            </div>
            <div class="flex items-center justify-between">
              <div>
                <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('tavern.enableUserAccounts') }}</div>
                <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('tavern.enableUserAccountsDesc') }}</div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.enableUserAccounts" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500"></div>
              </label>
            </div>
            <div class="flex items-center justify-between" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.enableUserAccounts }">
              <div>
                <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('tavern.discreetLogin') }}</div>
                <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('tavern.discreetLoginDesc') }}</div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.enableDiscreetLogin" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500"></div>
              </label>
            </div>
            <div class="flex items-center justify-between" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.enableUserAccounts }">
              <div>
                <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('tavern.perUserBasicAuth') }}</div>
                <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('tavern.perUserBasicAuthDesc') }}</div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.perUserBasicAuth" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500"></div>
              </label>
            </div>
            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>
            <div class="space-y-2" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.basicAuthMode }">
              <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.username') }}</div>
              <input
                type="text"
                v-model="tavernConfig.basicAuthUser.username"
                class="w-full bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-indigo-500 focus:border-indigo-500 block p-2 outline-none transition-all"
              />
            </div>
            <div class="space-y-2" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.basicAuthMode }">
              <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.password') }}</div>
              <input
                type="text"
                v-model="tavernConfig.basicAuthUser.password"
                class="w-full bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-indigo-500 focus:border-indigo-500 block p-2 outline-none transition-all"
              />
            </div>
          </div>
        </section>

        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 flex items-center gap-2">
            <PhGlobe :size="20" class="text-cyan-500" weight="duotone" />
            {{ t('tavern.corsConfig') }}
          </h2>

          <div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm">
            <div class="flex items-center justify-between">
              <div>
                <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('tavern.enableCors') }}</div>
                <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('tavern.enableCorsDesc') }}</div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.cors.enabled" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-cyan-500"></div>
              </label>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <div class="space-y-3" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.cors.enabled }">
              <div class="flex items-center justify-between">
                <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.allowedOrigins') }}</div>
                <button
                  @click="addCorsOrigin"
                  class="text-xs text-cyan-600 dark:text-cyan-400 hover:text-cyan-700 dark:hover:text-cyan-300 font-medium px-2 py-1 bg-cyan-50 dark:bg-cyan-900/30 rounded-md transition-colors"
                >
                  + {{ t('tavern.addOrigin') }}
                </button>
              </div>

              <div v-for="(_, index) in tavernConfig.cors.origin" :key="`cors-${index}`" class="flex items-center gap-2">
                <input
                  type="text"
                  v-model="tavernConfig.cors.origin[index]"
                  :placeholder="t('tavern.originPlaceholder')"
                  class="flex-1 bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-cyan-500 focus:border-cyan-500 block p-2 outline-none transition-all"
                />
                <button
                  @click="removeCorsOrigin(index)"
                  class="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-lg transition-colors"
                  :title="t('common.remove')"
                >
                  ✕
                </button>
              </div>

              <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

              <div class="flex items-center justify-between">
                <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.allowedMethods') }}</div>
                <button
                  @click="addCorsMethod"
                  class="text-xs text-cyan-600 dark:text-cyan-400 hover:text-cyan-700 dark:hover:text-cyan-300 font-medium px-2 py-1 bg-cyan-50 dark:bg-cyan-900/30 rounded-md transition-colors"
                >
                  + {{ t('tavern.addMethod') }}
                </button>
              </div>

              <div v-for="(_, index) in tavernConfig.cors.methods" :key="`cors-method-${index}`" class="flex items-center gap-2">
                <input
                  type="text"
                  v-model="tavernConfig.cors.methods[index]"
                  :placeholder="t('tavern.methodPlaceholder')"
                  class="flex-1 bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-cyan-500 focus:border-cyan-500 block p-2 outline-none transition-all"
                />
                <button
                  @click="removeCorsMethod(index)"
                  class="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-lg transition-colors"
                  :title="t('common.remove')"
                >
                  ✕
                </button>
              </div>

              <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

              <div class="flex items-center justify-between">
                <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.allowedHeaders') }}</div>
                <button
                  @click="addCorsAllowedHeader"
                  class="text-xs text-cyan-600 dark:text-cyan-400 hover:text-cyan-700 dark:hover:text-cyan-300 font-medium px-2 py-1 bg-cyan-50 dark:bg-cyan-900/30 rounded-md transition-colors"
                >
                  + {{ t('tavern.addHeader') }}
                </button>
              </div>

              <div v-for="(_, index) in tavernConfig.cors.allowedHeaders" :key="`cors-allowed-${index}`" class="flex items-center gap-2">
                <input
                  type="text"
                  v-model="tavernConfig.cors.allowedHeaders[index]"
                  :placeholder="t('tavern.headerPlaceholder')"
                  class="flex-1 bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-cyan-500 focus:border-cyan-500 block p-2 outline-none transition-all"
                />
                <button
                  @click="removeCorsAllowedHeader(index)"
                  class="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-lg transition-colors"
                  :title="t('common.remove')"
                >
                  ✕
                </button>
              </div>

              <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

              <div class="flex items-center justify-between">
                <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.exposedHeaders') }}</div>
                <button
                  @click="addCorsExposedHeader"
                  class="text-xs text-cyan-600 dark:text-cyan-400 hover:text-cyan-700 dark:hover:text-cyan-300 font-medium px-2 py-1 bg-cyan-50 dark:bg-cyan-900/30 rounded-md transition-colors"
                >
                  + {{ t('tavern.addExposedHeader') }}
                </button>
              </div>

              <div v-for="(_, index) in tavernConfig.cors.exposedHeaders" :key="`cors-exposed-${index}`" class="flex items-center gap-2">
                <input
                  type="text"
                  v-model="tavernConfig.cors.exposedHeaders[index]"
                  :placeholder="t('tavern.exposedHeaderPlaceholder')"
                  class="flex-1 bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-cyan-500 focus:border-cyan-500 block p-2 outline-none transition-all"
                />
                <button
                  @click="removeCorsExposedHeader(index)"
                  class="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-lg transition-colors"
                  :title="t('common.remove')"
                >
                  ✕
                </button>
              </div>

              <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

              <div class="flex items-center justify-between">
                <div class="text-sm text-slate-600 dark:text-slate-400">{{ t('tavern.allowCredentials') }}</div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" v-model="tavernConfig.cors.credentials" class="sr-only peer">
                  <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-cyan-500"></div>
                </label>
              </div>

              <div class="space-y-2">
                <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.maxAge') }}</div>
                <input
                  type="number"
                  :value="tavernConfig.cors.maxAge ?? ''"
                  @input="onCorsMaxAgeInput"
                  class="w-full bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-cyan-500 focus:border-cyan-500 block p-2 outline-none transition-all"
                />
              </div>
            </div>
          </div>
        </section>

        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 flex items-center gap-2">
            <PhGlobe :size="20" class="text-rose-500" weight="duotone" />
            {{ t('tavern.requestProxy') }}
          </h2>

          <div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm">
            <div class="flex items-center justify-between">
              <div>
                <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('tavern.enableProxy') }}</div>
                <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('tavern.enableProxyDesc') }}</div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.requestProxy.enabled" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-rose-500"></div>
              </label>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <div class="space-y-3" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.requestProxy.enabled }">
              <div class="space-y-2">
                <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.proxyUrl') }}</div>
                <input
                  type="text"
                  v-model="tavernConfig.requestProxy.url"
                  :placeholder="t('tavern.proxyUrlPlaceholder')"
                  class="w-full bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-rose-500 focus:border-rose-500 block p-2 outline-none transition-all"
                />
              </div>

              <div class="flex items-center justify-between">
                <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.bypassList') }}</div>
                <button
                  @click="addRequestProxyBypass"
                  class="text-xs text-rose-600 dark:text-rose-400 hover:text-rose-700 dark:hover:text-rose-300 font-medium px-2 py-1 bg-rose-50 dark:bg-rose-900/30 rounded-md transition-colors"
                >
                  + {{ t('tavern.addBypass') }}
                </button>
              </div>

              <div v-for="(_, index) in tavernConfig.requestProxy.bypass" :key="`proxy-${index}`" class="flex items-center gap-2">
                <input
                  type="text"
                  v-model="tavernConfig.requestProxy.bypass[index]"
                  :placeholder="t('tavern.bypassPlaceholder')"
                  class="flex-1 bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-rose-500 focus:border-rose-500 block p-2 outline-none transition-all"
                />
                <button
                  @click="removeRequestProxyBypass(index)"
                  class="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-lg transition-colors"
                  :title="t('common.remove')"
                >
                  ✕
                </button>
              </div>
            </div>
          </div>
        </section>

        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 flex items-center gap-2">
            <PhBrowser :size="20" class="text-emerald-500" weight="duotone" />
            {{ t('tavern.backupSettings') }}
          </h2>

          <div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm">
            <div class="space-y-2">
              <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.commonBackups') }}</div>
              <input
                type="number"
                v-model.number="tavernConfig.backups.common.numberOfBackups"
                class="w-full bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-emerald-500 focus:border-emerald-500 block p-2 outline-none transition-all"
              />
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <div class="flex items-center justify-between">
              <div class="text-sm text-slate-600 dark:text-slate-400">{{ t('tavern.enableChatBackup') }}</div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.backups.chat.enabled" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500"></div>
              </label>
            </div>

            <div class="flex items-center justify-between" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.backups.chat.enabled }">
              <div class="text-sm text-slate-600 dark:text-slate-400">{{ t('tavern.checkIntegrity') }}</div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.backups.chat.checkIntegrity" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500"></div>
              </label>
            </div>

            <div class="grid grid-cols-2 gap-3" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.backups.chat.enabled }">
              <div class="space-y-2">
                <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.maxTotalBackups') }}</div>
                <input
                  type="number"
                  v-model.number="tavernConfig.backups.chat.maxTotalBackups"
                  class="w-full bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-emerald-500 focus:border-emerald-500 block p-2 outline-none transition-all"
                />
              </div>
              <div class="space-y-2">
                <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.throttleInterval') }}</div>
                <input
                  type="number"
                  v-model.number="tavernConfig.backups.chat.throttleInterval"
                  class="w-full bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-emerald-500 focus:border-emerald-500 block p-2 outline-none transition-all"
                />
              </div>
            </div>
          </div>
        </section>

        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 flex items-center gap-2">
            <PhBrowser :size="20" class="text-teal-500" weight="duotone" />
            {{ t('tavern.thumbnailSettings') }}
          </h2>

          <div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm">
            <div class="flex items-center justify-between">
              <div class="text-sm text-slate-600 dark:text-slate-400">{{ t('tavern.enableThumbnails') }}</div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.thumbnails.enabled" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-teal-500"></div>
              </label>
            </div>

            <div class="grid grid-cols-2 gap-3" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.thumbnails.enabled }">
              <div class="space-y-2">
                <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.thumbnailFormat') }}</div>
                <input
                  type="text"
                  v-model="tavernConfig.thumbnails.format"
                  class="w-full bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-teal-500 focus:border-teal-500 block p-2 outline-none transition-all"
                />
              </div>
              <div class="space-y-2">
                <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.thumbnailQuality') }}</div>
                <input
                  type="number"
                  v-model.number="tavernConfig.thumbnails.quality"
                  class="w-full bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-teal-500 focus:border-teal-500 block p-2 outline-none transition-all"
                />
              </div>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <div class="space-y-3" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.thumbnails.enabled }">
              <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.dimensions') }}</div>
              <div class="grid grid-cols-5 gap-2 items-center">
                <div class="text-sm text-slate-500 dark:text-slate-400">{{ t('tavern.background') }}</div>
                <input type="number" v-model.number="tavernConfig.thumbnails.dimensions.bg[0]" class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg p-2 outline-none transition-all focus:ring-teal-500 focus:border-teal-500" />
                <div class="text-center text-slate-400">x</div>
                <input type="number" v-model.number="tavernConfig.thumbnails.dimensions.bg[1]" class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg p-2 outline-none transition-all focus:ring-teal-500 focus:border-teal-500" />
                <div></div>
              </div>
              <div class="grid grid-cols-5 gap-2 items-center">
                <div class="text-sm text-slate-500 dark:text-slate-400">{{ t('tavern.avatar') }}</div>
                <input type="number" v-model.number="tavernConfig.thumbnails.dimensions.avatar[0]" class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg p-2 outline-none transition-all focus:ring-teal-500 focus:border-teal-500" />
                <div class="text-center text-slate-400">x</div>
                <input type="number" v-model.number="tavernConfig.thumbnails.dimensions.avatar[1]" class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg p-2 outline-none transition-all focus:ring-teal-500 focus:border-teal-500" />
                <div></div>
              </div>
              <div class="grid grid-cols-5 gap-2 items-center">
                <div class="text-sm text-slate-500 dark:text-slate-400">{{ t('tavern.persona') }}</div>
                <input type="number" v-model.number="tavernConfig.thumbnails.dimensions.persona[0]" class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg p-2 outline-none transition-all focus:ring-teal-500 focus:border-teal-500" />
                <div class="text-center text-slate-400">x</div>
                <input type="number" v-model.number="tavernConfig.thumbnails.dimensions.persona[1]" class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg p-2 outline-none transition-all focus:ring-teal-500 focus:border-teal-500" />
                <div></div>
              </div>
            </div>
          </div>
        </section>

        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100 flex items-center gap-2">
            <PhBrowser :size="20" class="text-sky-500" weight="duotone" />
            {{ t('tavern.browserSettings') }}
          </h2>

          <div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm">
            <div class="flex items-center justify-between">
              <div>
                <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('tavern.autoLaunchBrowser') }}</div>
                <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('tavern.autoLaunchBrowserDesc') }}</div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.browserLaunchEnabled" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-sky-500"></div>
              </label>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <div class="space-y-2">
              <div class="font-medium text-slate-700 dark:text-slate-300 text-sm">{{ t('tavern.browserType') }}</div>
              <input
                type="text"
                v-model="tavernConfig.browserType"
                class="w-full bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-sky-500 focus:border-sky-500 block p-2 outline-none transition-all"
              />
            </div>
          </div>
        </section>

        <!-- Browser Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2">
            <PhBrowser :size="20" class="text-orange-500" weight="duotone" />
            {{ t('tavern.browserSettings') }}
          </h2>
          
          <div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 space-y-4 shadow-sm">
            <!-- Auto run -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-orange-50 dark:bg-orange-900/30 flex items-center justify-center text-orange-500 dark:text-orange-400">
                  <PhBrowser :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('tavern.autoOpenOnStart') }}</div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('tavern.autoOpenOnStartDesc') }}</div>
                </div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.browserLaunchEnabled" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-orange-500"></div>
              </label>
            </div>

            <div class="w-full h-px bg-slate-100 dark:bg-slate-700"></div>

            <!-- Browser Type -->
            <div class="flex items-center justify-between" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.browserLaunchEnabled }">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-slate-50 dark:bg-slate-700 flex items-center justify-center text-slate-500 dark:text-slate-400">
                  <PhBrowser :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700 dark:text-slate-300">{{ t('tavern.browserSelection') }}</div>
                  <div class="text-xs text-slate-500 dark:text-slate-400">{{ t('tavern.browserSelectionDesc') }}</div>
                </div>
              </div>
              <select 
                v-model="tavernConfig.browserType" 
                class="bg-slate-50 dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm rounded-lg focus:ring-orange-500 focus:border-orange-500 block p-2.5 min-w-[120px] outline-none transition-all"
              >
                <option value="default">{{ t('tavern.browserDefault') }}</option>
                <option value="chrome">Chrome</option>
                <option value="edge">Edge</option>
                <option value="firefox">Firefox</option>
              </select>
            </div>
          </div>
        </section>

      </div>
    </div>
  </div>
</template>

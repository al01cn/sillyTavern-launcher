<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { toast } from 'vue-sonner';
import {
  PhList, PhGlobe, PhLockKey, PhListNumbers,
  PhShieldCheck, PhImage, PhActivity,
  PhCaretDown,
  PhCircleNotch, PhCheckCircle,
  PhFolderOpen, PhGear,
  PhCpu, PhPlugsConnected, PhLightning,
  PhX, PhPlus, PhFile, PhArrowsDownUp
} from '@phosphor-icons/vue';
import { useI18n } from 'vue-i18n';
import ConfigMigrateDialog from '../components/ConfigMigrateDialog.vue';

const { t } = useI18n();

const loading = ref(true);
const configError = ref<string | null>(null);
const saveInProgress = ref(false);
const saveQueued = ref(false);
let saveTimer: ReturnType<typeof setTimeout> | null = null;

const showMigrateDialog = ref(false);

// 折叠状态管理
const collapsedSections = ref({
  network: false,
  security: true,
  ssl: true,
  cors: true,
  proxy: true,
  backup: true,
  thumbnail: true,
  performance: true,
  logging: true,
  other: true
});

const toggleSection = (section: keyof typeof collapsedSections.value) => {
  collapsedSections.value[section] = !collapsedSections.value[section];
};

type TavernConfigPayload = {
  port: number;
  listen: boolean;
  listenAddress: { ipv4: string; ipv6: string; };
  protocol: { ipv4: boolean; ipv6: boolean; };
  basicAuthMode: boolean;
  enableUserAccounts: boolean;
  enableDiscreetLogin: boolean;
  perUserBasicAuth: boolean;
  basicAuthUser: { username: string; password: string; };
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
    common: { numberOfBackups: number; };
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
    dimensions: { bg: number[]; avatar: number[]; persona: number[]; };
  };
  browserLaunchEnabled: boolean;
  browserType: string;
  ssl: {
    enabled: boolean;
    certPath: string;
    keyPath: string;
    keyPassphrase: string;
  };
  dnsPreferIPv6: boolean;
  heartbeatInterval: number;
  hostWhitelist: { enabled: boolean; scan: boolean; hosts: string[]; };
  whitelistImportDomains: string[];
  sessionTimeout: number;
  disableCsrfProtection: boolean;
  securityOverride: boolean;
  allowKeysExposure: boolean;
  skipContentCheck: boolean;
  logging: { enableAccessLog: boolean; minLogLevel: number; };
  performance: {
    lazyLoadCharacters: boolean;
    memoryCacheCapacity: string;
    useDiskCache: boolean;
  };
  cacheBuster: { enabled: boolean; userAgentPattern: string; };
  sso: { autheliaAuth: boolean; authentikAuth: boolean; };
  extensions: { enabled: boolean; autoUpdate: boolean; };
  enableServerPlugins: boolean;
  enableServerPluginsAutoUpdate: boolean;
  enableCorsProxy: boolean;
  promptPlaceholder: string;
  enableDownloadableTokenizers: boolean;
};

const tavernConfig = ref<TavernConfigPayload>({
  port: 8000,
  listen: false,
  listenAddress: { ipv4: '0.0.0.0', ipv6: '[::]' },
  protocol: { ipv4: true, ipv6: false },
  basicAuthMode: false,
  enableUserAccounts: false,
  enableDiscreetLogin: false,
  perUserBasicAuth: false,
  basicAuthUser: { username: 'user', password: 'password' },
  whitelistMode: true,
  whitelist: [],
  cors: {
    enabled: true, origin: ['null'], methods: ['OPTIONS'], allowedHeaders: [], exposedHeaders: [], credentials: false, maxAge: null
  },
  requestProxy: { enabled: false, url: '', bypass: [] },
  backups: {
    common: { numberOfBackups: 50 },
    chat: { enabled: true, checkIntegrity: true, maxTotalBackups: -1, throttleInterval: 10000 }
  },
  thumbnails: {
    enabled: true, format: 'jpg', quality: 95, dimensions: { bg: [160, 90], avatar: [96, 144], persona: [96, 144] }
  },
  browserLaunchEnabled: true,
  browserType: 'default',
  ssl: { enabled: false, certPath: './certs/cert.pem', keyPath: './certs/privkey.pem', keyPassphrase: '' },
  dnsPreferIPv6: false,
  heartbeatInterval: 0,
  hostWhitelist: { enabled: false, scan: true, hosts: [] },
  whitelistImportDomains: [],
  sessionTimeout: -1,
  disableCsrfProtection: false,
  securityOverride: false,
  allowKeysExposure: false,
  skipContentCheck: false,
  logging: { enableAccessLog: true, minLogLevel: 0 },
  performance: { lazyLoadCharacters: false, memoryCacheCapacity: '100mb', useDiskCache: true },
  cacheBuster: { enabled: false, userAgentPattern: '' },
  sso: { autheliaAuth: false, authentikAuth: false },
  extensions: { enabled: true, autoUpdate: true },
  enableServerPlugins: false,
  enableServerPluginsAutoUpdate: true,
  enableCorsProxy: false,
  promptPlaceholder: '[Start a new chat]',
  enableDownloadableTokenizers: true
});

const isInitialLoad = ref(true);

const loadConfig = async () => {
  try {
    const fetchedConfig = await invoke<TavernConfigPayload>('get_sillytavern_global_config_options');
    isInitialLoad.value = true;
    tavernConfig.value = fetchedConfig;
    setTimeout(() => { isInitialLoad.value = false; }, 50);
  } catch (error: any) {
    configError.value = typeof error === 'string' ? error : t('tavern.loadConfigFailed');
  } finally {
    loading.value = false;
  }
};

const saveConfig = async () => {
  if (isInitialLoad.value || loading.value || configError.value) return;
  if (saveInProgress.value) {
    saveQueued.value = true;
    return;
  }

  saveInProgress.value = true;
  try {
    await invoke('update_sillytavern_global_config_options', { config: tavernConfig.value });
  } catch (error) {
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
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => { void saveConfig(); }, 500);
};

watch(tavernConfig, () => {
  if (!isInitialLoad.value && !loading.value) scheduleSave();
}, { deep: true });

onMounted(loadConfig);

const openConfigFile = async () => {
  try {
    await invoke('open_sillytavern_global_config_file');
  } catch (e) {
    toast.error(t('tavern.cannotOpenFile'));
  }
};

const addDomain = () => {
  tavernConfig.value.whitelistImportDomains.push('');
};

const removeDomain = (index: number) => {
  tavernConfig.value.whitelistImportDomains.splice(index, 1);
};

const addWhitelistIP = () => {
  tavernConfig.value.whitelist.push('');
};

const removeWhitelistIP = (index: number) => {
  tavernConfig.value.whitelist.splice(index, 1);
};

const addHost = () => {
  tavernConfig.value.hostWhitelist.hosts.push('');
};

const removeHost = (index: number) => {
  tavernConfig.value.hostWhitelist.hosts.splice(index, 1);
};

// CORS 相关方法
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

const addCorsHeader = () => {
  tavernConfig.value.cors.allowedHeaders.push('');
};

const removeCorsHeader = (index: number) => {
  tavernConfig.value.cors.allowedHeaders.splice(index, 1);
};

const addCorsExposedHeader = () => {
  tavernConfig.value.cors.exposedHeaders.push('');
};

const removeCorsExposedHeader = (index: number) => {
  tavernConfig.value.cors.exposedHeaders.splice(index, 1);
};

// 代理相关方法
const addBypass = () => {
  tavernConfig.value.requestProxy.bypass.push('');
};

const removeBypass = (index: number) => {
  tavernConfig.value.requestProxy.bypass.splice(index, 1);
};

// SSL 证书文件选择
const selectCertFile = async () => {
  try {
    const selected = await open({
      title: t('tavern.selectCertFile'),
      multiple: false,
      filters: [{
        name: 'Certificate Files',
        extensions: ['pem', 'crt', 'cer']
      }]
    });
    if (selected) {
      tavernConfig.value.ssl.certPath = selected.toString();
    }
  } catch (error) {
    toast.error(t('tavern.cannotOpenFile'));
  }
};

const selectKeyFile = async () => {
  try {
    const selected = await open({
      title: t('tavern.selectKeyFile'),
      multiple: false,
      filters: [{
        name: 'Key Files',
        extensions: ['pem', 'key']
      }]
    });
    if (selected) {
      tavernConfig.value.ssl.keyPath = selected.toString();
    }
  } catch (error) {
    toast.error(t('tavern.cannotOpenFile'));
  }
};

// 内存缓存容量处理
const memoryCacheCapacityValue = ref<string>(
  tavernConfig.value.performance.memoryCacheCapacity.replace('mb', '') || '100'
);

watch(memoryCacheCapacityValue, (newValue) => {
  const numValue = newValue.replace(/[^0-9]/g, '');
  tavernConfig.value.performance.memoryCacheCapacity = numValue ? `${numValue}mb` : '0mb';
});

// 监听配置变化，同步更新显示值
watch(() => tavernConfig.value.performance.memoryCacheCapacity, (newValue) => {
  const numValue = newValue.replace('mb', '');
  if (numValue !== memoryCacheCapacityValue.value) {
    memoryCacheCapacityValue.value = numValue || '100';
  }
});
</script>

<template>
  <div class="flex flex-col h-full bg-slate-50 dark:bg-slate-900 transition-colors duration-300">
    <!-- Header -->
    <header
      class="flex items-center justify-between p-6 shrink-0 border-b border-slate-200 dark:border-slate-800 bg-white/90 dark:bg-slate-900/90 backdrop-blur sticky top-0 z-20">
      <div class="flex items-center gap-4">
        <div class="w-10 h-10 bg-blue-600 rounded-xl flex items-center justify-center shadow-md shadow-blue-500/20">
          <PhList :size="24" weight="fill" class="text-white" />
        </div>
        <div>
          <h1 class="text-lg font-bold text-slate-900 dark:text-slate-100 uppercase tracking-tight">
            {{ t('tavern.title') }}
          </h1>
          <p class="text-slate-500 dark:text-slate-400 text-[10px] flex items-center gap-1">
            <PhGear :size="10" />
            {{ t('tavern.subtitle') }}
          </p>
        </div>
      </div>
      <div class="flex items-center gap-3">
        <div v-if="saveInProgress"
          class="flex items-center gap-2 px-2 py-1 bg-blue-50 dark:bg-blue-900/30 rounded text-[10px] font-bold text-blue-600 dark:text-blue-400 border border-blue-100 dark:border-blue-800">
          <PhCircleNotch :size="12" class="animate-spin" />
          {{ t('tavern.syncing') }}
        </div>
        <div v-else
          class="flex items-center gap-2 px-2 py-1 bg-emerald-50 dark:bg-emerald-900/30 rounded text-[10px] font-bold text-emerald-600 dark:text-emerald-400 border border-emerald-100 dark:border-emerald-800">
          <PhCheckCircle :size="12" weight="fill" />
          {{ t('tavern.ready') }}
        </div>
        <button @click="openConfigFile"
          class="flex items-center gap-2 px-3 py-1.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-slate-600 dark:text-slate-300 hover:border-blue-500 hover:text-blue-500 transition-all text-xs font-bold active:scale-95 shadow-sm">
          <PhFolderOpen :size="16" />
          {{ t('tavern.openConfigFile') }}
        </button>
        <button @click="showMigrateDialog = true"
          class="flex items-center gap-2 px-3 py-1.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-slate-600 dark:text-slate-300 hover:border-indigo-500 hover:text-indigo-500 transition-all text-xs font-bold active:scale-95 shadow-sm">
          <PhArrowsDownUp :size="16" />
          {{ t('tavern.migrate.button') }}
        </button>
      </div>
    </header>

    <main class="flex-1 overflow-y-auto p-6 space-y-4">
      <div v-if="loading" class="flex flex-col items-center justify-center py-20">
        <PhCircleNotch :size="32" class="animate-spin text-blue-500 mb-2" />
        <p class="text-xs font-bold text-slate-400">{{ t('tavern.loadingConfiguration') }}</p>
      </div>

      <div v-else class="max-w-4xl mx-auto space-y-4 pb-20">
        <!-- 1. 网络基础 -->
        <div class="config-card" :class="{ 'collapsed': collapsedSections.network }">
          <div class="config-card-header" @click="toggleSection('network')">
            <div class="header-left">
              <div class="header-icon bg-blue-100 dark:bg-blue-900/50 text-blue-600">
                <PhGlobe :size="20" weight="duotone" />
              </div>
              <div class="header-info">
                <span class="title">{{ t('tavern.networkAndAccess') }}</span>
                <span class="desc">{{ t('tavern.networkAndAccessDesc') }}</span>
              </div>
            </div>
            <PhCaretDown :size="18" class="chevron" />
          </div>
          <div class="config-card-body">
            <div class="p-5 space-y-6">
              <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.serverPort') }}</label>
                  <input type="number" v-model="tavernConfig.port" class="form-input" />
                </div>
              </div>

              <div class="grid grid-cols-1 md:grid-cols-2 gap-6 pt-4 border-t border-slate-100 dark:border-slate-700">
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.ipv4ListenAddress') }}</label>
                  <input type="text" v-model="tavernConfig.listenAddress.ipv4" class="form-input"
                    placeholder="0.0.0.0" />
                </div>
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.ipv6ListenAddress') }}</label>
                  <input type="text" v-model="tavernConfig.listenAddress.ipv6" class="form-input" placeholder="[::]" />
                </div>
              </div>

              <div class="flex gap-4 pt-2">
                <label class="checkbox-pill">
                  <input type="checkbox" v-model="tavernConfig.listen">
                  <PhCheckCircle v-if="tavernConfig.listen" weight="fill" />
                  {{ t('tavern.allowLanAccess') }}
                </label>
                <label class="checkbox-pill">
                  <input type="checkbox" v-model="tavernConfig.protocol.ipv4">
                  <PhCheckCircle v-if="tavernConfig.protocol.ipv4" weight="fill" />
                  {{ t('tavern.enableIPv4') }}
                </label>
                <label class="checkbox-pill">
                  <input type="checkbox" v-model="tavernConfig.protocol.ipv6">
                  <PhCheckCircle v-if="tavernConfig.protocol.ipv6" weight="fill" />
                  {{ t('tavern.enableIPv6') }}
                </label>
                <label class="checkbox-pill">
                  <input type="checkbox" v-model="tavernConfig.dnsPreferIPv6">
                  <PhCheckCircle v-if="tavernConfig.dnsPreferIPv6" weight="fill" />
                  {{ t('tavern.dnsPreferIPv6') }}
                </label>
              </div>

              <div class="grid grid-cols-1 md:grid-cols-2 gap-4 pt-4 border-t border-slate-100 dark:border-slate-700">
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.heartbeatInterval') }}</label>
                  <input type="number" v-model="tavernConfig.heartbeatInterval" class="form-input" placeholder="0" />
                  <span class="text-[10px] text-slate-400 mt-1">{{ t('tavern.heartbeatIntervalDesc') }}</span>
                </div>
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.browserType') }}</label>
                  <select v-model="tavernConfig.browserType" class="form-input">
                    <option value="default">{{ t('tavern.browserDefault') }}</option>
                    <option value="chrome">Chrome</option>
                    <option value="firefox">Firefox</option>
                    <option value="edge">Edge</option>
                  </select>
                </div>
              </div>

              <div class="pt-2">
                <label class="checkbox-pill">
                  <input type="checkbox" v-model="tavernConfig.browserLaunchEnabled">
                  <PhCheckCircle v-if="tavernConfig.browserLaunchEnabled" weight="fill" />
                  {{ t('tavern.autoLaunchBrowser') }}
                </label>
              </div>
            </div>
          </div>
        </div>

        <!-- 2. 安全与账户 -->
        <div class="config-card" :class="{ 'collapsed': collapsedSections.security }">
          <div class="config-card-header" @click="toggleSection('security')">
            <div class="header-left">
              <div class="header-icon bg-purple-100 dark:bg-purple-900/50 text-purple-600">
                <PhShieldCheck :size="20" weight="duotone" />
              </div>
              <div class="header-info">
                <span class="title">{{ t('tavern.securityWhitelist') }}</span>
                <span class="desc">{{ t('tavern.securityWhitelistDesc') }}</span>
              </div>
            </div>
            <PhCaretDown :size="18" class="chevron" />
          </div>
          <div class="config-card-body">
            <div class="p-5 space-y-6">
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ t('tavern.enableBasicAuth') }}</span>
                  </div>
                  <label class="toggle-switch">
                    <input type="checkbox" v-model="tavernConfig.basicAuthMode" class="sr-only peer">
                    <div class="toggle-slider"></div>
                  </label>
                </div>
                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ t('tavern.enableUserAccounts') }}</span>
                  </div>
                  <label class="toggle-switch">
                    <input type="checkbox" v-model="tavernConfig.enableUserAccounts" class="sr-only peer">
                    <div class="toggle-slider"></div>
                  </label>
                </div>
              </div>

              <div v-if="tavernConfig.basicAuthMode"
                class="grid grid-cols-2 gap-4 p-4 bg-slate-50 dark:bg-slate-900/50 rounded-2xl animate-in">
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.globalUsername') }}</label>
                  <input type="text" v-model="tavernConfig.basicAuthUser.username" class="form-input" />
                </div>
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.globalPassword') }}</label>
                  <input type="password" v-model="tavernConfig.basicAuthUser.password" class="form-input" />
                </div>
              </div>

              <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                <label class="checkbox-pill">
                  <input type="checkbox" v-model="tavernConfig.enableDiscreetLogin">
                  {{ t('tavern.discreetLogin') }}
                </label>
                <label class="checkbox-pill">
                  <input type="checkbox" v-model="tavernConfig.perUserBasicAuth">
                  {{ t('tavern.perUserBasicAuth') }}
                </label>
                <label class="checkbox-pill">
                  <input type="checkbox" v-model="tavernConfig.whitelistMode">
                  {{ t('tavern.enableWhitelist') }}
                </label>
              </div>

              <div v-if="tavernConfig.whitelistMode" class="space-y-3 p-4 bg-slate-50 dark:bg-slate-900/50 rounded-2xl animate-in">
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.whitelistIPs') }}</label>
                  <div class="space-y-2">
                    <div v-for="(_ip, index) in tavernConfig.whitelist" :key="index" 
                      class="flex items-center gap-2">
                      <input type="text" v-model="tavernConfig.whitelist[index]" 
                        class="form-input flex-1" :placeholder="t('tavern.ipPlaceholder')" />
                      <button @click="removeWhitelistIP(index)" 
                        class="px-3 py-2 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 transition-all active:scale-95">
                        <PhX :size="16" weight="bold" />
                      </button>
                    </div>
                    <button @click="addWhitelistIP" 
                      class="w-full py-2.5 border-2 border-dashed border-slate-300 dark:border-slate-600 rounded-xl text-slate-500 dark:text-slate-400 hover:border-blue-500 hover:text-blue-500 dark:hover:border-blue-400 dark:hover:text-blue-400 transition-all flex items-center justify-center gap-2">
                      <PhPlus :size="16" weight="bold" />
                      {{ t('common.add') }}
                    </button>
                  </div>
                  <span class="text-[10px] text-slate-400 mt-1">{{ t('tavern.enableWhitelistDesc') }}</span>
                </div>
              </div>

              <div class="pt-4 border-t border-slate-100 dark:border-slate-700">
                <h3 class="text-sm font-bold text-slate-700 dark:text-slate-300 mb-3">{{ t('tavern.hostWhitelist') }}</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div class="setting-row">
                    <div class="setting-info">
                      <span class="setting-label">{{ t('tavern.hostWhitelistEnabled') }}</span>
                    </div>
                    <label class="toggle-switch">
                      <input type="checkbox" v-model="tavernConfig.hostWhitelist.enabled" class="sr-only peer">
                      <div class="toggle-slider"></div>
                    </label>
                  </div>
                  <div class="setting-row">
                    <div class="setting-info">
                      <span class="setting-label">{{ t('tavern.hostWhitelistScan') }}</span>
                    </div>
                    <label class="toggle-switch">
                      <input type="checkbox" v-model="tavernConfig.hostWhitelist.scan" class="sr-only peer">
                      <div class="toggle-slider"></div>
                    </label>
                  </div>
                </div>
                <div v-if="tavernConfig.hostWhitelist.enabled" class="mt-3 input-group">
                  <label class="input-label">{{ t('tavern.addHost') }}</label>
                  <div class="space-y-2">
                    <div v-for="(_host, index) in tavernConfig.hostWhitelist.hosts" :key="index" 
                      class="flex items-center gap-2">
                      <input type="text" v-model="tavernConfig.hostWhitelist.hosts[index]" 
                        class="form-input flex-1" :placeholder="t('tavern.hostPlaceholder')" />
                      <button @click="removeHost(index)" 
                        class="px-3 py-2 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 transition-all active:scale-95">
                        <PhX :size="16" weight="bold" />
                      </button>
                    </div>
                    <button @click="addHost" 
                      class="w-full py-2.5 border-2 border-dashed border-slate-300 dark:border-slate-600 rounded-xl text-slate-500 dark:text-slate-400 hover:border-blue-500 hover:text-blue-500 dark:hover:border-blue-400 dark:hover:text-blue-400 transition-all flex items-center justify-center gap-2">
                      <PhPlus :size="16" weight="bold" />
                      {{ t('common.add') }}
                    </button>
                  </div>
                </div>
              </div>

              <div class="pt-4 border-t border-slate-100 dark:border-slate-700">
                <h3 class="text-sm font-bold text-slate-700 dark:text-slate-300 mb-3">{{ t('tavern.whitelistImportDomains') }}</h3>
                <div class="space-y-2">
                  <div v-for="(_domain, index) in tavernConfig.whitelistImportDomains" :key="index" 
                    class="flex items-center gap-2">
                    <input type="text" v-model="tavernConfig.whitelistImportDomains[index]" 
                      class="form-input flex-1" :placeholder="t('tavern.domainPlaceholder')" />
                    <button @click="removeDomain(index)" 
                      class="px-3 py-2 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 transition-all active:scale-95">
                      <PhX :size="16" weight="bold" />
                    </button>
                  </div>
                  <button @click="addDomain" 
                    class="w-full py-2.5 border-2 border-dashed border-slate-300 dark:border-slate-600 rounded-xl text-slate-500 dark:text-slate-400 hover:border-blue-500 hover:text-blue-500 dark:hover:border-blue-400 dark:hover:text-blue-400 transition-all flex items-center justify-center gap-2">
                    <PhPlus :size="16" weight="bold" />
                    {{ t('common.add') }}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 3. SSL 证书 -->
        <div class="config-card" :class="{ 'collapsed': collapsedSections.ssl }">
          <div class="config-card-header" @click="toggleSection('ssl')">
            <div class="header-left">
              <div class="header-icon bg-emerald-100 dark:bg-emerald-900/50 text-emerald-600">
                <PhLockKey :size="20" weight="duotone" />
              </div>
              <div class="header-info">
                <span class="title">HTTPS / SSL</span>
                <span class="desc">{{ t('tavern.sslConfigDesc') }}</span>
              </div>
            </div>
            <PhCaretDown :size="18" class="chevron" />
          </div>
          <div class="config-card-body">
            <div class="p-5 space-y-6">
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">{{ t('tavern.enableSsl') }}</span>
                  <span class="setting-desc">{{ t('tavern.enableSslDesc') }}</span>
                </div>
                <label class="toggle-switch">
                  <input type="checkbox" v-model="tavernConfig.ssl.enabled" class="sr-only peer">
                  <div class="toggle-slider"></div>
                </label>
              </div>
              <div v-if="tavernConfig.ssl.enabled" class="space-y-4 animate-in">
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.sslCertPath') }}</label>
                  <div class="flex gap-2">
                    <input type="text" v-model="tavernConfig.ssl.certPath" class="form-input flex-1" :placeholder="t('tavern.sslCertPathPlaceholder')" />
                    <button @click="selectCertFile" 
                      class="px-4 py-2 bg-slate-100 dark:bg-slate-700 text-slate-700 dark:text-slate-300 rounded-xl hover:bg-blue-50 dark:hover:bg-blue-900/30 hover:text-blue-600 dark:hover:text-blue-400 border border-slate-200 dark:border-slate-600 transition-all active:scale-95 flex items-center gap-2">
                      <PhFile :size="16" weight="bold" />
                      {{ t('common.select') }}
                    </button>
                  </div>
                  <span class="text-[10px] text-slate-400 mt-1">{{ t('tavern.certificateFileDesc') }}</span>
                </div>
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.sslKeyPath') }}</label>
                  <div class="flex gap-2">
                    <input type="text" v-model="tavernConfig.ssl.keyPath" class="form-input flex-1" :placeholder="t('tavern.sslKeyPathPlaceholder')" />
                    <button @click="selectKeyFile" 
                      class="px-4 py-2 bg-slate-100 dark:bg-slate-700 text-slate-700 dark:text-slate-300 rounded-xl hover:bg-blue-50 dark:hover:bg-blue-900/30 hover:text-blue-600 dark:hover:text-blue-400 border border-slate-200 dark:border-slate-600 transition-all active:scale-95 flex items-center gap-2">
                      <PhFile :size="16" weight="bold" />
                      {{ t('common.select') }}
                    </button>
                  </div>
                  <span class="text-[10px] text-slate-400 mt-1">{{ t('tavern.privateKeyFileDesc') }}</span>
                </div>
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.sslKeyPassphrase') }}</label>
                  <input type="password" v-model="tavernConfig.ssl.keyPassphrase" class="form-input" placeholder="••••••••" />
                  <span class="text-[10px] text-slate-400 mt-1">{{ t('tavern.keyPassphraseDesc') }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 4. 跨域配置 (CORS) -->
        <div class="config-card" :class="{ 'collapsed': collapsedSections.cors }">
          <div class="config-card-header" @click="toggleSection('cors')">
            <div class="header-left">
              <div class="header-icon bg-orange-100 dark:bg-orange-900/50 text-orange-600">
                <PhPlugsConnected :size="20" weight="duotone" />
              </div>
              <div class="header-info">
                <span class="title">{{ t('tavern.corsConfig') }}</span>
                <span class="desc">{{ t('tavern.corsConfigDesc') }}</span>
              </div>
            </div>
            <PhCaretDown :size="18" class="chevron" />
          </div>
          <div class="config-card-body">
            <div class="p-5 space-y-6">
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">{{ t('tavern.enableCors') }}</span>
                </div>
                <label class="toggle-switch">
                  <input type="checkbox" v-model="tavernConfig.cors.enabled" class="sr-only peer">
                  <div class="toggle-slider"></div>
                </label>
              </div>
              <div v-if="tavernConfig.cors.enabled" class="space-y-4 animate-in">
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.allowedOrigins') }}</label>
                  <div class="space-y-2">
                    <div v-for="(_origin, index) in tavernConfig.cors.origin" :key="index" 
                      class="flex items-center gap-2">
                      <input type="text" v-model="tavernConfig.cors.origin[index]" 
                        class="form-input flex-1" :placeholder="t('tavern.originPlaceholder')" />
                      <button @click="removeCorsOrigin(index)" 
                        class="px-3 py-2 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 transition-all active:scale-95">
                        <PhX :size="16" weight="bold" />
                      </button>
                    </div>
                    <button @click="addCorsOrigin" 
                      class="w-full py-2.5 border-2 border-dashed border-slate-300 dark:border-slate-600 rounded-xl text-slate-500 dark:text-slate-400 hover:border-blue-500 hover:text-blue-500 dark:hover:border-blue-400 dark:hover:text-blue-400 transition-all flex items-center justify-center gap-2">
                      <PhPlus :size="16" weight="bold" />
                      {{ t('common.add') }}
                    </button>
                  </div>
                </div>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div class="input-group">
                    <label class="input-label">{{ t('tavern.allowedMethods') }}</label>
                    <div class="space-y-2">
                      <div v-for="(_method, index) in tavernConfig.cors.methods" :key="index" 
                        class="flex items-center gap-2">
                        <input type="text" v-model="tavernConfig.cors.methods[index]" 
                          class="form-input flex-1" placeholder="GET, POST, PUT, DELETE, OPTIONS" />
                        <button @click="removeCorsMethod(index)" 
                          class="px-3 py-2 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 transition-all active:scale-95">
                          <PhX :size="16" weight="bold" />
                        </button>
                      </div>
                      <button @click="addCorsMethod" 
                        class="w-full py-2.5 border-2 border-dashed border-slate-300 dark:border-slate-600 rounded-xl text-slate-500 dark:text-slate-400 hover:border-blue-500 hover:text-blue-500 dark:hover:border-blue-400 dark:hover:text-blue-400 transition-all flex items-center justify-center gap-2">
                        <PhPlus :size="16" weight="bold" />
                        {{ t('common.add') }}
                      </button>
                    </div>
                  </div>
                  <div class="input-group">
                    <label class="input-label">{{ t('tavern.maxAge') }}</label>
                    <input type="number" v-model="tavernConfig.cors.maxAge" class="form-input" />
                  </div>
                </div>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div class="input-group">
                    <label class="input-label">{{ t('tavern.allowedHeaders') }}</label>
                    <div class="space-y-2">
                      <div v-for="(_header, index) in tavernConfig.cors.allowedHeaders" :key="index" 
                        class="flex items-center gap-2">
                        <input type="text" v-model="tavernConfig.cors.allowedHeaders[index]" 
                          class="form-input flex-1" :placeholder="t('tavern.headerPlaceholder')" />
                        <button @click="removeCorsHeader(index)" 
                          class="px-3 py-2 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 transition-all active:scale-95">
                          <PhX :size="16" weight="bold" />
                        </button>
                      </div>
                      <button @click="addCorsHeader" 
                        class="w-full py-2.5 border-2 border-dashed border-slate-300 dark:border-slate-600 rounded-xl text-slate-500 dark:text-slate-400 hover:border-blue-500 hover:text-blue-500 dark:hover:border-blue-400 dark:hover:text-blue-400 transition-all flex items-center justify-center gap-2">
                        <PhPlus :size="16" weight="bold" />
                        {{ t('common.add') }}
                      </button>
                    </div>
                  </div>
                  <div class="input-group">
                    <label class="input-label">{{ t('tavern.exposedHeaders') }}</label>
                    <div class="space-y-2">
                      <div v-for="(_header, index) in tavernConfig.cors.exposedHeaders" :key="index" 
                        class="flex items-center gap-2">
                        <input type="text" v-model="tavernConfig.cors.exposedHeaders[index]" 
                          class="form-input flex-1" :placeholder="t('tavern.exposedHeaderPlaceholder')" />
                        <button @click="removeCorsExposedHeader(index)" 
                          class="px-3 py-2 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 transition-all active:scale-95">
                          <PhX :size="16" weight="bold" />
                        </button>
                      </div>
                      <button @click="addCorsExposedHeader" 
                        class="w-full py-2.5 border-2 border-dashed border-slate-300 dark:border-slate-600 rounded-xl text-slate-500 dark:text-slate-400 hover:border-blue-500 hover:text-blue-500 dark:hover:border-blue-400 dark:hover:text-blue-400 transition-all flex items-center justify-center gap-2">
                        <PhPlus :size="16" weight="bold" />
                        {{ t('common.add') }}
                      </button>
                    </div>
                  </div>
                </div>
                <div class="flex gap-4">
                  <label class="checkbox-pill">
                    <input type="checkbox" v-model="tavernConfig.cors.credentials">
                    {{ t('tavern.allowCredentials') }}
                  </label>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 5. 代理与备份 -->
        <div class="config-card" :class="{ 'collapsed': collapsedSections.proxy }">
          <div class="config-card-header" @click="toggleSection('proxy')">
            <div class="header-left">
              <div class="header-icon bg-cyan-100 dark:bg-cyan-900/50 text-cyan-600">
                <PhLightning :size="20" weight="duotone" />
              </div>
              <div class="header-info">
                <span class="title">{{ t('tavern.proxyAndBackups') }}</span>
                <span class="desc">{{ t('tavern.proxyAndBackupsDesc') }}</span>
              </div>
            </div>
            <PhCaretDown :size="18" class="chevron" />
          </div>
          <div class="config-card-body">
            <!-- 代理 -->
            <div class="p-5 space-y-6">
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">{{ t('tavern.enableProxy') }}</span>
                  <span class="setting-desc">{{ t('tavern.enableProxyDesc') }}</span>
                </div>
                <label class="toggle-switch">
                  <input type="checkbox" v-model="tavernConfig.requestProxy.enabled" class="sr-only peer">
                  <div class="toggle-slider"></div>
                </label>
              </div>
              <div v-if="tavernConfig.requestProxy.enabled" class="space-y-4 animate-in">
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.proxyUrl') }}</label>
                  <input type="text" v-model="tavernConfig.requestProxy.url" class="form-input"
                    :placeholder="t('tavern.proxyUrlPlaceholder')" />
                </div>
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.bypassList') }}</label>
                  <div class="space-y-2">
                    <div v-for="(_bypass, index) in tavernConfig.requestProxy.bypass" :key="index" 
                      class="flex items-center gap-2">
                      <input type="text" v-model="tavernConfig.requestProxy.bypass[index]" 
                        class="form-input flex-1" :placeholder="t('tavern.bypassPlaceholder')" />
                      <button @click="removeBypass(index)" 
                        class="px-3 py-2 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 transition-all active:scale-95">
                        <PhX :size="16" weight="bold" />
                      </button>
                    </div>
                    <button @click="addBypass" 
                      class="w-full py-2.5 border-2 border-dashed border-slate-300 dark:border-slate-600 rounded-xl text-slate-500 dark:text-slate-400 hover:border-blue-500 hover:text-blue-500 dark:hover:border-blue-400 dark:hover:text-blue-400 transition-all flex items-center justify-center gap-2">
                      <PhPlus :size="16" weight="bold" />
                      {{ t('common.add') }}
                    </button>
                  </div>
                </div>
              </div>
            </div>

            <!-- 备份 -->
            <div class="p-5 border-t border-slate-100 dark:border-slate-700 space-y-4">
              <div class="input-group">
                <label class="input-label">{{ t('tavern.commonBackups') }}</label>
                <input type="number" v-model="tavernConfig.backups.common.numberOfBackups" class="form-input" />
              </div>
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">{{ t('tavern.enableChatBackup') }}</span>
                </div>
                <label class="toggle-switch">
                  <input type="checkbox" v-model="tavernConfig.backups.chat.enabled" class="sr-only peer">
                  <div class="toggle-slider"></div>
                </label>
              </div>
              <div v-if="tavernConfig.backups.chat.enabled" class="grid grid-cols-1 md:grid-cols-2 gap-4 animate-in">
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.maxTotalBackups') }}</label>
                  <input type="number" v-model="tavernConfig.backups.chat.maxTotalBackups" class="form-input" />
                </div>
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.throttleInterval') }}</label>
                  <input type="number" v-model="tavernConfig.backups.chat.throttleInterval" class="form-input" />
                </div>
              </div>
              <div v-if="tavernConfig.backups.chat.enabled" class="pt-2">
                <label class="checkbox-pill">
                  <input type="checkbox" v-model="tavernConfig.backups.chat.checkIntegrity">
                  {{ t('tavern.checkIntegrity') }}
                </label>
              </div>
            </div>
          </div>
        </div>

        <!-- 6. 缩略图设置 -->
        <div class="config-card" :class="{ 'collapsed': collapsedSections.thumbnail }">
          <div class="config-card-header" @click="toggleSection('thumbnail')">
            <div class="header-left">
              <div class="header-icon bg-pink-100 dark:bg-pink-900/50 text-pink-600">
                <PhImage :size="20" weight="duotone" />
              </div>
              <div class="header-info">
                <span class="title">{{ t('tavern.thumbnailSettings') }}</span>
                <span class="desc">{{ t('tavern.thumbnailSettingsDesc') }}</span>
              </div>
            </div>
            <PhCaretDown :size="18" class="chevron" />
          </div>
          <div class="config-card-body">
            <div class="p-5 space-y-6">
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">{{ t('tavern.enableThumbnails') }}</span>
                </div>
                <label class="toggle-switch">
                  <input type="checkbox" v-model="tavernConfig.thumbnails.enabled" class="sr-only peer">
                  <div class="toggle-slider"></div>
                </label>
              </div>
              <div v-if="tavernConfig.thumbnails.enabled" class="space-y-6 animate-in">
                <div class="grid grid-cols-2 gap-4">
                  <div class="input-group">
                    <label class="input-label">{{ t('tavern.imageFormat') }}</label>
                    <select v-model="tavernConfig.thumbnails.format"
                      class="form-input appearance-none bg-no-repeat bg-right pr-10"
                      style="background-image: url('data:image/svg+xml;charset=utf-8,%3Csvg xmlns=%22http%32.org/2000/svg%22 width=%2216%22 height=%2216%22 fill=%22%2394a3b8%22%3E%3Cpath d=%22M8 11L3 6h10l-5 5z%22/%3E%3C/svg%3E')">
                      <option value="jpg">JPEG ({{ t('tavern.default') }})</option>
                      <option value="png">PNG</option>
                      <option value="webp">WebP ({{ t('tavern.recommended') }})</option>
                    </select>
                  </div>
                  <div class="input-group">
                    <label class="input-label">{{ t('tavern.compressionQuality') }}</label>
                    <input type="number" v-model="tavernConfig.thumbnails.quality" min="1" max="100"
                      class="form-input" />
                  </div>
                </div>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div class="input-group">
                    <label class="input-label">{{ t('tavern.backgroundDimensions') }}</label>
                    <div class="flex gap-2">
                      <input type="number" v-model="tavernConfig.thumbnails.dimensions.bg[0]"
                        class="form-input flex-1 min-w-[60px]" :placeholder="t('tavern.width')" />
                      <input type="number" v-model="tavernConfig.thumbnails.dimensions.bg[1]"
                        class="form-input flex-1 min-w-[60px]" :placeholder="t('tavern.height')" />
                    </div>
                  </div>
                  <div class="input-group">
                    <label class="input-label">{{ t('tavern.avatarDimensions') }}</label>
                    <div class="flex gap-2">
                      <input type="number" v-model="tavernConfig.thumbnails.dimensions.avatar[0]"
                        class="form-input flex-1 min-w-[60px]" :placeholder="t('tavern.width')" />
                      <input type="number" v-model="tavernConfig.thumbnails.dimensions.avatar[1]"
                        class="form-input flex-1 min-w-[60px]" :placeholder="t('tavern.height')" />
                    </div>
                  </div>
                  <div class="input-group">
                    <label class="input-label">{{ t('tavern.personaDimensions') }}</label>
                    <div class="flex gap-2">
                      <input type="number" v-model="tavernConfig.thumbnails.dimensions.persona[0]"
                        class="form-input flex-1 min-w-[60px]" :placeholder="t('tavern.width')" />
                      <input type="number" v-model="tavernConfig.thumbnails.dimensions.persona[1]"
                        class="form-input flex-1 min-w-[60px]" :placeholder="t('tavern.height')" />
                    </div>
                  </div>
                </div>
              </div>
            </div>

          </div>
        </div>

        <!-- 7. 性能优化 -->
        <div class="config-card" :class="{ 'collapsed': collapsedSections.performance }">
          <div class="config-card-header" @click="toggleSection('performance')">
            <div class="header-left">
              <div class="header-icon bg-yellow-100 dark:bg-yellow-900/50 text-yellow-600">
                <PhCpu :size="20" weight="duotone" />
              </div>
              <div class="header-info">
                <span class="title">{{ t('tavern.loggingPerformance') }}</span>
                <span class="desc">{{ t('tavern.loggingPerformanceDesc') }}</span>
              </div>
            </div>
            <PhCaretDown :size="18" class="chevron" />
          </div>
          <div class="config-card-body">
            <div class="p-5 space-y-6">
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ t('tavern.lazyLoadCharacters') }}</span>
                    <span class="setting-desc">{{ t('tavern.lazyLoadCharactersDesc') }}</span>
                  </div>
                  <label class="toggle-switch">
                    <input type="checkbox" v-model="tavernConfig.performance.lazyLoadCharacters" class="sr-only peer">
                    <div class="toggle-slider"></div>
                  </label>
                </div>
                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ t('tavern.useDiskCache') }}</span>
                  </div>
                  <label class="toggle-switch">
                    <input type="checkbox" v-model="tavernConfig.performance.useDiskCache" class="sr-only peer">
                    <div class="toggle-slider"></div>
                  </label>
                </div>
              </div>
              <div class="input-group">
                <label class="input-label">{{ t('tavern.memoryCacheCapacity') }}</label>
                <input type="number" v-model="memoryCacheCapacityValue" class="form-input"
                  :placeholder="t('tavern.memoryCacheCapacityDesc')" min="0" />
                <span class="text-[10px] text-slate-400 mt-1">{{ t('tavern.mbMegabytes') }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 8. 日志与调试 -->
        <div class="config-card" :class="{ 'collapsed': collapsedSections.logging }">
          <div class="config-card-header" @click="toggleSection('logging')">
            <div class="header-left">
              <div class="header-icon bg-slate-100 dark:bg-slate-700 text-slate-600">
                <PhActivity :size="20" weight="duotone" />
              </div>
              <div class="header-info">
                <span class="title">{{ t('tavern.loggingPerformance') }}</span>
                <span class="desc">{{ t('tavern.loggingDesc') }}</span>
              </div>
            </div>
            <PhCaretDown :size="18" class="chevron" />
          </div>
          <div class="config-card-body">
            <div class="p-5 space-y-6">
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">{{ t('tavern.enableAccessLog') }}</span>
                </div>
                <label class="toggle-switch">
                  <input type="checkbox" v-model="tavernConfig.logging.enableAccessLog" class="sr-only peer">
                  <div class="toggle-slider"></div>
                </label>
              </div>
              <div class="input-group">
                <label class="input-label">{{ t('tavern.minLogLevel') }}</label>
                <select v-model="tavernConfig.logging.minLogLevel" class="form-input">
                  <option :value="0">{{ t('tavern.logLevelDebug') }}</option>
                  <option :value="1">{{ t('tavern.logLevelInfo') }}</option>
                  <option :value="2">{{ t('tavern.logLevelWarn') }}</option>
                  <option :value="3">{{ t('tavern.logLevelError') }}</option>
                </select>
              </div>
            </div>
          </div>
        </div>

        <!-- 9. 会话与安全 -->
        <div class="config-card" :class="{ 'collapsed': collapsedSections.other }">
          <div class="config-card-header" @click="toggleSection('other')">
            <div class="header-left">
              <div class="header-icon bg-red-100 dark:bg-red-900/50 text-red-600">
                <PhListNumbers :size="20" weight="duotone" />
              </div>
              <div class="header-info">
                <span class="title">{{ t('tavern.sessionSecurity') }}</span>
                <span class="desc">{{ t('tavern.sessionSecurityDesc') }}</span>
              </div>
            </div>
            <PhCaretDown :size="18" class="chevron" />
          </div>
          <div class="config-card-body">
            <div class="p-5 space-y-6">
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.promptPlaceholder') }}</label>
                  <input type="text" v-model="tavernConfig.promptPlaceholder" class="form-input" />
                  <span class="text-[10px] text-slate-400 mt-1">{{ t('tavern.promptPlaceholderDesc') }}</span>
                </div>
                <div class="input-group">
                  <label class="input-label">{{ t('tavern.sessionTimeout') }}</label>
                  <input type="number" v-model="tavernConfig.sessionTimeout" class="form-input" />
                  <span class="text-[10px] text-slate-400 mt-1">{{ t('tavern.sessionTimeoutDesc') }}</span>
                </div>
              </div>
        
              <div class="border-t border-slate-100 dark:border-slate-700 pt-4">
                <h3 class="text-sm font-bold text-slate-700 dark:text-slate-300 mb-3">{{ t('tavern.securityOptions') }}</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                  <label class="checkbox-pill">
                    <input type="checkbox" v-model="tavernConfig.disableCsrfProtection">
                    {{ t('tavern.disableCsrfProtection') }}
                  </label>
                  <label class="checkbox-pill">
                    <input type="checkbox" v-model="tavernConfig.securityOverride">
                    {{ t('tavern.securityOverride') }}
                  </label>
                  <label class="checkbox-pill">
                    <input type="checkbox" v-model="tavernConfig.allowKeysExposure">
                    {{ t('tavern.allowKeysExposure') }}
                  </label>
                  <label class="checkbox-pill">
                    <input type="checkbox" v-model="tavernConfig.skipContentCheck">
                    {{ t('tavern.skipContentCheck') }}
                  </label>
                </div>
              </div>
        
              <div class="border-t border-slate-100 dark:border-slate-700 pt-4">
                <h3 class="text-sm font-bold text-slate-700 dark:text-slate-300 mb-3">{{ t('tavern.extensionsPlugins') }}</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div class="setting-row">
                    <div class="setting-info">
                      <span class="setting-label">{{ t('tavern.extensionsEnabled') }}</span>
                    </div>
                    <label class="toggle-switch">
                      <input type="checkbox" v-model="tavernConfig.extensions.enabled" class="sr-only peer">
                      <div class="toggle-slider"></div>
                    </label>
                  </div>
                  <div class="setting-row">
                    <div class="setting-info">
                      <span class="setting-label">{{ t('tavern.extensionsAutoUpdate') }}</span>
                    </div>
                    <label class="toggle-switch">
                      <input type="checkbox" v-model="tavernConfig.extensions.autoUpdate" class="sr-only peer">
                      <div class="toggle-slider"></div>
                    </label>
                  </div>
                </div>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mt-3">
                  <label class="checkbox-pill">
                    <input type="checkbox" v-model="tavernConfig.enableServerPlugins">
                    {{ t('tavern.enableServerPlugins') }}
                  </label>
                  <label class="checkbox-pill">
                    <input type="checkbox" v-model="tavernConfig.enableServerPluginsAutoUpdate">
                    {{ t('tavern.enableServerPluginsAutoUpdate') }}
                  </label>
                </div>
              </div>
        
              <div class="border-t border-slate-100 dark:border-slate-700 pt-4">
                <h3 class="text-sm font-bold text-slate-700 dark:text-slate-300 mb-3">{{ t('tavern.ssoConfig') }}</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                  <label class="checkbox-pill">
                    <input type="checkbox" v-model="tavernConfig.sso.autheliaAuth">
                    Authelia SSO
                  </label>
                  <label class="checkbox-pill">
                    <input type="checkbox" v-model="tavernConfig.sso.authentikAuth">
                    Authentik SSO
                  </label>
                </div>
              </div>
        
              <div class="border-t border-slate-100 dark:border-slate-700 pt-4">
                <h3 class="text-sm font-bold text-slate-700 dark:text-slate-300 mb-3">{{ t('tavern.cacheBuster') }}</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div class="setting-row">
                    <div class="setting-info">
                      <span class="setting-label">{{ t('tavern.cacheBusterEnabled') }}</span>
                    </div>
                    <label class="toggle-switch">
                      <input type="checkbox" v-model="tavernConfig.cacheBuster.enabled" class="sr-only peer">
                      <div class="toggle-slider"></div>
                    </label>
                  </div>
                  <div class="input-group">
                    <label class="input-label">{{ t('tavern.cacheBusterPattern') }}</label>
                    <input type="text" v-model="tavernConfig.cacheBuster.userAgentPattern" class="form-input" :placeholder="t('tavern.cacheBusterPatternDesc')" />
                  </div>
                </div>
              </div>
        
              <div class="border-t border-slate-100 dark:border-slate-700 pt-4">
                <h3 class="text-sm font-bold text-slate-700 dark:text-slate-300 mb-3">{{ t('tavern.otherSettings') }}</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                  <label class="checkbox-pill">
                    <input type="checkbox" v-model="tavernConfig.enableCorsProxy">
                    {{ t('tavern.enableCorsProxy') }}
                  </label>
                  <label class="checkbox-pill">
                    <input type="checkbox" v-model="tavernConfig.enableDownloadableTokenizers">
                    {{ t('tavern.enableDownloadableTokenizers') }}
                  </label>
                </div>
              </div>
            </div>
          </div>
        </div>

      </div>
    </main>
  </div>

  <!-- 配置迁移弹窗 -->
  <ConfigMigrateDialog
    :open="showMigrateDialog"
    @close="showMigrateDialog = false"
    @migrated="loadConfig"
  />
</template>

<style>
@reference "../style.css";

.config-card {
  @apply bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl overflow-hidden transition-all duration-300 shadow-sm;
}

.config-card-header {
  @apply flex items-center justify-between p-4 cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-700/50 transition-colors select-none;
}

.header-left {
  @apply flex items-center gap-4;
}

.header-icon {
  @apply w-10 h-10 rounded-xl flex items-center justify-center;
}

.header-info {
  @apply flex flex-col;
}

.header-info .title {
  @apply font-bold text-slate-800 dark:text-slate-100 text-[15px] tracking-tight;
}

.header-info .desc {
  @apply text-[11px] text-slate-500 dark:text-slate-400 mt-0.5;
}

.chevron {
  @apply transition-transform duration-300 text-slate-400;
}

.collapsed .chevron {
  @apply -rotate-90;
}

.config-card-body {
  @apply border-t border-slate-100 dark:border-slate-700/50 transition-all duration-300 max-h-[5000px] opacity-100;
}

.collapsed .config-card-body {
  @apply max-h-0 opacity-0 border-transparent overflow-hidden;
}

.input-group {
  @apply flex flex-col gap-1.5;
}

.input-label {
  @apply text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider ml-1;
}

.form-input {
  @apply px-4 py-2.5 bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl text-sm outline-none focus:ring-4 focus:ring-blue-500/10 focus:border-blue-500/50 transition-all dark:text-slate-200;
}

.setting-row {
  @apply flex items-center justify-between gap-4 p-4 bg-slate-50/50 dark:bg-slate-900/30 rounded-2xl border border-slate-100 dark:border-slate-700/50;
}

.setting-info {
  @apply flex flex-col gap-0.5;
}

.setting-label {
  @apply text-sm font-bold text-slate-700 dark:text-slate-200;
}

.setting-desc {
  @apply text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed;
}

/* 开关组件 */
.toggle-switch {
  @apply relative inline-flex items-center cursor-pointer shrink-0;
}

.toggle-switch input {
  @apply absolute opacity-0 w-0 h-0;
}

.toggle-slider {
  @apply w-11 h-6 bg-slate-200 dark:bg-slate-700 rounded-full peer-checked:bg-blue-600 transition-all after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-5 shadow-sm;
}

/* 复选框药丸 */
.checkbox-pill {
  @apply flex items-center gap-2 px-3 py-2 bg-slate-100 dark:bg-slate-800 rounded-xl cursor-pointer text-[12px] font-bold text-slate-500 dark:text-slate-400 border border-transparent transition-all active:scale-95 select-none;
}

.checkbox-pill:has(input:checked) {
  @apply bg-blue-600 text-white shadow-lg shadow-blue-500/20;
}

.checkbox-pill input {
  @apply hidden;
}

.animate-in {
  animation: slideDown 0.2s ease-out forwards;
}

@keyframes slideDown {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

::-webkit-scrollbar {
  width: 5px;
}

::-webkit-scrollbar-thumb {
  @apply bg-slate-300 dark:bg-slate-700 rounded-full;
}
</style>
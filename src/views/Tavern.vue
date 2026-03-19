<script setup lang="ts">
import { ref, onBeforeUnmount, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'vue-sonner';
import { PhFileCode, PhGlobe, PhLockKey, PhBrowser, PhListNumbers } from '@phosphor-icons/vue';

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
    loading.value = true;
    configError.value = null;
    isInitialLoad.value = true;

    // Get current version
    const appConfig = await invoke<any>('get_app_config');
    version.value = appConfig.sillytavern?.version || '';

    if (!version.value) {
      configError.value = '未选择酒馆版本，请先在"版本"页面选择或安装一个版本。';
      return;
    }

    tavernConfig.value = await invoke<TavernConfigPayload>('get_sillytavern_config_options', {
      version: version.value
    });
    isInitialLoad.value = false;

  } catch (error: any) {
    console.error('Failed to load tavern config:', error);
    configError.value = typeof error === 'string' ? error : '加载配置文件失败，可能是版本未安装或配置丢失。';
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
    toast.error('保存配置失败');
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
    toast.error('无法打开配置文件');
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
        <h1 class="text-2xl font-bold">酒馆配置</h1>
        <p class="text-slate-500 text-sm mt-1">管理当前版本 ({{ version || '未选择' }}) 的 config.yaml 选项</p>
      </div>
      <button
        @click="openConfigFile"
        class="flex items-center gap-2 px-4 py-2 bg-white border border-slate-200 text-slate-700 rounded-lg hover:bg-slate-50 hover:text-blue-600 transition-colors shadow-sm text-sm font-medium"
        :disabled="!version || !!configError"
        :class="{ 'opacity-50 cursor-not-allowed': !version || !!configError }"
      >
        <PhFileCode :size="18" weight="duotone" />
        打开配置文件
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto px-1 pb-10 min-h-0 relative">
      
      <!-- Loading State -->
      <div v-if="loading" class="flex flex-col items-center justify-center py-20">
        <div class="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin mb-4"></div>
        <p class="text-sm font-medium text-slate-500">正在读取配置文件...</p>
      </div>

      <!-- Error State -->
      <div v-else-if="configError" class="bg-red-50 text-red-600 p-4 rounded-xl border border-red-100 flex items-start gap-3">
        <div class="mt-0.5">⚠️</div>
        <div>
          <h3 class="font-semibold">无法加载配置</h3>
          <p class="text-sm opacity-90 mt-1">{{ configError }}</p>
        </div>
      </div>

      <!-- Settings Form -->
      <div v-else class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
        
        <!-- Network Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
            <PhGlobe :size="20" class="text-blue-500" weight="duotone" />
            网络与访问
          </h2>
          
          <div class="bg-white rounded-xl border border-slate-200 p-4 space-y-4 shadow-sm">
            <!-- Port -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-blue-50 flex items-center justify-center text-blue-500">
                  <PhListNumbers :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700">服务端口</div>
                  <div class="text-xs text-slate-500">酒馆运行的本地端口号 (默认: 8000)</div>
                </div>
              </div>
              <input 
                type="number" 
                v-model="tavernConfig.port"
                class="bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 w-24 outline-none transition-all"
              />
            </div>

            <div class="w-full h-px bg-slate-100"></div>

            <!-- Listen (Host) -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-green-50 flex items-center justify-center text-green-500">
                  <PhGlobe :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700">允许局域网访问 (Listen)</div>
                  <div class="text-xs text-slate-500">开启后局域网内的其他设备可以访问酒馆</div>
                </div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.listen" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
              </label>
            </div>

            <div class="w-full h-px bg-slate-100"></div>

            <div class="space-y-3">
              <div class="font-medium text-slate-700 text-sm">监听地址</div>
              <div class="flex items-center gap-2">
                <span class="text-xs text-slate-500 w-14">IPv4</span>
                <input
                  type="text"
                  v-model="tavernConfig.listenAddress.ipv4"
                  class="flex-1 bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2 outline-none transition-all"
                />
              </div>
              <div class="flex items-center gap-2">
                <span class="text-xs text-slate-500 w-14">IPv6</span>
                <input
                  type="text"
                  v-model="tavernConfig.listenAddress.ipv6"
                  class="flex-1 bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2 outline-none transition-all"
                />
              </div>
            </div>

            <div class="w-full h-px bg-slate-100"></div>

            <div class="space-y-3">
              <div class="font-medium text-slate-700 text-sm">协议开关</div>
              <div class="flex items-center justify-between">
                <div class="text-sm text-slate-600">启用 IPv4</div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" v-model="tavernConfig.protocol.ipv4" class="sr-only peer">
                  <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
                </label>
              </div>
              <div class="flex items-center justify-between">
                <div class="text-sm text-slate-600">启用 IPv6</div>
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
          <h2 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
            <PhLockKey :size="20" class="text-purple-500" weight="duotone" />
            安全白名单
          </h2>
          
          <div class="bg-white rounded-xl border border-slate-200 p-4 space-y-4 shadow-sm">
            <!-- Whitelist Mode -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-purple-50 flex items-center justify-center text-purple-500">
                  <PhLockKey :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700">启用白名单</div>
                  <div class="text-xs text-slate-500">仅允许白名单内的IP地址访问</div>
                </div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.whitelistMode" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-purple-500"></div>
              </label>
            </div>

            <div class="w-full h-px bg-slate-100"></div>

            <!-- Whitelist IPs -->
            <div class="space-y-3" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.whitelistMode }">
              <div class="flex items-center justify-between">
                <div class="font-medium text-slate-700 text-sm">白名单 IP 列表</div>
                <button 
                  @click="addWhitelistItem"
                  class="text-xs text-purple-600 hover:text-purple-700 font-medium px-2 py-1 bg-purple-50 rounded-md transition-colors"
                >
                  + 添加 IP
                </button>
              </div>
              
              <div v-for="(_, index) in tavernConfig.whitelist" :key="index" class="flex items-center gap-2">
                <input 
                  type="text" 
                  v-model="tavernConfig.whitelist[index]"
                  placeholder="例如: 192.168.1.100"
                  class="flex-1 bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-purple-500 focus:border-purple-500 block p-2 outline-none transition-all"
                />
                <button 
                  @click="removeWhitelistItem(index)"
                  class="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors"
                  title="移除"
                >
                  ✕
                </button>
              </div>
              <div v-if="tavernConfig.whitelist.length === 0" class="text-xs text-slate-400 italic">
                列表为空，请添加允许的IP地址
              </div>
            </div>
          </div>
        </section>

        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
            <PhLockKey :size="20" class="text-indigo-500" weight="duotone" />
            基础认证
          </h2>

          <div class="bg-white rounded-xl border border-slate-200 p-4 space-y-3 shadow-sm">
            <div class="flex items-center justify-between">
              <div>
                <div class="font-medium text-slate-700">启用基础认证</div>
                <div class="text-xs text-slate-500">开启后访问需要用户名和密码</div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.basicAuthMode" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500"></div>
              </label>
            </div>
            <div class="flex items-center justify-between">
              <div>
                <div class="font-medium text-slate-700">启用用户账户</div>
                <div class="text-xs text-slate-500">开启后支持独立用户登录</div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.enableUserAccounts" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500"></div>
              </label>
            </div>
            <div class="flex items-center justify-between" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.enableUserAccounts }">
              <div>
                <div class="font-medium text-slate-700">低调登录模式</div>
                <div class="text-xs text-slate-500">隐藏显式登录入口</div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.enableDiscreetLogin" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500"></div>
              </label>
            </div>
            <div class="flex items-center justify-between" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.enableUserAccounts }">
              <div>
                <div class="font-medium text-slate-700">按用户基础认证</div>
                <div class="text-xs text-slate-500">每个用户使用独立基础认证</div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.perUserBasicAuth" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-500"></div>
              </label>
            </div>
            <div class="w-full h-px bg-slate-100"></div>
            <div class="space-y-2" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.basicAuthMode }">
              <div class="font-medium text-slate-700 text-sm">用户名</div>
              <input
                type="text"
                v-model="tavernConfig.basicAuthUser.username"
                class="w-full bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-indigo-500 focus:border-indigo-500 block p-2 outline-none transition-all"
              />
            </div>
            <div class="space-y-2" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.basicAuthMode }">
              <div class="font-medium text-slate-700 text-sm">密码</div>
              <input
                type="text"
                v-model="tavernConfig.basicAuthUser.password"
                class="w-full bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-indigo-500 focus:border-indigo-500 block p-2 outline-none transition-all"
              />
            </div>
          </div>
        </section>

        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
            <PhGlobe :size="20" class="text-cyan-500" weight="duotone" />
            CORS 配置
          </h2>

          <div class="bg-white rounded-xl border border-slate-200 p-4 space-y-4 shadow-sm">
            <div class="flex items-center justify-between">
              <div>
                <div class="font-medium text-slate-700">启用 CORS</div>
                <div class="text-xs text-slate-500">控制跨域请求来源</div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.cors.enabled" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-cyan-500"></div>
              </label>
            </div>

            <div class="w-full h-px bg-slate-100"></div>

            <div class="space-y-3" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.cors.enabled }">
              <div class="flex items-center justify-between">
                <div class="font-medium text-slate-700 text-sm">允许来源 Origin</div>
                <button
                  @click="addCorsOrigin"
                  class="text-xs text-cyan-600 hover:text-cyan-700 font-medium px-2 py-1 bg-cyan-50 rounded-md transition-colors"
                >
                  + 添加来源
                </button>
              </div>

              <div v-for="(_, index) in tavernConfig.cors.origin" :key="`cors-${index}`" class="flex items-center gap-2">
                <input
                  type="text"
                  v-model="tavernConfig.cors.origin[index]"
                  placeholder="例如: https://example.com"
                  class="flex-1 bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-cyan-500 focus:border-cyan-500 block p-2 outline-none transition-all"
                />
                <button
                  @click="removeCorsOrigin(index)"
                  class="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors"
                  title="移除"
                >
                  ✕
                </button>
              </div>

              <div class="w-full h-px bg-slate-100"></div>

              <div class="flex items-center justify-between">
                <div class="font-medium text-slate-700 text-sm">允许方法 Methods</div>
                <button
                  @click="addCorsMethod"
                  class="text-xs text-cyan-600 hover:text-cyan-700 font-medium px-2 py-1 bg-cyan-50 rounded-md transition-colors"
                >
                  + 添加方法
                </button>
              </div>

              <div v-for="(_, index) in tavernConfig.cors.methods" :key="`cors-method-${index}`" class="flex items-center gap-2">
                <input
                  type="text"
                  v-model="tavernConfig.cors.methods[index]"
                  placeholder="例如: OPTIONS"
                  class="flex-1 bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-cyan-500 focus:border-cyan-500 block p-2 outline-none transition-all"
                />
                <button
                  @click="removeCorsMethod(index)"
                  class="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors"
                  title="移除"
                >
                  ✕
                </button>
              </div>

              <div class="w-full h-px bg-slate-100"></div>

              <div class="flex items-center justify-between">
                <div class="font-medium text-slate-700 text-sm">允许请求头 Allowed Headers</div>
                <button
                  @click="addCorsAllowedHeader"
                  class="text-xs text-cyan-600 hover:text-cyan-700 font-medium px-2 py-1 bg-cyan-50 rounded-md transition-colors"
                >
                  + 添加请求头
                </button>
              </div>

              <div v-for="(_, index) in tavernConfig.cors.allowedHeaders" :key="`cors-allowed-${index}`" class="flex items-center gap-2">
                <input
                  type="text"
                  v-model="tavernConfig.cors.allowedHeaders[index]"
                  placeholder="例如: Content-Type"
                  class="flex-1 bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-cyan-500 focus:border-cyan-500 block p-2 outline-none transition-all"
                />
                <button
                  @click="removeCorsAllowedHeader(index)"
                  class="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors"
                  title="移除"
                >
                  ✕
                </button>
              </div>

              <div class="w-full h-px bg-slate-100"></div>

              <div class="flex items-center justify-between">
                <div class="font-medium text-slate-700 text-sm">暴露响应头 Exposed Headers</div>
                <button
                  @click="addCorsExposedHeader"
                  class="text-xs text-cyan-600 hover:text-cyan-700 font-medium px-2 py-1 bg-cyan-50 rounded-md transition-colors"
                >
                  + 添加响应头
                </button>
              </div>

              <div v-for="(_, index) in tavernConfig.cors.exposedHeaders" :key="`cors-exposed-${index}`" class="flex items-center gap-2">
                <input
                  type="text"
                  v-model="tavernConfig.cors.exposedHeaders[index]"
                  placeholder="例如: X-Trace-Id"
                  class="flex-1 bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-cyan-500 focus:border-cyan-500 block p-2 outline-none transition-all"
                />
                <button
                  @click="removeCorsExposedHeader(index)"
                  class="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors"
                  title="移除"
                >
                  ✕
                </button>
              </div>

              <div class="w-full h-px bg-slate-100"></div>

              <div class="flex items-center justify-between">
                <div class="text-sm text-slate-600">允许携带凭证 Credentials</div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" v-model="tavernConfig.cors.credentials" class="sr-only peer">
                  <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-cyan-500"></div>
                </label>
              </div>

              <div class="space-y-2">
                <div class="font-medium text-slate-700 text-sm">缓存时间 Max Age (留空为 null)</div>
                <input
                  type="number"
                  :value="tavernConfig.cors.maxAge ?? ''"
                  @input="onCorsMaxAgeInput"
                  class="w-full bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-cyan-500 focus:border-cyan-500 block p-2 outline-none transition-all"
                />
              </div>
            </div>
          </div>
        </section>

        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
            <PhGlobe :size="20" class="text-rose-500" weight="duotone" />
            请求代理
          </h2>

          <div class="bg-white rounded-xl border border-slate-200 p-4 space-y-4 shadow-sm">
            <div class="flex items-center justify-between">
              <div>
                <div class="font-medium text-slate-700">启用 Request Proxy</div>
                <div class="text-xs text-slate-500">为外部请求设置统一代理</div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.requestProxy.enabled" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-rose-500"></div>
              </label>
            </div>

            <div class="w-full h-px bg-slate-100"></div>

            <div class="space-y-3" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.requestProxy.enabled }">
              <div class="space-y-2">
                <div class="font-medium text-slate-700 text-sm">代理地址 URL</div>
                <input
                  type="text"
                  v-model="tavernConfig.requestProxy.url"
                  placeholder="例如: socks5://127.0.0.1:1080"
                  class="w-full bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-rose-500 focus:border-rose-500 block p-2 outline-none transition-all"
                />
              </div>

              <div class="flex items-center justify-between">
                <div class="font-medium text-slate-700 text-sm">绕过代理地址</div>
                <button
                  @click="addRequestProxyBypass"
                  class="text-xs text-rose-600 hover:text-rose-700 font-medium px-2 py-1 bg-rose-50 rounded-md transition-colors"
                >
                  + 添加地址
                </button>
              </div>

              <div v-for="(_, index) in tavernConfig.requestProxy.bypass" :key="`proxy-${index}`" class="flex items-center gap-2">
                <input
                  type="text"
                  v-model="tavernConfig.requestProxy.bypass[index]"
                  placeholder="例如: localhost"
                  class="flex-1 bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-rose-500 focus:border-rose-500 block p-2 outline-none transition-all"
                />
                <button
                  @click="removeRequestProxyBypass(index)"
                  class="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors"
                  title="移除"
                >
                  ✕
                </button>
              </div>
            </div>
          </div>
        </section>

        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
            <PhBrowser :size="20" class="text-emerald-500" weight="duotone" />
            备份配置
          </h2>

          <div class="bg-white rounded-xl border border-slate-200 p-4 space-y-4 shadow-sm">
            <div class="space-y-2">
              <div class="font-medium text-slate-700 text-sm">通用备份数量</div>
              <input
                type="number"
                v-model.number="tavernConfig.backups.common.numberOfBackups"
                class="w-full bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-emerald-500 focus:border-emerald-500 block p-2 outline-none transition-all"
              />
            </div>

            <div class="w-full h-px bg-slate-100"></div>

            <div class="flex items-center justify-between">
              <div class="text-sm text-slate-600">启用聊天备份</div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.backups.chat.enabled" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500"></div>
              </label>
            </div>

            <div class="flex items-center justify-between" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.backups.chat.enabled }">
              <div class="text-sm text-slate-600">启用完整性校验</div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.backups.chat.checkIntegrity" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500"></div>
              </label>
            </div>

            <div class="grid grid-cols-2 gap-3" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.backups.chat.enabled }">
              <div class="space-y-2">
                <div class="font-medium text-slate-700 text-sm">聊天最大备份数</div>
                <input
                  type="number"
                  v-model.number="tavernConfig.backups.chat.maxTotalBackups"
                  class="w-full bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-emerald-500 focus:border-emerald-500 block p-2 outline-none transition-all"
                />
              </div>
              <div class="space-y-2">
                <div class="font-medium text-slate-700 text-sm">节流间隔 (ms)</div>
                <input
                  type="number"
                  v-model.number="tavernConfig.backups.chat.throttleInterval"
                  class="w-full bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-emerald-500 focus:border-emerald-500 block p-2 outline-none transition-all"
                />
              </div>
            </div>
          </div>
        </section>

        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
            <PhBrowser :size="20" class="text-teal-500" weight="duotone" />
            缩略图配置
          </h2>

          <div class="bg-white rounded-xl border border-slate-200 p-4 space-y-4 shadow-sm">
            <div class="flex items-center justify-between">
              <div class="text-sm text-slate-600">启用缩略图</div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.thumbnails.enabled" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-teal-500"></div>
              </label>
            </div>

            <div class="grid grid-cols-2 gap-3" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.thumbnails.enabled }">
              <div class="space-y-2">
                <div class="font-medium text-slate-700 text-sm">格式</div>
                <input
                  type="text"
                  v-model="tavernConfig.thumbnails.format"
                  class="w-full bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-teal-500 focus:border-teal-500 block p-2 outline-none transition-all"
                />
              </div>
              <div class="space-y-2">
                <div class="font-medium text-slate-700 text-sm">质量</div>
                <input
                  type="number"
                  v-model.number="tavernConfig.thumbnails.quality"
                  class="w-full bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-teal-500 focus:border-teal-500 block p-2 outline-none transition-all"
                />
              </div>
            </div>

            <div class="w-full h-px bg-slate-100"></div>

            <div class="space-y-3" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.thumbnails.enabled }">
              <div class="font-medium text-slate-700 text-sm">尺寸 (宽 x 高)</div>
              <div class="grid grid-cols-5 gap-2 items-center">
                <div class="text-sm text-slate-500">背景</div>
                <input type="number" v-model.number="tavernConfig.thumbnails.dimensions.bg[0]" class="bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg p-2 outline-none transition-all focus:ring-teal-500 focus:border-teal-500" />
                <div class="text-center text-slate-400">x</div>
                <input type="number" v-model.number="tavernConfig.thumbnails.dimensions.bg[1]" class="bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg p-2 outline-none transition-all focus:ring-teal-500 focus:border-teal-500" />
                <div></div>
              </div>
              <div class="grid grid-cols-5 gap-2 items-center">
                <div class="text-sm text-slate-500">头像</div>
                <input type="number" v-model.number="tavernConfig.thumbnails.dimensions.avatar[0]" class="bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg p-2 outline-none transition-all focus:ring-teal-500 focus:border-teal-500" />
                <div class="text-center text-slate-400">x</div>
                <input type="number" v-model.number="tavernConfig.thumbnails.dimensions.avatar[1]" class="bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg p-2 outline-none transition-all focus:ring-teal-500 focus:border-teal-500" />
                <div></div>
              </div>
              <div class="grid grid-cols-5 gap-2 items-center">
                <div class="text-sm text-slate-500">人设</div>
                <input type="number" v-model.number="tavernConfig.thumbnails.dimensions.persona[0]" class="bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg p-2 outline-none transition-all focus:ring-teal-500 focus:border-teal-500" />
                <div class="text-center text-slate-400">x</div>
                <input type="number" v-model.number="tavernConfig.thumbnails.dimensions.persona[1]" class="bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg p-2 outline-none transition-all focus:ring-teal-500 focus:border-teal-500" />
                <div></div>
              </div>
            </div>
          </div>
        </section>

        <!-- Browser Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
            <PhBrowser :size="20" class="text-orange-500" weight="duotone" />
            浏览器启动
          </h2>
          
          <div class="bg-white rounded-xl border border-slate-200 p-4 space-y-4 shadow-sm">
            <!-- Auto run -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-orange-50 flex items-center justify-center text-orange-500">
                  <PhBrowser :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700">启动时自动打开</div>
                  <div class="text-xs text-slate-500">启动酒馆后自动在浏览器中打开页面</div>
                </div>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="tavernConfig.browserLaunchEnabled" class="sr-only peer">
                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-orange-500"></div>
              </label>
            </div>

            <div class="w-full h-px bg-slate-100"></div>

            <!-- Browser Type -->
            <div class="flex items-center justify-between" :class="{ 'opacity-50 pointer-events-none': !tavernConfig.browserLaunchEnabled }">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-slate-50 flex items-center justify-center text-slate-500">
                  <PhBrowser :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700">浏览器选择</div>
                  <div class="text-xs text-slate-500">选择要使用的浏览器</div>
                </div>
              </div>
              <select 
                v-model="tavernConfig.browserType" 
                class="bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-orange-500 focus:border-orange-500 block p-2.5 min-w-[120px] outline-none transition-all"
              >
                <option value="default">系统默认</option>
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

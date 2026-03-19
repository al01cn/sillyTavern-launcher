<script lang="ts">
// Global promise to prevent concurrent loads across component instances
let globalLoadConfigPromise: Promise<void> | null = null;
</script>

<script setup lang="ts">
import { ref, onMounted, watch, computed, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { toast } from 'vue-sonner';
import { PhCheck, PhArrowsClockwise, PhGlobe, PhPalette, PhGithubLogo, PhInfo, PhPackage, PhDownloadSimple } from '@phosphor-icons/vue';

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
  theme: string;
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

const npmRegistries: NpmRegistry[] = [
  { name: '官方源', url: 'https://registry.npmjs.org/' },
  { name: '淘宝源', url: 'https://registry.npmmirror.com/' },
  { name: '腾讯源', url: 'https://mirrors.cloud.tencent.com/npm/' },
  { name: '华为源', url: 'https://repo.huaweicloud.com/repository/npm/' }
];

const config = ref<AppConfig>({
  lang: 'zh-CN',
  theme: 'light',
  rememberWindowPosition: false,
  githubProxy: {
    enable: false,
    url: 'https://gh.llkk.cc'
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
      const resStr = JSON.stringify(res);

      if (resStr !== cachedConfig) {
        // 如果接口数据与缓存不一致，更新缓存并静默刷新界面
        // 注意：由于下面有一个 watch(config, ...) 自动保存逻辑
        // 我们需要一种机制避免初次读取后触发无意义的保存，
        // 这里 loading.value 状态可以用于控制 watch，我们暂时保持为 true
        loading.value = true; 
        config.value = res;
        localStorage.setItem('app_settings_config_cache', resStr);
      }

      // 等待 DOM 更新后解除 loading 状态，这样 watch 里如果判断 loading=false 才会去保存
      await nextTick();
    } catch (error) {
      console.error('Failed to load config:', error);
      toast.error('加载配置失败，将使用默认配置');
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
        } catch(e) {}
    }
    localStorage.setItem('app_settings_config_cache', JSON.stringify(mergedConfig));
    
    // toast.success('设置已保存'); // Remove toast for real-time save to avoid spam
    console.log('Config saved');
  } catch (error) {
    console.error('Failed to save config:', error);
    toast.error('保存配置失败');
  }
};

const fetchProxies = async () => {
  try {
    proxyLoading.value = true;
    const res = await invoke<ProxyItem[]>('fetch_github_proxies');
    proxies.value = res.sort((a, b) => a.latency - b.latency);
    // toast.success('获取加速列表成功'); // Remove toast on auto-fetch
  } catch (error) {
    console.error('Failed to fetch proxies:', error);
    toast.error('获取加速列表失败');
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
    const res = await invoke<NodeInfo>('check_nodejs');
    nodeInfo.value = res;
  } catch (error) {
    console.error('Failed to check nodejs:', error);
  }
};

const checkNpm = async () => {
  try {
    const res = await invoke<NpmInfo>('check_npm');
    npmInfo.value = res;
  } catch (error) {
    console.error('Failed to check npm:', error);
  }
};

const installNode = async () => {
  if (installingNode.value) return;
  installingNode.value = true;
  nodeProgress.value = { status: 'starting', progress: 0, log: '准备安装...' };
  
  try {
    await invoke('install_nodejs');
    toast.success('Node.js 安装成功');
    await checkNode();
    await checkNpm();
  } catch (error) {
    console.error('Failed to install nodejs:', error);
    toast.error('安装失败: ' + error);
  } finally {
    installingNode.value = false;
  }
};

// Watch for config changes and save automatically
watch(config, () => {
  if (!loading.value) {
    saveConfig();
  }
}, { deep: true });

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
</script>

<template>
  <div class="flex flex-col h-full">
    <h1 class="text-2xl font-bold mb-6 px-1">设置</h1>

    <!-- Tabs -->
    <div class="flex space-x-1 bg-slate-100 p-1 rounded-xl w-fit mb-6 shrink-0">
      <button
        @click="activeTab = 'general'"
        :class="[
          'px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 flex items-center gap-2',
          activeTab === 'general' 
            ? 'bg-white text-slate-900 shadow-sm' 
            : 'text-slate-500 hover:text-slate-700 hover:bg-slate-200/50'
        ]"
      >
        <PhPalette :size="16" weight="duotone" />
        一般设置
      </button>
      <button
        @click="activeTab = 'about'"
        :class="[
          'px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 flex items-center gap-2',
          activeTab === 'about' 
            ? 'bg-white text-slate-900 shadow-sm' 
            : 'text-slate-500 hover:text-slate-700 hover:bg-slate-200/50'
        ]"
      >
        <PhInfo :size="16" weight="duotone" />
        关于
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto px-1 pb-10 min-h-0 relative">
      
      <!-- Loading State -->
      <div v-if="loading" class="absolute inset-0 flex flex-col items-center justify-center bg-slate-50/50 backdrop-blur-sm z-10">
        <PhArrowsClockwise :size="48" class="animate-spin mb-4 text-blue-500/80" weight="duotone" />
        <p class="text-sm font-medium text-slate-500 animate-pulse">正在加载配置...</p>
      </div>

      <!-- General Settings -->
      <div v-if="activeTab === 'general'" class="space-y-8 animate-in fade-in slide-in-from-bottom-2 duration-300">
        
        <!-- Interface Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
            <PhPalette :size="20" class="text-blue-500" weight="duotone" />
            界面设置
          </h2>
          
          <div class="bg-white rounded-xl border border-slate-200 p-4 space-y-4 shadow-sm">
            <!-- Language -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-blue-50 flex items-center justify-center text-blue-500">
                  <PhGlobe :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700">界面语言</div>
                  <div class="text-xs text-slate-500">选择应用程序显示的语言</div>
                </div>
              </div>
              <select 
                v-model="config.lang" 
                class="bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 min-w-[120px] outline-none transition-all"
              >
                <option value="zh-CN">简体中文</option>
                <option value="en-US">English</option>
              </select>
            </div>

            <div class="w-full h-px bg-slate-100"></div>

            <!-- Theme -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-purple-50 flex items-center justify-center text-purple-500">
                  <PhPalette :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700">界面主题</div>
                  <div class="text-xs text-slate-500">切换明亮或深夜模式</div>
                </div>
              </div>
              <select 
                v-model="config.theme" 
                class="bg-slate-50 border border-slate-200 text-slate-700 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 min-w-[120px] outline-none transition-all"
              >
                <option value="light">明亮</option>
                <option value="dark">深夜</option>
              </select>
            </div>

            <div class="w-full h-px bg-slate-100"></div>

            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-indigo-50 flex items-center justify-center text-indigo-500">
                  <PhPalette :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700">记住上次窗口位置</div>
                  <div class="text-xs text-slate-500">开启后，下次启动会恢复到上次关闭前的位置</div>
                </div>
              </div>
              <label class="inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="config.rememberWindowPosition" class="sr-only peer">
                <div class="relative w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
              </label>
            </div>
          </div>
        </section>

        <!-- NodeJs Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
            <PhPackage :size="20" class="text-green-600" weight="duotone" />
            NodeJs 设置
          </h2>
          
          <div class="bg-white rounded-xl border border-slate-200 p-4 space-y-4 shadow-sm">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-green-50 flex items-center justify-center text-green-600">
                  <PhPackage :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700">NodeJs 环境</div>
                  <div class="text-xs text-slate-500">
                    <span v-if="nodeInfo.version">
                      当前版本: {{ nodeInfo.version }} ({{ nodeInfo.source === 'local' ? '内置' : '系统' }})
                      <div v-if="isNodeVersionValid && nodeInfo.path" class="mt-1 text-slate-400 break-all select-all">
                        路径: {{ nodeInfo.path }}
                      </div>
                      <div v-if="!isNodeVersionValid" class="mt-1 text-red-500">
                        版本低于 18，请更新或下载内置 NodeJs
                      </div>
                    </span>
                    <span v-else>未检测到 Node.js 环境，部分功能可能无法使用</span>
                  </div>
                </div>
              </div>
              
              <div v-if="!isNodeVersionValid || nodeInfo.source === 'local'" class="flex items-center gap-2">
                 <button 
                    @click="installNode" 
                    :disabled="installingNode"
                    class="px-3 py-1.5 text-xs font-medium bg-green-50 text-green-600 rounded-md hover:bg-green-100 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                 >
                    <PhArrowsClockwise v-if="installingNode" :size="14" class="animate-spin" />
                    <PhDownloadSimple v-else :size="14" />
                    {{ installingNode ? '安装中...' : (nodeInfo.version ? '重新安装' : '立即安装') }}
                 </button>
              </div>
            </div>

            <div v-if="installingNode" class="space-y-2 pt-2 border-t border-slate-100">
               <div class="flex justify-between text-xs text-slate-500">
                  <span>{{ nodeProgress.log }}</span>
                  <span>{{ Math.round(nodeProgress.progress * 100) }}%</span>
               </div>
               <div class="w-full bg-slate-100 rounded-full h-1.5 overflow-hidden">
                  <div class="bg-green-500 h-1.5 rounded-full transition-all duration-300" :style="{ width: `${nodeProgress.progress * 100}%` }"></div>
               </div>
            </div>

            <div class="w-full h-px bg-slate-100"></div>

            <!-- NPM Info -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-red-50 flex items-center justify-center text-red-500">
                  <PhPackage :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700">NPM 环境</div>
                  <div class="text-xs text-slate-500">
                    <span v-if="npmInfo.version">
                      当前版本: {{ npmInfo.version }} ({{ npmInfo.source === 'local' ? '内置' : '系统' }})
                      <div v-if="npmInfo.path" class="mt-1 text-slate-400 break-all select-all">
                        路径: {{ npmInfo.path }}
                      </div>
                    </span>
                    <span v-else>未检测到 NPM 环境</span>
                  </div>
                </div>
              </div>
            </div>

            <div class="w-full h-px bg-slate-100"></div>

            <!-- NPM Registry Selection -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-orange-50 flex items-center justify-center text-orange-500">
                  <PhGlobe :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700">NPM 源设置</div>
                  <div class="text-xs text-slate-500">选择合适的镜像源以加速依赖安装</div>
                </div>
              </div>
              <select 
                v-model="config.npmRegistry" 
                @change="saveConfig"
                class="bg-slate-50 border border-slate-200 text-slate-700 text-xs rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-48 p-2"
              >
                <option v-for="registry in npmRegistries" :key="registry.url" :value="registry.url">
                  {{ registry.name }}
                </option>
              </select>
            </div>
            <div class="text-[10px] text-slate-400 pl-11">
               当前地址: {{ config.npmRegistry }}
            </div>
          </div>
        </section>

        <!-- Github Proxy Settings -->
        <section class="space-y-4">
          <h2 class="text-lg font-semibold text-slate-800 flex items-center gap-2">
            <PhGithubLogo :size="20" class="text-slate-700" weight="duotone" />
            Github 加速设置
          </h2>
          
          <div class="bg-white rounded-xl border border-slate-200 p-4 space-y-4 shadow-sm">
            <!-- Toggle -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-slate-100 flex items-center justify-center text-slate-600">
                  <PhGithubLogo :size="18" weight="duotone" />
                </div>
                <div>
                  <div class="font-medium text-slate-700">加速总开关</div>
                  <div class="text-xs text-slate-500">开启后将使用代理加速 Github 资源下载</div>
                </div>
              </div>
              <label class="inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="config.githubProxy.enable" class="sr-only peer">
                <div class="relative w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
              </label>
            </div>

            <!-- Current URL Display -->
             <div class="flex items-center gap-3 p-3 bg-slate-50 rounded-lg border border-slate-100">
                <div class="text-sm font-medium text-slate-500 whitespace-nowrap">当前地址:</div>
                <div class="text-sm text-slate-800 font-mono truncate select-all">{{ config.githubProxy.url }}</div>
             </div>

            <div class="w-full h-px bg-slate-100"></div>

            <!-- Proxy List Header -->
            <div class="flex items-center justify-between pt-2">
              <h3 class="text-sm font-medium text-slate-700">加速节点列表</h3>
              <button 
                @click="fetchProxies" 
                :disabled="proxyLoading"
                class="text-xs flex items-center gap-1.5 px-3 py-1.5 bg-blue-50 text-blue-600 rounded-md hover:bg-blue-100 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <PhArrowsClockwise :class="{'animate-spin': proxyLoading}" :size="14" />
                {{ proxyLoading ? '测速中...' : '刷新列表' }}
              </button>
            </div>

            <!-- Proxy List -->
            <div v-if="proxies.length > 0" class="space-y-2 max-h-60 overflow-y-auto custom-scrollbar pr-1">
              <div 
                v-for="proxy in proxies" 
                :key="proxy.url"
                @click="selectProxy(proxy.url)"
                :class="[
                  'flex items-center justify-between p-3 rounded-lg border cursor-pointer transition-all hover:shadow-sm',
                  config.githubProxy.url === proxy.url 
                    ? 'bg-blue-50 border-blue-200 ring-1 ring-blue-200' 
                    : 'bg-white border-slate-100 hover:border-slate-300'
                ]"
              >
                <div class="flex items-center gap-3 overflow-hidden">
                   <div :class="[
                     'w-4 h-4 rounded-full flex items-center justify-center shrink-0',
                     config.githubProxy.url === proxy.url ? 'text-blue-600' : 'text-transparent'
                   ]">
                     <PhCheck :size="12" weight="bold" />
                   </div>
                   <div class="text-sm font-mono truncate text-slate-600">{{ proxy.url }}</div>
                </div>
                <div class="flex items-center gap-2 shrink-0">
                  <span :class="[
                    'text-xs font-medium px-2 py-0.5 rounded',
                    proxy.latency < 200 ? 'bg-green-100 text-green-700' : 
                    proxy.latency < 500 ? 'bg-yellow-100 text-yellow-700' : 
                    'bg-red-100 text-red-700'
                  ]">
                    {{ proxy.latency }}ms
                  </span>
                </div>
              </div>
            </div>
            
            <div v-else-if="!proxyLoading" class="text-center py-8 text-slate-400 text-sm bg-slate-50 rounded-lg border border-dashed border-slate-200">
              点击刷新列表获取最新的 Github 加速节点
            </div>
            
            <div v-else class="py-8 flex justify-center">
               <div class="animate-pulse flex space-x-4 w-full px-4">
                 <div class="flex-1 space-y-3 py-1">
                   <div class="h-10 bg-slate-100 rounded"></div>
                   <div class="h-10 bg-slate-100 rounded"></div>
                   <div class="h-10 bg-slate-100 rounded"></div>
                 </div>
               </div>
            </div>

          </div>
        </section>

      </div>

      <!-- About Settings -->
      <div v-if="activeTab === 'about'" class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
         <div class="bg-white rounded-xl border border-slate-200 p-8 shadow-sm flex flex-col items-center text-center space-y-4">
            <div class="w-20 h-20 bg-slate-100 rounded-2xl flex items-center justify-center mb-2">
               <img src="/tauri.svg" alt="Logo" class="w-12 h-12 opacity-80" />
            </div>
            <div>
               <h2 class="text-xl font-bold text-slate-800">Tavern Assistant</h2>
               <p class="text-slate-500 text-sm mt-1">版本 0.1.0</p>
            </div>
            <p class="text-slate-600 max-w-md text-sm leading-relaxed">
               SillyTavern Launcher 是一个辅助管理 SillyTavern 的工具，提供了一键启动、版本管理、插件管理等功能。
            </p>
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

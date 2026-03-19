<template>
  <div class="h-full flex flex-col p-4 gap-8 text-black">
    <!-- 顶部 Banner -->
    <div class="w-full h-48 rounded-xl overflow-hidden">
      <img src="../assets/images/banner.png" alt="Banner" class="w-full h-full object-cover" />
    </div>

    <!-- 中部 目录按钮 -->
    <div class="flex-1 bg-slate-100 p-6 rounded-xl shadow-sm border border-slate-200">
      <h2 class="text-lg font-bold mb-4 flex items-center text-black">
        <FolderOpenIcon class="w-5 h-5 mr-2 text-primary" />
        快捷目录
      </h2>
      <div class="grid grid-cols-2 gap-4 sm:grid-cols-6">
        <button class="btn btn-outline hover:bg-slate-200 bg-slate-100 border-slate-300 text-black hover:text-black" @click="openDir('root')">
          <FolderIcon class="w-5 h-5 mr-1" />
          根目录
        </button>
        <button class="btn btn-outline hover:bg-slate-200 bg-slate-100 border-slate-300 text-black hover:text-black" @click="openDir('data')">
          <DatabaseIcon class="w-5 h-5 mr-1" />
          数据目录
        </button>
        <button class="btn btn-outline hover:bg-slate-200 bg-slate-100 border-slate-300 text-black hover:text-black" @click="openDir('logs')">
          <FileTextIcon class="w-5 h-5 mr-1" />
          日志目录
        </button>
        <button class="btn btn-outline hover:bg-slate-200 bg-slate-100 border-slate-300 text-black hover:text-black" @click="openDir('tavern')">
          <BeerIcon class="w-5 h-5 mr-1" />
          酒馆目录
        </button>
        <button class="btn btn-outline hover:bg-slate-200 bg-slate-100 border-slate-300 text-black hover:text-black" @click="openExtensionFolder">
          <PuzzleIcon class="w-5 h-5 mr-1" />
          扩展目录
        </button>
        <button class="btn btn-outline hover:bg-slate-200 bg-slate-100 border-slate-300 text-black hover:text-black" @click="openDir('node')">
          <BoxIcon class="w-5 h-5 mr-1" />
          NodeJs
        </button>
      </div>
    </div>

    <!-- 底部 版本信息和一键启动按钮 -->
    <div class="flex items-center justify-between flex-shrink-0 p-6 bg-slate-100 rounded-xl shadow-sm border border-slate-200">
      <div class="flex flex-col space-y-2 text-sm text-black/80">
        <div class="flex items-center">
          <span class="w-24">助手版本：</span>
          <span class="font-medium text-black">{{ appVersion || '获取中...' }}</span>
        </div>
        <div class="flex items-center">
          <span class="w-24">Node.js：</span>
          <span class="font-medium text-black">{{ nodeVersion || '未安装' }}</span>
        </div>
        <div class="flex items-center">
          <span class="w-24">酒馆版本：</span>
          <span class="font-medium text-black">{{ tavernVersion || '未安装' }}</span>
        </div>
      </div>
      
      <button 
        class="btn btn-lg flex items-center justify-center gap-2 px-10 h-20 rounded-2xl shadow-md hover:shadow-lg transition-all border-none text-white"
        :class="status === 1 || status === 2 ? 'btn-error' : 'btn-primary'"
        @click="handleToggleProcess"
      >
        <StopCircleIcon v-if="status === 1 || status === 2" class="w-8 h-8 fill-current" />
        <PlayIcon v-else class="w-8 h-8 fill-current" />
        <span class="text-2xl font-bold tracking-widest">{{ (status === 1 || status === 2) ? '停止进程' : '一键启动' }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
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
  Puzzle as PuzzleIcon
} from 'lucide-vue-next'
import { consoleStatus as status, startProcess, stopProcess } from '../lib/consoleState'
import { Dialog } from '../lib/useDialog'
import { toast } from 'vue-sonner'

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
        title: '选择目录',
        msg: '请选择要打开的扩展目录：',
        confirmText: '当前用户',
        thirdBtnText: '全局',
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
                toast.error('打开用户目录失败: ' + String(e));
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
                if (!version || version === '未安装') {
                    toast.warning('未检测到已安装的酒馆版本，无法打开全局目录');
                    return;
                }
                await invoke('open_extension_folder', { scope: 'global', version });
            } catch (e) {
                toast.error('打开全局目录失败: ' + String(e));
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

const fetchVersions = async () => {
  try {
    appVersion.value = await invoke('get_app_version')
  } catch (e) {
    console.error(e)
  }

  try {
    const nodeInfo: any = await invoke('check_nodejs')
    nodeVersion.value = nodeInfo.version || '未安装'
    nodePath.value = nodeInfo.path || ''
  } catch (e) {
    nodeVersion.value = '未安装'
  }

  try {
    tavernVersion.value = await invoke('get_tavern_version')
  } catch (e) {
    tavernVersion.value = '未安装'
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

onMounted(() => {
  fetchVersions()
})
</script>

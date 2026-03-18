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
      <div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
        <button class="btn btn-outline hover:bg-slate-200 bg-slate-100 border-slate-300 text-black hover:text-black" @click="openDir('root')">
          <FolderIcon class="w-5 h-5 mr-1" />
          根目录
        </button>
        <button class="btn btn-outline hover:bg-slate-200 bg-slate-100 border-slate-300 text-black hover:text-black" @click="openDir('logs')">
          <FileTextIcon class="w-5 h-5 mr-1" />
          日志目录
        </button>
        <button class="btn btn-outline hover:bg-slate-200 bg-slate-100 border-slate-300 text-black hover:text-black" @click="openDir('tavern')">
          <BeerIcon class="w-5 h-5 mr-1" />
          酒馆目录
        </button>
        <button class="btn btn-outline hover:bg-slate-200 bg-slate-100 border-slate-300 text-black hover:text-black" @click="openDir('node')">
          <BoxIcon class="w-5 h-5 mr-1" />
          NodeJs目录
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
      
      <button class="btn btn-primary btn-lg flex items-center justify-center gap-2 px-10 h-20 rounded-2xl shadow-md hover:shadow-lg transition-all border-none text-white">
        <PlayIcon class="w-8 h-8 fill-current" />
        <span class="text-2xl font-bold tracking-widest">一键启动</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { 
  Play as PlayIcon, 
  Folder as FolderIcon, 
  FolderOpen as FolderOpenIcon,
  FileText as FileTextIcon, 
  Beer as BeerIcon, 
  Box as BoxIcon 
} from 'lucide-vue-next'

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

onMounted(() => {
  fetchVersions()
})
</script>

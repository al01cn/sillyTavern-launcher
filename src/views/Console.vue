<script setup lang="ts">
import { ref } from 'vue'
import { 
  TerminalSquare, 
  Play, 
  StopCircle, 
  Square, 
  CircleDashed, 
  CheckCircle2, 
  XCircle 
} from 'lucide-vue-next'

// 控制台状态
// 0: 未启动, 1: 启动中, 2: 启动成功, 3: 启动失败
const status = ref(0)

// 一键启动
const startProcess = () => {
  if (status.value === 1 || status.value === 2) return
  status.value = 1
  
  // 模拟启动过程
  setTimeout(() => {
    // 随机模拟成功或失败，以便查看UI效果
    status.value = Math.random() > 0.2 ? 2 : 3
  }, 2000)
}

// 停止进程
const stopProcess = () => {
  if (status.value === 0) return
  status.value = 0
}
</script>

<template>
  <div class="absolute inset-0 flex flex-col bg-slate-50 z-10">
    <!-- 顶部：状态栏和按钮组 -->
    <div class="h-16 shrink-0 bg-white border-b border-slate-200/80 px-6 flex items-center justify-between shadow-sm">
      
      <!-- 左侧：图标、标题和状态展示 -->
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2">
          <div class="w-8 h-8 rounded-lg bg-slate-100 flex items-center justify-center text-slate-700">
            <TerminalSquare class="w-5 h-5" />
          </div>
          <h1 class="text-base font-bold text-slate-800">控制台日志</h1>
        </div>
        
        <div class="h-5 w-px bg-slate-200"></div>
        
        <!-- 状态指示器 -->
        <div class="flex items-center gap-2 px-3 py-1.5 rounded-full text-sm font-medium transition-colors"
             :class="{
               'bg-slate-100 text-slate-600': status === 0,
               'bg-blue-50 text-blue-600': status === 1,
               'bg-emerald-50 text-emerald-600': status === 2,
               'bg-red-50 text-red-600': status === 3
             }">
          <!-- 状态图标 -->
          <Square v-if="status === 0" class="w-4 h-4" />
          <CircleDashed v-else-if="status === 1" class="w-4 h-4 animate-spin" />
          <CheckCircle2 v-else-if="status === 2" class="w-4 h-4" />
          <XCircle v-else-if="status === 3" class="w-4 h-4" />
          
          <!-- 状态文本 -->
          <span>
            <template v-if="status === 0">未启动</template>
            <template v-else-if="status === 1">启动中...</template>
            <template v-else-if="status === 2">启动成功</template>
            <template v-else-if="status === 3">启动失败</template>
          </span>
        </div>
      </div>
      
      <!-- 右侧：按钮组 -->
      <div class="flex items-center gap-3">
        <!-- 停止进程按钮 -->
        <button 
          @click="stopProcess"
          :disabled="status === 0"
          class="btn btn-sm btn-error btn-outline flex items-center gap-1.5 disabled:opacity-50 disabled:bg-transparent"
        >
          <StopCircle class="w-4 h-4" />
          停止进程
        </button>
        
        <!-- 一键启动按钮 -->
        <button 
          @click="startProcess"
          :disabled="status === 1 || status === 2"
          class="btn btn-sm btn-primary flex items-center gap-1.5 text-white disabled:opacity-50"
        >
          <Play class="w-4 h-4 fill-current" />
          一键启动
        </button>
      </div>
    </div>
    
    <!-- 日志内容区域（深色主题） -->
    <div class="flex-1 overflow-y-auto bg-[#1e1e2e] p-4 text-slate-300 font-mono text-sm leading-relaxed selection:bg-blue-500/30">
      <div class="max-w-full space-y-1">
        <!-- 初始提示 -->
        <div class="flex gap-4 opacity-50">
          <span class="shrink-0 text-slate-500 w-20">System</span>
          <span>控制台已就绪，等待进程启动...</span>
        </div>
        
        <!-- 模拟启动日志 -->
        <template v-if="status > 0">
          <div class="flex gap-4">
            <span class="shrink-0 text-blue-400 w-20">Info</span>
            <span>正在初始化酒馆环境...</span>
          </div>
          <div class="flex gap-4">
            <span class="shrink-0 text-blue-400 w-20">Info</span>
            <span>检查 Node.js 依赖...</span>
          </div>
        </template>
        
        <!-- 成功日志 -->
        <div v-if="status === 2" class="flex gap-4 mt-2">
          <span class="shrink-0 text-emerald-400 w-20">Success</span>
          <span class="text-emerald-300">酒馆服务已成功启动！运行在 http://localhost:8000</span>
        </div>
        
        <!-- 失败日志 -->
        <div v-if="status === 3" class="flex gap-4 mt-2">
          <span class="shrink-0 text-red-400 w-20">Error</span>
          <span class="text-red-300">启动失败：环境初始化异常或端口被占用。</span>
        </div>
      </div>
    </div>
  </div>
</template>

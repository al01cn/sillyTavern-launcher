<script setup lang="ts">
import { ref, onMounted, nextTick, watch } from 'vue'
import { 
  TerminalSquare, 
  Play, 
  StopCircle, 
  Square, 
  CircleDashed, 
  CheckCircle2, 
  XCircle 
} from 'lucide-vue-next'
import { consoleStatus as status, consoleLogs as logs, startProcess, stopProcess } from '../lib/consoleState'

const logsContainer = ref<HTMLElement | null>(null)

// 自动滚动到底部
const scrollToBottom = () => {
  if (logsContainer.value) {
    logsContainer.value.scrollTop = logsContainer.value.scrollHeight
  }
}

watch(logs, () => {
  nextTick(scrollToBottom)
}, { deep: true })

onMounted(() => {
  scrollToBottom()
})
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
            <template v-else-if="status === 2">运行中</template>
            <template v-else-if="status === 3">已停止 / 异常</template>
          </span>
        </div>
      </div>
      
      <!-- 右侧：按钮组 -->
      <div class="flex items-center gap-3">
        <!-- 停止进程按钮 -->
        <button 
          @click="stopProcess"
          :disabled="status === 0 || status === 3"
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
    <div ref="logsContainer" class="flex-1 overflow-y-auto bg-[#1e1e2e] p-4 text-slate-300 font-mono text-sm leading-relaxed selection:bg-blue-500/30">
      <div class="max-w-full space-y-1">
        <!-- 初始提示 -->
        <div v-if="logs.length === 0" class="flex gap-4 opacity-50">
          <span class="shrink-0 text-slate-500 w-20">System</span>
          <span>控制台已就绪，等待进程启动...</span>
        </div>
        
        <!-- 动态日志 -->
        <div v-for="log in logs" :key="log.id" class="flex gap-4 break-all whitespace-pre-wrap">
          <span class="shrink-0 w-20 capitalize" :class="{
            'text-blue-400': log.type === 'info',
            'text-emerald-400': log.type === 'success',
            'text-red-400': log.type === 'error',
            'text-slate-400': log.type === 'output',
            'text-slate-500': log.type === 'system'
          }">{{ log.type }}</span>
          <span :class="{
            'text-blue-300': log.type === 'info',
            'text-emerald-300': log.type === 'success',
            'text-red-300': log.type === 'error',
            'text-slate-300': log.type === 'output',
            'text-slate-400': log.type === 'system'
          }">{{ log.text }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import { TerminalSquare, Play, Square, CircleDashed, CheckCircle2, XCircle, Trash2 } from 'lucide-vue-next'
import {
  consoleStatus as status,
  consoleLogs as logs,
  serverUrl,
  processPid,
  startProcess,
  stopProcess,
  clearLogs,
  networkMode,
  networkPort,
  launchMode,
} from '../lib/consoleState'
import { openUrl } from '@tauri-apps/plugin-opener'
import { updateOneClickMessage, startOneClickSetup, finishOneClickSetup, simulateClickEffect } from '../lib/useOneClick'
import NetworkLinkDialog from '../components/NetworkLinkDialog.vue'
const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const logsContainer = ref<HTMLElement | null>(null)
const showNetworkDialog = ref(false)

// 自动滚动到底部
const scrollToBottom = () => {
  if (logsContainer.value) {
    logsContainer.value.scrollTop = logsContainer.value.scrollHeight
  }
}

watch(
  logs,
  () => {
    nextTick(scrollToBottom)
  },
  { deep: true },
)

onMounted(() => {
  scrollToBottom()

  if (route.query.action === 'one_click_start') {
    startOneClickSetup(t('oneClick.starting'))
    setTimeout(() => {
      simulateClickEffect('btn-console-start')
      startProcess()
    }, 1000)
  }
})

watch(status, newStatus => {
  if (route.query.action === 'one_click_start') {
    if (newStatus === 2) {
      // 运行中
      updateOneClickMessage(t('oneClick.started'))
      setTimeout(() => {
        finishOneClickSetup()
        router.replace({ query: {} })
      }, 3000)
    } else if (newStatus === 4 || newStatus === 3) {
      // 错误或停止
      updateOneClickMessage(t('oneClick.startFailed'))
      setTimeout(() => {
        finishOneClickSetup()
        router.replace({ query: {} })
      }, 3000)
    }
  }
})

const formatTime = (time: number) => {
  const date = new Date(time)
  return `${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}:${date.getSeconds().toString().padStart(2, '0')}`
}

const urlRegex = /(https?:\/\/[^\s"'<>]+)/g

const parseLogText = (text: string) => {
  if (!text) return []
  const parts = []
  let lastIndex = 0
  let match

  const regex = new RegExp(urlRegex)
  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push({ type: 'text', content: text.slice(lastIndex, match.index) })
    }

    let url = match[0]
    let trailing = ''

    // 处理 URL 末尾可能带有的标点符号
    const trailingPunctuation = /[.,;?!)]$/
    if (trailingPunctuation.test(url)) {
      trailing = url.slice(-1)
      url = url.slice(0, -1)
    }

    parts.push({ type: 'link', content: url })
    if (trailing) {
      parts.push({ type: 'text', content: trailing })
    }

    lastIndex = regex.lastIndex
  }
  if (lastIndex < text.length) {
    parts.push({ type: 'text', content: text.slice(lastIndex) })
  }
  return parts
}

const handleOpenUrl = async (url: string) => {
  try {
    await openUrl(url)
  } catch (err) {
    console.error('Failed to open url:', err)
  }
}
</script>

<template>
  <div class="absolute inset-0 flex flex-col bg-[#0f111a] z-10 font-sans shadow-inner">
    <!-- 顶部栏：macOS 风格控制台头部 -->
    <div
      class="h-14 shrink-0 bg-[#1a1d27] border-b border-[#2a2d3d] px-4 flex items-center justify-between shadow-sm select-none"
    >
      <!-- 左侧：状态指示器与标题 -->
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-2">
          <TerminalSquare class="w-5 h-5 text-slate-400" />
          <h1 class="text-sm font-medium text-slate-200 tracking-wide">
            {{ t('console.serviceConsole') }}
          </h1>
        </div>

        <div class="h-4 w-px bg-[#2a2d3d] mx-1"></div>

        <!-- 状态标签 -->
        <div
          class="flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium transition-colors"
          :class="{
            'bg-[#2a2d3d] text-slate-400': status === 0,
            'bg-blue-500/20 text-blue-400': status === 1,
            'bg-emerald-500/20 text-emerald-400': status === 2,
            'bg-orange-500/20 text-orange-400': status === 3,
            'bg-red-500/20 text-red-400': status === 4,
          }"
        >
          <Square v-if="status === 0" class="w-3.5 h-3.5" />
          <CircleDashed v-else-if="status === 1" class="w-3.5 h-3.5 animate-spin" />
          <CheckCircle2 v-else-if="status === 2" class="w-3.5 h-3.5" />
          <Square v-else-if="status === 3" class="w-3.5 h-3.5" />
          <XCircle v-else-if="status === 4" class="w-3.5 h-3.5" />

          <span>
            <template v-if="status === 0">{{ t('console.notStarted') }}</template>
            <template v-else-if="status === 1">{{ t('console.starting') }}</template>
            <template v-else-if="status === 2">{{ t('console.started') }}</template>
            <template v-else-if="status === 3">{{ t('console.stopped') }}</template>
            <template v-else-if="status === 4">{{ t('console.failed') }}</template>
          </span>
        </div>

        <!-- PID 和 访问链接 -->
        <div v-if="(status === 1 || status === 2) && processPid" class="flex items-center gap-2 ml-2">
          <div class="h-4 w-px bg-[#2a2d3d]"></div>
          <span class="text-xs font-mono text-slate-400">PID: {{ processPid }}</span>
        </div>

        <!-- 普通模式：直接显示访问链接 -->
        <div
          v-if="status === 2 && serverUrl && networkMode === null && launchMode !== 'desktop'"
          class="flex items-center gap-2 ml-2"
        >
          <div class="h-4 w-px bg-[#2a2d3d]"></div>
          <button
            class="flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium text-emerald-400 bg-emerald-500/10 hover:bg-emerald-500/20 border border-emerald-500/20 transition-colors group"
            :title="t('console.openInBrowser')"
            @click="handleOpenUrl(serverUrl)"
          >
            <span>{{ t('console.visitTavern') }}: {{ serverUrl }}</span>
            <svg
              class="w-3.5 h-3.5 opacity-70 group-hover:opacity-100"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
              <polyline points="15 3 21 3 21 9"></polyline>
              <line x1="10" y1="14" x2="21" y2="3"></line>
            </svg>
          </button>
        </div>

        <!-- 局域网/公网模式：打开连接弹窗 -->
        <div v-else-if="status === 2 && networkMode !== null" class="flex items-center gap-2 ml-2">
          <div class="h-4 w-px bg-[#2a2d3d]"></div>
          <button
            :class="[
              'flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium border transition-colors group',
              networkMode === 'lan'
                ? 'text-emerald-400 bg-emerald-500/10 hover:bg-emerald-500/20 border-emerald-500/20'
                : 'text-red-400 bg-red-500/10 hover:bg-red-500/20 border-red-500/20',
            ]"
            @click="showNetworkDialog = true"
          >
            <span>{{ networkMode === 'lan' ? t('home.lanLink') : t('home.publicLink') }}</span>
            <svg
              class="w-3.5 h-3.5 opacity-70 group-hover:opacity-100"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <circle cx="12" cy="12" r="10" />
              <line x1="2" y1="12" x2="22" y2="12" />
              <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
            </svg>
          </button>
        </div>
      </div>

      <!-- 右侧：操作按钮 -->
      <div class="flex items-center gap-2">
        <button
          :title="t('console.clearLogs')"
          class="h-8 w-8 rounded-md flex items-center justify-center text-slate-400 hover:text-slate-200 hover:bg-[#2a2d3d] transition-colors"
          @click="clearLogs"
        >
          <Trash2 class="w-4 h-4" />
        </button>

        <div class="h-4 w-px bg-[#2a2d3d] mx-1"></div>

        <!-- 停止进程按钮 -->
        <button
          :disabled="status === 0 || status === 3 || status === 4"
          class="h-8 px-3 rounded-md flex items-center gap-1.5 text-xs font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          :class="
            status === 0 || status === 3 || status === 4
              ? 'text-slate-500 bg-[#2a2d3d]'
              : 'text-red-400 bg-red-500/10 hover:bg-red-500/20 border border-red-500/20'
          "
          @click="stopProcess"
        >
          <Square class="w-3.5 h-3.5 fill-current" />
          {{ t('console.stop') }}
        </button>

        <!-- 启动按钮 -->
        <button
          id="btn-console-start"
          :disabled="status === 1 || status === 2"
          class="h-8 px-3 rounded-md flex items-center gap-1.5 text-xs font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          :class="
            status === 1 || status === 2
              ? 'text-slate-500 bg-[#2a2d3d]'
              : 'text-emerald-400 bg-emerald-500/10 hover:bg-emerald-500/20 border border-emerald-500/20'
          "
          @click="startProcess"
        >
          <Play class="w-3.5 h-3.5 fill-current" />
          {{ t('console.start') }}
        </button>
      </div>
    </div>

    <!-- 日志内容区域 -->
    <div
      ref="logsContainer"
      class="flex-1 overflow-y-auto p-4 text-sm font-mono leading-relaxed selection:bg-blue-500/30 scroll-smooth custom-scrollbar"
    >
      <div class="max-w-full flex flex-col gap-1">
        <!-- 初始提示 -->
        <div
          v-if="logs.length === 0"
          class="flex items-center justify-center h-full min-h-[200px] opacity-40 select-none"
        >
          <div class="flex flex-col items-center gap-3">
            <TerminalSquare class="w-12 h-12 text-slate-500" />
            <span class="text-slate-400">{{ t('console.consoleReady') }}</span>
          </div>
        </div>

        <!-- 动态日志 -->
        <div
          v-for="log in logs"
          :key="log.id"
          class="flex gap-3 hover:bg-[#1a1d27] px-2 py-0.5 rounded transition-colors group"
        >
          <span class="shrink-0 text-slate-600 select-none text-xs mt-0.5">[{{ formatTime(log.time) }}]</span>
          <span
            class="shrink-0 w-[4.5rem] text-xs font-bold uppercase tracking-wider mt-0.5 select-none"
            :class="{
              'text-blue-400': log.type === 'info',
              'text-emerald-400': log.type === 'success',
              'text-red-400': log.type === 'error',
              'text-slate-400': log.type === 'output',
              'text-purple-400': log.type === 'system',
            }"
            >{{ log.type }}</span
          >
          <span
            class="break-all whitespace-pre-wrap"
            :class="{
              'text-blue-100': log.type === 'info',
              'text-emerald-100': log.type === 'success',
              'text-red-300': log.type === 'error',
              'text-slate-300': log.type === 'output',
              'text-purple-200': log.type === 'system',
            }"
          >
            <template v-for="(part, index) in parseLogText(log.text)" :key="index">
              <span v-if="part.type === 'text'">{{ part.content }}</span>
              <a
                v-else
                class="text-blue-400 hover:text-blue-300 underline cursor-pointer transition-colors"
                :title="t('console.clickToOpen')"
                @click.prevent="handleOpenUrl(part.content)"
                >{{ part.content }}</a
              >
            </template>
          </span>
        </div>
      </div>
    </div>
  </div>

  <!-- 局域网/公网连接弹窗 -->
  <NetworkLinkDialog
    v-if="networkMode !== null"
    :open="showNetworkDialog"
    :mode="networkMode"
    :port="networkPort"
    @close="showNetworkDialog = false"
  />
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 10px;
  height: 10px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #2a2d3d;
  border: 2px solid #0f111a;
  border-radius: 5px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #3a3d4d;
}
</style>

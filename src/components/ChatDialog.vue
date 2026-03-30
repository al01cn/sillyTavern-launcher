<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { X, Loader2, MessageCircle } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { chatDialogState, closeChatDialog } from '../lib/useChatDialog'

const { t } = useI18n()

interface ChatMessage {
  name: string
  mes: string
  isUser: boolean
  isSystem: boolean
  sendDate?: string | null
}

const loading = ref(false)
const errorMsg = ref('')
const messages = ref<ChatMessage[]>([])
const messagesEl = ref<HTMLElement | null>(null)

// 打开时加载消息
watch(
  () => chatDialogState.show,
  async (show) => {
    if (show) {
      await loadChat()
    } else {
      messages.value = []
      errorMsg.value = ''
    }
  }
)

async function loadChat() {
  loading.value = true
  errorMsg.value = ''
  try {
    const result = await invoke<ChatMessage[]>('read_chat', {
      charFolder: chatDialogState.charFolder,
      fileName: chatDialogState.fileName,
    })
    messages.value = result
    await nextTick()
    scrollToBottom()
  } catch (e: any) {
    errorMsg.value = e?.message ? String(e.message) : String(e)
  } finally {
    loading.value = false
  }
}

function scrollToBottom() {
  if (messagesEl.value) {
    messagesEl.value.scrollTop = messagesEl.value.scrollHeight
  }
}

// 简单 markdown 渲染：**粗体** *斜体* \n 换行
function renderMes(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/\*\*(.+?)\*\*/gs, '<strong>$1</strong>')
    .replace(/\*(.+?)\*/gs, '<em>$1</em>')
    .replace(/\n/g, '<br/>')
}

// "Seraphina - 2023-5-12 @21h 32m 29s 224ms.jsonl" → "Seraphina · 2023-5-12 21:32"
function parseFileTitle(name: string): string {
  const stem = name.replace(/\.jsonl$/i, '')
  const m = stem.match(/^(.+?)\s*-\s*(\d{4}-\d{1,2}-\d{1,2})\s*@(\d+)h\s*(\d+)m/)
  if (m) {
    return `${m[1]} · ${m[2]} ${m[3].padStart(2, '0')}:${m[4].padStart(2, '0')}`
  }
  return stem
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="chatDialogState.show"
      class="fixed inset-0 z-50 flex items-center justify-center p-4"
    >
      <!-- 背景遮罩 -->
      <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" @click="closeChatDialog" />

      <!-- 弹窗 -->
      <div
        class="relative z-10 w-full max-w-2xl max-h-[85vh] bg-white dark:bg-slate-900 rounded-2xl shadow-2xl flex flex-col overflow-hidden animate-in fade-in zoom-in-95 duration-200"
      >
        <!-- 头部 -->
        <div class="flex items-center gap-3 px-5 py-4 border-b border-slate-200 dark:border-slate-700 shrink-0">
          <div class="w-8 h-8 rounded-lg bg-purple-100 dark:bg-purple-900/40 flex items-center justify-center shrink-0">
            <MessageCircle class="w-4 h-4 text-purple-500 dark:text-purple-400" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-sm font-semibold text-slate-800 dark:text-slate-100 truncate">
              {{ chatDialogState.charName }}
            </div>
            <div class="text-xs text-slate-500 dark:text-slate-400 truncate">
              {{ parseFileTitle(chatDialogState.fileName) }}
            </div>
          </div>
          <div v-if="!loading" class="text-xs text-slate-400 dark:text-slate-500 shrink-0">
            {{ t('resources.chat.totalMessages', { count: messages.length }) }}
          </div>
          <button
            @click="closeChatDialog"
            class="p-1.5 rounded-lg text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- 消息区域 -->
        <div
          ref="messagesEl"
          class="flex-1 overflow-y-auto px-4 py-4 space-y-3 min-h-0"
          style="scroll-behavior: smooth"
        >
          <!-- 加载中 -->
          <div v-if="loading" class="flex flex-col items-center justify-center h-40 gap-3 text-slate-400">
            <Loader2 class="w-6 h-6 animate-spin" />
            <span class="text-sm">{{ t('common.loading') }}</span>
          </div>

          <!-- 错误 -->
          <div
            v-else-if="errorMsg"
            class="bg-red-50 dark:bg-red-900/20 border border-red-100 dark:border-red-800 rounded-xl p-4 text-sm text-red-600 dark:text-red-400"
          >
            {{ errorMsg }}
          </div>

          <!-- 空消息 -->
          <div
            v-else-if="messages.length === 0"
            class="flex items-center justify-center h-32 text-slate-400 text-sm"
          >
            {{ t('resources.chat.noMessages') }}
          </div>

          <!-- 消息气泡 -->
          <template v-else>
            <div
              v-for="(msg, idx) in messages"
              :key="idx"
              class="flex items-end gap-2"
              :class="msg.isSystem ? 'justify-center' : (msg.isUser ? 'flex-row-reverse' : 'flex-row')"
            >
              <!-- 系统消息 -->
              <template v-if="msg.isSystem">
                <div class="text-xs text-slate-400 dark:text-slate-500 bg-slate-100 dark:bg-slate-800 px-3 py-1 rounded-full max-w-[80%] text-center">
                  <span v-html="renderMes(msg.mes)" />
                </div>
              </template>

              <!-- 普通消息 -->
              <template v-else>
                <!-- 头像 -->
                <div
                  class="w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold shrink-0 mb-0.5"
                  :class="msg.isUser
                    ? 'bg-blue-500 text-white'
                    : 'bg-purple-100 dark:bg-purple-900/40 text-purple-600 dark:text-purple-400'"
                >
                  {{ (msg.name || (msg.isUser ? 'U' : chatDialogState.charName[0] || 'A'))[0].toUpperCase() }}
                </div>

                <!-- 气泡 -->
                <div
                  class="max-w-[72%] flex flex-col"
                  :class="msg.isUser ? 'items-end' : 'items-start'"
                >
                  <!-- 发送者名字 -->
                  <div class="text-xs text-slate-400 dark:text-slate-500 mb-1 px-1">
                    {{ msg.name || (msg.isUser ? t('resources.chat.you') : chatDialogState.charName) }}
                  </div>

                  <!-- 消息内容 -->
                  <div
                    class="px-3.5 py-2.5 rounded-2xl text-sm leading-relaxed break-words"
                    :class="msg.isUser
                      ? 'bg-blue-500 text-white rounded-br-sm'
                      : 'bg-slate-100 dark:bg-slate-800 text-slate-800 dark:text-slate-100 rounded-bl-sm'"
                  >
                    <span v-html="renderMes(msg.mes)" />
                  </div>

                  <!-- 时间戳 -->
                  <div
                    v-if="msg.sendDate"
                    class="text-xs text-slate-400 dark:text-slate-500 mt-1 px-1"
                  >
                    {{ msg.sendDate }}
                  </div>
                </div>
              </template>
            </div>
          </template>
        </div>

        <!-- 底部文件名 -->
        <div class="px-4 py-3 border-t border-slate-200 dark:border-slate-700 text-xs text-slate-400 dark:text-slate-500 text-center shrink-0 truncate">
          {{ chatDialogState.fileName }}
        </div>
      </div>
    </div>
  </Teleport>
</template>

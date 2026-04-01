<script lang="ts" setup>
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { repairGitDialogState, closeRepairGitDialog } from '../lib/useRepairGitDialog'
import { Terminal, Copy, Trash2, CheckCircle2, XCircle, Loader2, WifiOff } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const logContainer = ref<HTMLElement | null>(null)
let unlisten: UnlistenFn | null = null

onMounted(async () => {
  unlisten = await listen<string>('git-install-log', event => {
    repairGitDialogState.logs.push(event.payload)
    nextTick(() => {
      if (logContainer.value) {
        logContainer.value.scrollTop = logContainer.value.scrollHeight
      }
    })
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})

const copyLogs = async () => {
  try {
    await writeText(repairGitDialogState.logs.join('\n'))
    toast.success(t('common.copySuccess'))
  } catch (e) {
    toast.error(String(e))
  }
}
</script>

<template>
  <div
    v-if="repairGitDialogState.show"
    class="absolute inset-0 z-[300] flex items-center justify-center px-4 animate-in fade-in duration-200"
  >
    <!-- Backdrop：修复中禁止点击关闭 -->
    <div
      class="absolute inset-0 bg-slate-900/60 backdrop-blur-sm pointer-events-auto"
      @click="!repairGitDialogState.isRepairing && closeRepairGitDialog()"
    ></div>

    <!-- Dialog -->
    <div
      class="relative bg-slate-900 w-full max-w-lg rounded-2xl shadow-2xl border border-slate-700 overflow-hidden flex flex-col pointer-events-auto animate-in zoom-in-95 duration-200"
      @click.stop
    >
      <!-- Header -->
      <div class="px-5 py-3 border-b border-slate-700 flex items-center justify-between bg-slate-800">
        <div class="flex items-center gap-2 min-w-0">
          <Terminal class="w-4 h-4 text-blue-400 shrink-0" />
          <h4 class="text-xs font-bold text-slate-200 truncate">
            {{ t('extensions.repairAutoUpdate') }}
            <span v-if="repairGitDialogState.extensionName" class="text-slate-400 font-normal ml-1">
              · {{ repairGitDialogState.extensionName }}
            </span>
          </h4>
        </div>
        <div class="flex items-center gap-3 shrink-0 ml-3">
          <button
            class="text-[10px] text-slate-400 hover:text-white transition-colors flex items-center gap-1"
            @click="copyLogs"
          >
            <Copy class="w-3 h-3" />
            {{ t('resources.copyLog') }}
          </button>
          <button
            class="text-[10px] text-slate-400 hover:text-white transition-colors flex items-center gap-1"
            @click="repairGitDialogState.logs = []"
          >
            <Trash2 class="w-3 h-3" />
            {{ t('resources.clearLog') }}
          </button>
        </div>
      </div>

      <!-- Log Area -->
      <div
        ref="logContainer"
        class="p-4 h-[300px] overflow-y-auto font-mono text-[11px] leading-relaxed custom-scrollbar bg-slate-950"
      >
        <div
          v-if="repairGitDialogState.logs.length === 0"
          class="flex flex-col items-center justify-center h-full text-slate-500 italic"
        >
          <Loader2 class="w-5 h-5 animate-spin mb-2 opacity-20" />
          <span>{{ t('resources.waitingForGitOutput') }}</span>
        </div>
        <div
          v-for="(log, index) in repairGitDialogState.logs"
          v-else
          :key="index"
          class="text-slate-300 break-all mb-1"
        >
          <span class="text-slate-600 mr-2 select-none">[{{ index + 1 }}]</span>
          <span>{{ log }}</span>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-5 py-3 bg-slate-800 border-t border-slate-700 flex items-center justify-between gap-3">
        <!-- 状态区域 -->
        <div v-if="repairGitDialogState.isRepairing" class="flex items-center gap-2 text-blue-400 text-xs font-medium">
          <Loader2 class="w-3.5 h-3.5 animate-spin" />
          <span>{{ t('extensions.repairing') }}</span>
        </div>
        <div
          v-else-if="repairGitDialogState.result === 'success'"
          class="flex items-center gap-2 text-emerald-400 text-xs font-medium"
        >
          <CheckCircle2 class="w-3.5 h-3.5" />
          <span>{{ t('extensions.repairGitSuccess') }}</span>
        </div>
        <div
          v-else-if="repairGitDialogState.result === 'offline'"
          class="flex items-center gap-2 text-amber-400 text-xs font-medium min-w-0"
        >
          <WifiOff class="w-3.5 h-3.5 shrink-0" />
          <span class="break-all">{{ t('extensions.repairGitOffline') }}</span>
        </div>
        <div
          v-else-if="repairGitDialogState.result === 'error'"
          class="flex items-center gap-2 text-red-400 text-xs font-medium"
        >
          <XCircle class="w-3.5 h-3.5" />
          <span>{{ t('extensions.repairGitFailed') }}</span>
        </div>
        <div v-else class="flex-1"></div>

        <!-- 关闭按钮：修复中禁用 -->
        <button
          :disabled="repairGitDialogState.isRepairing"
          class="px-4 py-1.5 rounded-lg text-white text-xs font-bold transition-all disabled:opacity-40 disabled:cursor-not-allowed shrink-0"
          :class="
            repairGitDialogState.isRepairing
              ? 'bg-slate-600'
              : repairGitDialogState.result === 'error'
                ? 'bg-red-600 hover:bg-red-500'
                : repairGitDialogState.result === 'offline'
                  ? 'bg-amber-600 hover:bg-amber-500'
                  : 'bg-blue-600 hover:bg-blue-500'
          "
          @click="closeRepairGitDialog"
        >
          {{ t('common.close') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: #475569;
  border-radius: 20px;
}
</style>

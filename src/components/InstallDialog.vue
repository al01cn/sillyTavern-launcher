<script lang="ts" setup>
import { onMounted, onUnmounted, ref, nextTick } from 'vue';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { installState, resetInstallState } from '../lib/useInstall';
import { CheckCircle2, Loader2, XCircle, Terminal } from 'lucide-vue-next';

const logContainer = ref<HTMLElement | null>(null);
let unlisten: UnlistenFn | null = null;

interface DownloadProgress {
    status: string;
    progress: number;
    log: string;
}

const scrollToBottom = () => {
    nextTick(() => {
        if (logContainer.value) {
            logContainer.value.scrollTop = logContainer.value.scrollHeight;
        }
    });
};

onMounted(async () => {
    unlisten = await listen<DownloadProgress>('install-progress', (event) => {
        const { status, progress, log } = event.payload;

        // Update state
        installState.status = status as any;
        installState.progress = progress;
        
        // Add log if not duplicate of last log (optional, but good for cleanliness)
        if (installState.logs.length === 0 || installState.logs[installState.logs.length - 1] !== log) {
            installState.logs.push(log);
            if (installState.logs.length > 300) {
                installState.logs.shift();
            }
            scrollToBottom();
        }
    });
});

onUnmounted(() => {
    if (unlisten) {
        unlisten();
    }
});

const close = () => {
    if (installState.status === 'done' || installState.status === 'error') {
        resetInstallState();
    }
};

const cancel = async () => {
    if (installState.isCanceling) return;
    
    try {
        installState.isCanceling = true;
        await invoke('cancel_install');
        resetInstallState();
    } catch (e) {
        console.error('Failed to cancel:', e);
        installState.isCanceling = false;
    }
};

</script>

<template>
    <div :class="[
        'absolute inset-0 z-200 flex items-center justify-center px-4 transition-all duration-300',
        installState.show ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'
    ]">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-slate-900/60 backdrop-blur-md"></div>

        <!-- Modal Content -->
        <div :class="[
            'modal-content relative bg-white w-full max-w-2xl rounded-3xl shadow-2xl border border-slate-100 overflow-hidden transition-all duration-300 transform flex flex-col',
            installState.show ? 'scale-100 translate-y-0' : 'scale-95 translate-y-8',
            'h-[600px]'
        ]">
            <!-- Header -->
            <div class="px-8 py-6 border-b border-slate-100 bg-slate-50/50 flex items-center justify-between">
                <div>
                    <h3 class="text-xl font-bold text-slate-800 flex items-center gap-2">
                        <Loader2 v-if="['downloading', 'extracting', 'installing', 'deleting'].includes(installState.status)" class="w-5 h-5 animate-spin text-blue-500" />
                        <CheckCircle2 v-else-if="installState.status === 'done'" class="w-6 h-6 text-emerald-500" />
                        <XCircle v-else-if="installState.status === 'error'" class="w-6 h-6 text-red-500" />
                        <span>{{ installState.operation === 'delete' ? '删除' : '安装' }}酒馆版本 {{ installState.version }}</span>
                    </h3>
                    <p class="text-slate-500 text-sm mt-1">
                        {{ 
                            installState.status === 'downloading' ? '正在下载文件...' : 
                            installState.status === 'extracting' ? '正在解压文件...' : 
                            installState.status === 'installing' ? '正在安装依赖...' : 
                            installState.status === 'deleting' ? '正在删除文件...' :
                            installState.status === 'done' ? (installState.operation === 'delete' ? '删除已完成' : '安装已完成') : 
                            '发生错误'
                        }}
                    </p>
                </div>
            </div>

            <!-- Terminal / Logs -->
            <div class="flex-1 bg-slate-900 overflow-hidden flex flex-col">
                <div class="px-4 h-10 bg-slate-800 border-b border-slate-700 flex items-center justify-between gap-2 text-xs text-slate-400 font-mono">
                    <div class="flex items-center gap-2">
                        <Terminal class="w-3 h-3" />
                        <span>{{ installState.operation === 'delete' ? 'DELETION' : 'INSTALLATION' }} LOGS</span>
                    </div>
                </div>
                
                <div ref="logContainer" class="flex-1 p-4 overflow-y-auto font-mono text-xs text-slate-300 space-y-1">
                    <div v-for="(log, index) in installState.logs" :key="index" class="break-words">
                        <span class="text-slate-500 mr-2">[{{ new Date().toLocaleTimeString() }}]</span>
                        <span :class="log.toLowerCase().includes('error') ? 'text-red-400' : ''">{{ log }}</span>
                    </div>
                    <div v-if="['downloading', 'extracting', 'installing', 'deleting'].includes(installState.status)" class="animate-pulse">_</div>
                </div>
            </div>

            <!-- Footer -->
            <div class="p-5 border-t border-slate-100 bg-white flex justify-end gap-3">
                <button 
                    v-if="['downloading', 'extracting'].includes(installState.status)"
                    @click="cancel"
                    :disabled="installState.isCanceling"
                    :class="[
                        'px-6 py-2.5 rounded-xl font-bold transition-all shadow-sm',
                        installState.isCanceling 
                            ? 'bg-slate-100 text-slate-400 cursor-not-allowed' 
                            : 'bg-red-50 text-red-500 hover:bg-red-100 active:scale-95'
                    ]"
                >
                    {{ installState.isCanceling ? '正在取消...' : '取消' }}
                </button>
                <button 
                    @click="close"
                    :disabled="['downloading', 'extracting', 'installing', 'deleting'].includes(installState.status)"
                    :class="[
                        'px-6 py-2.5 rounded-xl font-bold transition-all',
                        ['downloading', 'extracting', 'installing', 'deleting'].includes(installState.status)
                            ? 'bg-slate-100 text-slate-400 cursor-not-allowed'
                            : 'bg-slate-900 text-white hover:bg-slate-800 active:scale-95 shadow-lg shadow-slate-200'
                    ]"
                >
                    {{ installState.status === 'done' ? '完成' : '关闭' }}
                </button>
            </div>
        </div>
    </div>
</template>

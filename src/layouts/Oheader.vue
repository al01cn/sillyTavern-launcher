<script lang="ts" setup>
import { onMounted, onUnmounted } from 'vue';
import { PhMinus, PhX, PhPlay, PhList, PhClock, PhPlug, PhWrench, PhTerminalWindow, PhGear } from '@phosphor-icons/vue';
import config from '../lib/config'
import { getCurrentWindow } from '@tauri-apps/api/window';
import { installState } from '../lib/useInstall';
import { Dialog } from '../lib/useDialog';

const appWindow = getCurrentWindow();
let unlistenClose: (() => void) | null = null;

const checkCanClose = () => {
    if (installState.show && ['downloading', 'extracting', 'installing', 'deleting'].includes(installState.status)) {
        Dialog.warning({
            title: '警告',
            msg: '正在下载或删除版本，请等待完成',
            showCancel: false,
            confirmText: '我知道了'
        });
        return false;
    }
    return true;
};

const forceClose = async () => {
    if (unlistenClose) {
        unlistenClose();
        unlistenClose = null;
    }
    await appWindow.close();
};

const close = async () => {
    if (!checkCanClose()) return;
    await forceClose();
};

const minimize = async () => {
    await appWindow.minimize();
};

onMounted(async () => {
    unlistenClose = await appWindow.onCloseRequested(async (event) => {
        // 完全接管关闭事件
        event.preventDefault();
        
        if (checkCanClose()) {
            await forceClose();
        }
    });
});

onUnmounted(() => {
    if (unlistenClose) {
        unlistenClose();
    }
});
</script>

<template>
    <div class="flex flex-col h-screen w-screen overflow-hidden bg-white">
        <!-- 1. Header & Navigation -->
        <header class="app-titlebar h-14 shrink-0 flex items-center justify-between px-6 z-60">
            <div class="flex items-center gap-2.5 w-40">
                <div class="w-8 h-8 rounded-lg flex items-center justify-center text-white">
                    <img src="/tauri.svg" alt="logo">
                </div>
                <span class="font-black text-sm tracking-tight text-slate-800 text-nowrap">{{ config.appName }}</span>
            </div>

            <div class="flex items-center gap-1">
                <div class="flex items-center w-40 justify-end h-full gap-1">
                    <button @click="minimize()"
                        class="h-8 w-8 rounded-lg flex items-center justify-center text-slate-400 hover:bg-slate-100 transition-colors">
                        <PhMinus class="w-4 h-4" />
                    </button>
                    <button @click="close()"
                        :class="`h-8 w-8 rounded-lg flex items-center justify-center text-slate-400 hover:bg-red-500 hover:text-white transition-colors`">
                        <PhX class="w-4 h-4" />
                    </button>
                </div>
            </div>
        </header>

        <!-- 2. Body (Sidebar & Content) -->
        <div class="flex flex-1 overflow-hidden relative">
            <!-- Sidebar -->
            <aside class="w-24 shrink-0 border-r border-slate-200/80 bg-white flex flex-col justify-between py-4 z-50">
                <!-- Top Menu -->
                <div class="flex flex-col gap-2 px-3">
                    <router-link to="/" active-class="bg-slate-100 text-slate-900"
                        class="flex flex-col items-center justify-center w-full aspect-square rounded-xl hover:bg-slate-100 transition-colors text-slate-500 hover:text-slate-900 group">
                        <PhPlay :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium">一键启动</span>
                    </router-link>
                    <router-link to="/tavern" active-class="bg-slate-100 text-slate-900"
                        class="flex flex-col items-center justify-center w-full aspect-square rounded-xl hover:bg-slate-100 transition-colors text-slate-500 hover:text-slate-900 group">
                        <PhList :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium">酒馆选项</span>
                    </router-link>
                    <router-link to="/versions" active-class="bg-slate-100 text-slate-900"
                        class="flex flex-col items-center justify-center w-full aspect-square rounded-xl hover:bg-slate-100 transition-colors text-slate-500 hover:text-slate-900 group">
                        <PhClock :size="24" weight="duotone"
                            class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium">版本管理</span>
                    </router-link>
                    <router-link to="/extensions" active-class="bg-slate-100 text-slate-900"
                        class="flex flex-col items-center justify-center w-full aspect-square rounded-xl hover:bg-slate-100 transition-colors text-slate-500 hover:text-slate-900 group">
                        <PhPlug :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium">扩展管理</span>
                    </router-link>
                    <router-link to="/tools" active-class="bg-slate-100 text-slate-900"
                        class="flex flex-col items-center justify-center w-full aspect-square rounded-xl hover:bg-slate-100 transition-colors text-slate-500 hover:text-slate-900 group">
                        <PhWrench :size="24" weight="duotone"
                            class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium">小工具</span>
                    </router-link>
                </div>

                <!-- Bottom Menu -->
                <div class="flex flex-col gap-2 px-3">
                    <router-link to="/console" active-class="bg-slate-100 text-slate-900"
                        class="flex flex-col items-center justify-center w-full aspect-square rounded-xl hover:bg-slate-100 transition-colors text-slate-500 hover:text-slate-900 group">
                        <PhTerminalWindow :size="24" weight="duotone"
                            class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium">控制台</span>
                    </router-link>
                    <router-link to="/settings" active-class="bg-slate-100 text-slate-900"
                        class="flex flex-col items-center justify-center w-full aspect-square rounded-xl hover:bg-slate-100 transition-colors text-slate-500 hover:text-slate-900 group">
                        <PhGear :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium">设置</span>
                    </router-link>
                </div>
            </aside>

            <!-- Main Content -->
            <main class="flex-1 relative bg-slate-50/50 overflow-y-auto">
                <div class="max-w-4xl mx-auto px-6 py-10 pb-24 h-full">
                    <slot></slot>
                </div>
            </main>

            <slot name="Modal"></slot>
        </div>
    </div>
</template>

<style scoped>
/* Titlebar */
.app-titlebar {
    -webkit-app-region: drag !important;
    background: rgba(255, 255, 255, 0.8);
    backdrop-filter: blur(20px);
    border-bottom: 1px solid rgba(226, 232, 240, 0.8);
    position: relative;
    font-family: var(--font-main) !important;
}

.app-titlebar button {
    -webkit-app-region: no-drag;
}

.app-titlebar a {
    -webkit-app-region: no-drag;
}

/* Navigation Tabs */
.nav-tab {
    position: relative;
    transition: all 0.3s var(--ease-spring);
    display: flex;
    align-items: center;
    justify-content: center;
}

.active {
    color: #4DB7FF;
}
</style>

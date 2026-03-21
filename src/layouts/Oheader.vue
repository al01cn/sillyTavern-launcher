<script lang="ts" setup>
import { onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { PhMinus, PhX, PhPlay, PhList, PhClock, PhPlug, PhWrench, PhFolderOpen, PhTerminalWindow, PhGear } from '@phosphor-icons/vue';
import config from '../lib/config'
import { getCurrentWindow } from '@tauri-apps/api/window';
import { installState } from '../lib/useInstall';
import { Dialog } from '../lib/useDialog';
import { consoleStatus } from '../lib/consoleState';

const { t } = useI18n();
const appWindow = getCurrentWindow();
let unlistenClose: (() => void) | null = null;

const checkCanClose = () => {
    if (installState.show && ['downloading', 'extracting', 'installing', 'deleting'].includes(installState.status)) {
        Dialog.warning({
            title: t('common.warning'),
            msg: '正在下载或删除版本，请等待完成',
            showCancel: false,
            confirmText: t('common.iKnow')
        });
        return false;
    }
    return true;
};

const close = async () => {
    if (!checkCanClose()) return;
    await appWindow.close();
};

const minimize = async () => {
    await appWindow.minimize();
};

onMounted(async () => {
    unlistenClose = await appWindow.onCloseRequested(async (event) => {
        if (!checkCanClose()) {
            event.preventDefault();
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
    <div class="flex flex-col h-screen w-screen overflow-hidden bg-white dark:bg-slate-900">
        <!-- 1. Header & Navigation -->
        <header class="app-titlebar h-14 shrink-0 flex items-center justify-between px-6 z-60">
            <div class="flex items-center gap-2.5 w-40">
                <div class="w-8 h-8 rounded-lg flex items-center justify-center text-white">
                    <img :src="config.appIcon" alt="logo">
                </div>
                <span class="font-black text-sm tracking-tight text-slate-800 dark:text-slate-200 text-nowrap">{{ config.appName }}</span>
            </div>

            <div class="flex items-center gap-1">
                <div class="flex items-center w-40 justify-end h-full gap-1">
                    <button @click="minimize()"
                        class="h-8 w-8 rounded-lg flex items-center justify-center text-slate-400 dark:text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors">
                        <PhMinus class="w-4 h-4" />
                    </button>
                    <button @click="close()"
                        :class="`h-8 w-8 rounded-lg flex items-center justify-center text-slate-400 dark:text-slate-500 hover:bg-red-500 hover:text-white transition-colors`">
                        <PhX class="w-4 h-4" />
                    </button>
                </div>
            </div>
        </header>

        <!-- 2. Body (Sidebar & Content) -->
        <div class="flex flex-1 overflow-hidden relative">
            <!-- Sidebar -->
            <aside class="w-24 shrink-0 border-r border-slate-200/80 dark:border-slate-700/80 bg-white dark:bg-slate-900 flex flex-col justify-between py-4 z-50">
                <!-- Top Menu -->
                <div class="flex flex-col gap-2 px-3">
                    <router-link to="/" active-class="bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400"
                        class="flex flex-col items-center justify-center w-full aspect-square rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors text-slate-500 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100 group">
                        <PhPlay :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.quickStart') }}</span>
                    </router-link>
                    <router-link to="/tavern" active-class="bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400"
                        class="flex flex-col items-center justify-center w-full aspect-square rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors text-slate-500 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100 group">
                        <PhList :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.tavernOptions') }}</span>
                    </router-link>
                    <router-link to="/versions" active-class="bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400"
                        class="flex flex-col items-center justify-center w-full aspect-square rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors text-slate-500 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100 group">
                        <PhClock :size="24" weight="duotone"
                            class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.versionManagement') }}</span>
                    </router-link>
                    <router-link to="/extensions" active-class="bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400"
                        class="flex flex-col items-center justify-center w-full aspect-square rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors text-slate-500 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100 group">
                        <PhPlug :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.extensionManagement') }}</span>
                    </router-link>
                    <router-link to="/resources" active-class="bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400"
                        class="flex flex-col items-center justify-center w-full aspect-square rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors text-slate-500 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100 group">
                        <PhFolderOpen :size="24" weight="duotone"
                            class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.resourceManagement') }}</span>
                    </router-link>
                    <router-link to="/tools" active-class="bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400"
                        class="flex flex-col items-center justify-center w-full aspect-square rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors text-slate-500 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100 group">
                        <PhWrench :size="24" weight="duotone"
                            class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.smallTools') }}</span>
                    </router-link>
                </div>

                <!-- Bottom Menu -->
                <div class="flex flex-col gap-2 px-3">
                    <router-link to="/console"
                        :active-class="consoleStatus === 2 ? 'bg-emerald-50 dark:bg-emerald-900/30 text-emerald-600 dark:text-emerald-400' : 'bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400'"
                        :class="[
                            'flex flex-col items-center justify-center w-full aspect-square rounded-xl transition-colors group',
                            consoleStatus === 2
                                ? 'text-emerald-500 dark:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-900/30 hover:text-emerald-600 dark:hover:text-emerald-300'
                                : 'text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/30 hover:text-red-600 dark:hover:text-red-300'
                        ]">
                        <PhTerminalWindow :size="24" weight="duotone"
                            class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.console') }}</span>
                    </router-link>
                    <router-link to="/settings" active-class="bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400"
                        class="flex flex-col items-center justify-center w-full aspect-square rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors text-slate-500 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100 group">
                        <PhGear :size="24" weight="duotone" class="mb-1.5 group-hover:scale-110 transition-transform" />
                        <span class="text-[11px] font-medium text-center leading-tight">{{ t('nav.settings') }}</span>
                    </router-link>
                </div>
            </aside>

            <!-- Main Content -->
            <main class="flex-1 relative bg-slate-50/50 dark:bg-slate-950/50 overflow-y-auto">
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

.dark .app-titlebar {
    background: rgba(15, 23, 42, 0.8);
    border-bottom: 1px solid rgba(51, 65, 85, 0.8);
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

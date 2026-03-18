<script lang="ts" setup>
import { Minus, X } from 'lucide-vue-next';
import config from '../lib/config'
import GlobalDialog from '../components/GlobalDialog.vue';
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

const close = async () => {
    await appWindow.close();
};

const minimize = async () => {
    await appWindow.minimize();
};

</script>

<template>
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
                    <Minus class="w-4 h-4" />
                </button>
                <button @click="close()"
                    :class="`h-8 w-8 rounded-lg flex items-center justify-center text-slate-400 hover:bg-red-500 hover:text-white transition-colors`">
                    <X class="w-4 h-4" />
                </button>
            </div>
        </div>

    </header>

    <main class="flex-1 relative bg-slate-50/50 h-screen">
        <div class="max-w-4xl mx-auto px-6 py-10 pb-24 overflow-y-auto">

        </div>

        <!-- 全局确认框 -->
        <GlobalDialog />
    </main>
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

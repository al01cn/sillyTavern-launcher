<script setup lang="ts">
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'vue-sonner';
import { useI18n } from 'vue-i18n';
import {
    PhX, PhArrowRight, PhFileArrowDown, PhCircleNotch,
    PhCheckCircle, PhWarningCircle, PhFolderOpen
} from '@phosphor-icons/vue';

const { t } = useI18n();

interface MigrateSource {
    path: string;
    tavernPath: string;
    version: string;
    display: string;
}

const props = defineProps<{
    open: boolean;
}>();

const emit = defineEmits<{
    (e: 'close'): void;
    (e: 'migrated'): void;
}>();

const sources = ref<MigrateSource[]>([]);
const selectedPath = ref<string>('');
const loading = ref(false);
const migrating = ref(false);
const noSources = ref(false);

const loadSources = async () => {
    loading.value = true;
    noSources.value = false;
    sources.value = [];
    selectedPath.value = '';
    try {
        const list = await invoke<MigrateSource[]>('list_config_migration_sources');
        sources.value = list;
        if (list.length === 0) {
            noSources.value = true;
        }
    } catch (e: any) {
        toast.error(typeof e === 'string' ? e : t('tavern.migrate.loadFailed'));
    } finally {
        loading.value = false;
    }
};

watch(() => props.open, (v) => {
    if (v) loadSources();
});

const handleClose = () => {
    if (migrating.value) return;
    emit('close');
};

const handleMigrate = async () => {
    if (!selectedPath.value) {
        toast.warning(t('tavern.migrate.selectFirst'));
        return;
    }
    migrating.value = true;
    try {
        await invoke('migrate_tavern_config', { sourcePath: selectedPath.value });
        toast.success(t('tavern.migrate.success'));
        emit('migrated');
        emit('close');
    } catch (e: any) {
        toast.error(typeof e === 'string' ? e : t('tavern.migrate.failed'));
    } finally {
        migrating.value = false;
    }
};
</script>

<template>
    <!-- Backdrop -->
    <Teleport to="body">
        <Transition name="dialog-fade">
            <div v-if="open" class="fixed inset-0 z-50 flex items-center justify-center">
                <!-- Overlay -->
                <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" @click="handleClose" />

                <!-- Dialog -->
                <div
                    class="relative z-10 w-full max-w-lg mx-4 bg-white dark:bg-slate-900 rounded-2xl shadow-2xl border border-slate-200 dark:border-slate-700 overflow-hidden">
                    <!-- Header -->
                    <div
                        class="flex items-center justify-between px-5 py-4 border-b border-slate-100 dark:border-slate-800 bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-slate-800 dark:to-slate-800">
                        <div class="flex items-center gap-3">
                            <div
                                class="w-9 h-9 bg-blue-600 rounded-xl flex items-center justify-center shadow-md shadow-blue-500/25">
                                <PhFileArrowDown :size="18" weight="fill" class="text-white" />
                            </div>
                            <div>
                                <h2 class="text-sm font-bold text-slate-800 dark:text-slate-100">
                                    {{ t('tavern.migrate.title') }}
                                </h2>
                                <p class="text-[11px] text-slate-500 dark:text-slate-400">
                                    {{ t('tavern.migrate.subtitle') }}
                                </p>
                            </div>
                        </div>
                        <button @click="handleClose" :disabled="migrating"
                            class="w-7 h-7 flex items-center justify-center rounded-lg text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all active:scale-95 disabled:opacity-40">
                            <PhX :size="16" weight="bold" />
                        </button>
                    </div>

                    <!-- Body -->
                    <div class="p-5 space-y-4">
                        <!-- 描述 -->
                        <p class="text-[12px] text-slate-500 dark:text-slate-400 leading-relaxed">
                            {{ t('tavern.migrate.desc') }}
                        </p>

                        <!-- 加载中 -->
                        <div v-if="loading" class="flex items-center justify-center py-10">
                            <PhCircleNotch :size="28" class="animate-spin text-blue-500" />
                        </div>

                        <!-- 无可用来源 -->
                        <div v-else-if="noSources"
                            class="flex flex-col items-center justify-center py-8 gap-3 text-center">
                            <PhWarningCircle :size="36" class="text-amber-400" weight="duotone" />
                            <p class="text-sm font-bold text-slate-600 dark:text-slate-300">
                                {{ t('tavern.migrate.noSources') }}
                            </p>
                            <p class="text-[11px] text-slate-400 dark:text-slate-500">
                                {{ t('tavern.migrate.noSourcesDesc') }}
                            </p>
                        </div>

                        <!-- 来源列表 -->
                        <div v-else class="space-y-2 max-h-72 overflow-y-auto pr-1">
                            <label v-for="src in sources" :key="src.path"
                                class="flex items-start gap-3 p-3 rounded-xl border-2 cursor-pointer transition-all select-none"
                                :class="selectedPath === src.path
                                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                                    : 'border-slate-200 dark:border-slate-700 hover:border-blue-300 dark:hover:border-blue-700 bg-white dark:bg-slate-800'">
                                <input type="radio" :value="src.path" v-model="selectedPath" class="mt-0.5 accent-blue-600 shrink-0" />
                                <div class="flex flex-col gap-0.5 min-w-0">
                                    <span class="text-sm font-bold text-slate-700 dark:text-slate-200 truncate">
                                        {{ src.version || t('tavern.migrate.unknownVersion') }}
                                    </span>
                                    <span class="text-[11px] text-slate-400 dark:text-slate-500 truncate flex items-center gap-1">
                                        <PhFolderOpen :size="11" />
                                        {{ src.path }}
                                    </span>
                                </div>
                                <PhCheckCircle v-if="selectedPath === src.path"
                                    :size="18" weight="fill"
                                    class="text-blue-500 shrink-0 ml-auto mt-0.5" />
                            </label>
                        </div>

                        <!-- 目标说明 -->
                        <div v-if="!loading && !noSources"
                            class="flex items-center gap-2 p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-700/50 rounded-xl">
                            <PhWarningCircle :size="16" weight="duotone" class="text-amber-500 shrink-0" />
                            <p class="text-[11px] text-amber-700 dark:text-amber-300">
                                {{ t('tavern.migrate.overwriteWarning') }}
                            </p>
                        </div>
                    </div>

                    <!-- Footer -->
                    <div
                        class="flex items-center justify-end gap-3 px-5 py-4 border-t border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/50">
                        <button @click="handleClose" :disabled="migrating"
                            class="px-4 py-2 text-sm font-bold text-slate-600 dark:text-slate-300 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl hover:border-slate-300 dark:hover:border-slate-600 transition-all active:scale-95 disabled:opacity-40">
                            {{ t('common.cancel') }}
                        </button>
                        <button @click="handleMigrate"
                            :disabled="migrating || !selectedPath || loading || noSources"
                            class="flex items-center gap-2 px-4 py-2 text-sm font-bold text-white bg-blue-600 rounded-xl hover:bg-blue-700 transition-all active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed shadow-md shadow-blue-500/20">
                            <PhCircleNotch v-if="migrating" :size="15" class="animate-spin" />
                            <PhArrowRight v-else :size="15" weight="bold" />
                            {{ migrating ? t('tavern.migrate.migrating') : t('tavern.migrate.confirm') }}
                        </button>
                    </div>
                </div>
            </div>
        </Transition>
    </Teleport>
</template>

<style scoped>
.dialog-fade-enter-active,
.dialog-fade-leave-active {
    transition: opacity 0.2s ease;
}

.dialog-fade-enter-from,
.dialog-fade-leave-to {
    opacity: 0;
}

.dialog-fade-enter-active .relative,
.dialog-fade-leave-active .relative {
    transition: transform 0.2s ease, opacity 0.2s ease;
}

.dialog-fade-enter-from .relative,
.dialog-fade-leave-to .relative {
    transform: scale(0.95) translateY(8px);
    opacity: 0;
}
</style>

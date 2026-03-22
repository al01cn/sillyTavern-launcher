<script lang="ts" setup>
import { ref, watch, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { uploadCharacterCardState, closeUploadCharacterCardDialog } from '../lib/useUploadCharacterCard';
import { X, UploadCloud, FileImage, CheckCircle2, AlertTriangle, Loader2 } from 'lucide-vue-next';
import { toast } from 'vue-sonner';
import { getCharacterInfo } from 'gstinfo';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const isImporting = ref(false);

interface CharacterCardFile {
    id: string;
    path?: string; // 改为可选，因为前端拖拽没有实际路径
    name: string;
    status: 'pending' | 'verifying' | 'valid' | 'invalid' | 'importing' | 'success' | 'error';
    characterName?: string;
    errorMsg?: string;
    rawFile?: File; // 保存前端拖拽的文件对象
}

const selectedFiles = ref<CharacterCardFile[]>([]);

watch(() => uploadCharacterCardState.show, (newVal) => {
    if (newVal) {
        resetState();
    }
});

const resetState = () => {
    selectedFiles.value = [];
    isImporting.value = false;
};

const dragging = ref(false);

const onDragOver = (e: DragEvent) => {
    e.preventDefault();
    dragging.value = true;
};

const onDragLeave = (e: DragEvent) => {
    e.preventDefault();
    dragging.value = false;
};

const onDrop = (e: DragEvent) => {
    e.preventDefault();
    dragging.value = false;

    const files = Array.from(e.dataTransfer?.files || []);
    if (files.length === 0) return;

    // 前端拖拽的文件，没有实际系统路径
    const newFiles: CharacterCardFile[] = files.map(file => ({
        id: Math.random().toString(36).substring(2, 9),
        name: file.name,
        status: 'verifying',
        rawFile: file // 保存前端 File 对象
    }));

    processFiles(newFiles);
};

const handleFilesSelected = (filePaths: string[]) => {
    // 弹窗选择的文件，拥有系统绝对路径
    const newFiles: CharacterCardFile[] = filePaths.map(filePath => ({
        id: Math.random().toString(36).substring(2, 9),
        path: filePath,
        name: filePath.split(/[/\\]/).pop() || 'unknown.png',
        status: 'verifying'
    }));

    processFiles(newFiles);
};

const processFiles = async (newFiles: CharacterCardFile[]) => {
    selectedFiles.value = newFiles;

    for (const file of selectedFiles.value) {
        try {
            let info;

            if (file.rawFile) {
                // 1. 拖拽的文件：直接通过前端 File 对象解析卡片信息
                info = await getCharacterInfo(file.rawFile);
            } else if (file.path) {
                // 2. 弹窗选择的文件：通过 Rust 读取文件字节后再解析
                const bytes = await invoke<number[]>('read_local_file', { path: file.path });
                const u8 = new Uint8Array(bytes);
                info = await getCharacterInfo(u8);
            }

            if (!info || !info.name) {
                throw new Error(t('resources.noValidCardData'));
            }

            file.characterName = info.name;
            file.status = 'valid';
        } catch (e: any) {
            file.errorMsg = e?.message || String(e);
            file.status = 'invalid';
        }
    }
};

const selectFile = async () => {
    if (isImporting.value) return;

    try {
        const selected = await open({
            multiple: true,
            filters: [{
                name: 'Image',
                extensions: ['png']
            }]
        });

        if (selected) {
            if (Array.isArray(selected)) {
                const paths = selected.map(s => typeof s === 'string' ? s : (s as any).path);
                handleFilesSelected(paths);
            } else if (typeof selected === 'string') {
                handleFilesSelected([selected]);
            } else if (typeof selected === 'object' && 'path' in (selected as any)) {
                handleFilesSelected([(selected as any).path as string]);
            }
        }
    } catch (e) {
        console.error('选择文件失败:', e);
    }
};

const hasValidFiles = computed(() => {
    return selectedFiles.value.some(f => f.status === 'valid');
});

const isAnyVerifying = computed(() => {
    return selectedFiles.value.some(f => f.status === 'verifying');
});

const importCards = async () => {
    const validFiles = selectedFiles.value.filter(f => f.status === 'valid');
    if (validFiles.length === 0 || isImporting.value || isAnyVerifying.value) return;

    isImporting.value = true;
    let successCount = 0;

    for (const file of validFiles) {
        file.status = 'importing';
        try {
            if (file.rawFile) {
                // 【分支1】拖拽上传：读取 File 为 ArrayBuffer，发送给 Rust 进行保存
                const arrayBuffer = await file.rawFile.arrayBuffer();
                const bytes = Array.from(new Uint8Array(arrayBuffer));
                
                // 注意：这里需要你在 Rust 端实现一个对应的 command，例如 `import_character_card_from_bytes`
                // 接收 `bytes: Vec<u8>` 和 `filename: String` 并保存到目标目录
                await invoke('import_character_card_from_bytes', { 
                    bytes: bytes,
                    filename: file.name
                });
            } else if (file.path) {
                // 【分支2】点击选择：传递路径给 Rust 让其去复制
                await invoke('import_character_card', { sourcePath: file.path });
            }
            
            file.status = 'success';
            successCount++;
        } catch (e) {
            file.status = 'error';
            file.errorMsg = String(e);
        }
    }

    isImporting.value = false;

    if (successCount > 0) {
        toast.success(t('resources.importSuccessMsg', { count: successCount, type: t('resources.characterCard') }));
        window.dispatchEvent(new Event('character-card-imported'));
        if (uploadCharacterCardState.onSuccess) {
            uploadCharacterCardState.onSuccess();
        }

        if (successCount === validFiles.length) {
            setTimeout(() => {
                closeUploadCharacterCardDialog();
            }, 1000);
        }
    } else {
        toast.error(t('resources.importFailedMsg', { type: t('resources.characterCard') }));
    }
};
</script>

<template>
    <div :class="[
        'absolute inset-0 z-[300] flex items-center justify-center px-4 transition-all duration-300',
        uploadCharacterCardState.show ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'
    ]">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-md"
            @click="!isImporting && closeUploadCharacterCardDialog()"></div>

        <!-- Modal Content -->
        <div :class="[
            'modal-content relative bg-white dark:bg-slate-800 w-full max-w-lg rounded-3xl shadow-2xl border border-slate-100 dark:border-slate-700 overflow-hidden transition-all duration-300 transform flex flex-col',
            uploadCharacterCardState.show ? 'scale-100 translate-y-0' : 'scale-95 translate-y-8'
        ]">
            <!-- Header -->
            <div class="px-6 py-5 border-b border-slate-100 dark:border-slate-700 flex items-center justify-between">
                <h3 class="text-lg font-bold text-slate-800 dark:text-slate-100">{{ t('resources.uploadCardTitle') }}
                </h3>
                <button @click="!isImporting && closeUploadCharacterCardDialog()" :disabled="isImporting"
                    class="p-1.5 text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 rounded-xl transition-colors disabled:opacity-50">
                    <X class="w-5 h-5" />
                </button>
            </div>

            <!-- Body -->
            <div class="p-6 space-y-6">
                <!-- Empty State -->
                <div v-if="selectedFiles.length === 0" @dragover="onDragOver" @dragleave="onDragLeave" @drop="onDrop">
                    <label class="block text-sm font-bold text-slate-700 dark:text-slate-300 mb-2">{{
                        t('resources.cardImageFile') }}</label>
                    <div @click="selectFile" :class="[
                        'border-2 border-dashed rounded-2xl p-8 text-center transition-all cursor-pointer relative overflow-hidden border-slate-200 dark:border-slate-600 bg-slate-50 dark:bg-slate-700/50 hover:bg-slate-100 dark:hover:bg-slate-700 hover:border-slate-300 dark:hover:border-slate-500',
                        isImporting ? 'pointer-events-none opacity-60' : '',
                        dragging ? 'bg-slate-100 dark:bg-slate-700/50' : ''
                    ]">
                        <div class="flex flex-col items-center pointer-events-none">
                            <div
                                class="w-12 h-12 rounded-full bg-white dark:bg-slate-800 shadow-sm flex items-center justify-center text-slate-400 dark:text-slate-500 mb-3">
                                <UploadCloud class="w-6 h-6" />
                            </div>
                            <p class="text-slate-700 dark:text-slate-300 font-medium">{{ t('resources.clickOrDrag') }}
                            </p>
                            <p class="text-slate-400 dark:text-slate-500 text-xs mt-1">{{
                                t('resources.supportMultipleCards') }}</p>
                        </div>
                    </div>
                </div>

                <!-- Single File UI -->
                <div v-else-if="selectedFiles.length === 1">
                    <label class="block text-sm font-bold text-slate-700 dark:text-slate-300 mb-2">{{
                        t('resources.cardImageFile') }}</label>
                    <div @click="selectFile" :class="[
                        'border-2 border-dashed rounded-2xl p-8 text-center transition-all cursor-pointer relative overflow-hidden border-slate-200 dark:border-slate-600 bg-slate-50 dark:bg-slate-700/50 hover:bg-slate-100 dark:hover:bg-slate-700 hover:border-slate-300 dark:hover:border-slate-500',
                        isImporting ? 'pointer-events-none opacity-60' : ''
                    ]">
                        <div class="flex flex-col items-center pointer-events-none">
                            <div
                                class="w-12 h-12 rounded-full bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center text-blue-500 dark:text-blue-400 mb-3">
                                <FileImage class="w-6 h-6" />
                            </div>
                            <p class="text-slate-700 dark:text-slate-300 font-medium truncate max-w-[250px]">{{
                                selectedFiles[0].name }}</p>
                            <p class="text-blue-500 dark:text-blue-400 text-xs mt-1">{{ t('resources.clickToReselect')
                                }}</p>
                        </div>
                    </div>

                    <!-- Validation Result -->
                    <div class="mt-6 rounded-xl p-4 transition-all" :class="[
                        selectedFiles[0].status === 'verifying' || selectedFiles[0].status === 'importing' ? 'bg-slate-50 dark:bg-slate-700/50' :
                            selectedFiles[0].status === 'invalid' || selectedFiles[0].status === 'error' ? 'bg-red-50 dark:bg-red-900/20' : 'bg-emerald-50 dark:bg-emerald-900/20'
                    ]">
                        <div v-if="selectedFiles[0].status === 'verifying'"
                            class="flex items-center gap-3 text-slate-500 dark:text-slate-400">
                            <Loader2 class="w-5 h-5 animate-spin" />
                            <span class="text-sm font-medium">{{ t('resources.verifyingCard') }}</span>
                        </div>

                        <div v-else-if="selectedFiles[0].status === 'invalid'"
                            class="flex items-start gap-3 text-red-600 dark:text-red-400">
                            <AlertTriangle class="w-5 h-5 shrink-0 mt-0.5" />
                            <div>
                                <p class="text-sm font-bold">{{ t('resources.invalidCard') }}</p>
                                <p class="text-xs mt-1 opacity-80">{{ selectedFiles[0].errorMsg }}</p>
                            </div>
                        </div>

                        <div v-else-if="selectedFiles[0].status === 'error'"
                            class="flex items-start gap-3 text-red-600 dark:text-red-400">
                            <X class="w-5 h-5 shrink-0 mt-0.5" />
                            <div>
                                <p class="text-sm font-bold">{{ t('resources.importFailed') }}</p>
                                <p class="text-xs mt-1 opacity-80">{{ selectedFiles[0].errorMsg }}</p>
                            </div>
                        </div>

                        <div v-else-if="selectedFiles[0].status === 'importing'"
                            class="flex items-center gap-3 text-blue-600 dark:text-blue-400">
                            <Loader2 class="w-5 h-5 animate-spin" />
                            <span class="text-sm font-medium">{{ t('resources.importing') }}...</span>
                        </div>

                        <div v-else-if="selectedFiles[0].status === 'success'"
                            class="flex items-center gap-3 text-emerald-600 dark:text-emerald-400">
                            <CheckCircle2 class="w-5 h-5" />
                            <span class="text-sm font-medium">{{ t('resources.importSuccess') }}</span>
                        </div>

                        <div v-else-if="selectedFiles[0].status === 'valid'"
                            class="flex items-start gap-3 text-emerald-700 dark:text-emerald-400">
                            <CheckCircle2 class="w-5 h-5 shrink-0 mt-0.5" />
                            <div class="flex-1 min-w-0">
                                <p class="text-sm font-bold">{{ t('resources.validCard') }}</p>
                                <div class="mt-2 space-y-1 text-xs opacity-90">
                                    <p v-if="selectedFiles[0].characterName"><span class="font-semibold">{{
                                        t('resources.characterName') }}：</span>{{ selectedFiles[0].characterName }}
                                    </p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Multiple Files UI -->
                <div v-else-if="selectedFiles.length > 1" class="space-y-4">
                    <div class="flex items-center justify-between">
                        <label class="block text-sm font-bold text-slate-700 dark:text-slate-300">{{
                            t('resources.selectedImages', { count: selectedFiles.length }) }}</label>
                        <button @click="selectFile" :disabled="isImporting"
                            class="text-xs text-blue-500 dark:text-blue-400 hover:text-blue-600 dark:hover:text-blue-300 font-medium disabled:opacity-50">
                            {{ t('resources.reselect') }}
                        </button>
                    </div>

                    <div class="max-h-[240px] overflow-y-auto pr-2 space-y-2 custom-scrollbar">
                        <div v-for="file in selectedFiles" :key="file.id"
                            class="bg-slate-50 dark:bg-slate-700/50 rounded-xl p-3 flex items-center justify-between border border-slate-100 dark:border-slate-600">
                            <div class="flex items-center gap-3 min-w-0 flex-1">
                                <div
                                    class="w-8 h-8 rounded-lg bg-white dark:bg-slate-800 flex items-center justify-center text-slate-400 dark:text-slate-500 shrink-0">
                                    <FileImage class="w-4 h-4" />
                                </div>
                                <div class="min-w-0 flex-1">
                                    <p class="text-sm font-bold text-slate-700 dark:text-slate-300 truncate"
                                        :title="file.characterName || file.name">
                                        {{ file.characterName || file.name }}
                                    </p>
                                    <p class="text-[10px] text-red-500 dark:text-red-400 truncate"
                                        v-if="file.status === 'invalid' || file.status === 'error'"
                                        :title="file.errorMsg">
                                        {{ file.errorMsg }}
                                    </p>
                                    <p class="text-[10px] text-slate-400 dark:text-slate-500 truncate" v-else>
                                        {{ file.name }}
                                    </p>
                                </div>
                            </div>

                            <!-- Status Indicator -->
                            <div class="shrink-0 ml-3 flex items-center justify-end min-w-[70px]">
                                <div v-if="file.status === 'verifying'"
                                    class="flex items-center gap-1.5 text-slate-400 dark:text-slate-500 text-xs">
                                    <Loader2 class="w-3.5 h-3.5 animate-spin" />
                                    <span>{{ t('resources.verifying') }}</span>
                                </div>
                                <div v-else-if="file.status === 'invalid'"
                                    class="flex items-center gap-1.5 text-red-500 dark:text-red-400 text-xs"
                                    :title="t('resources.invalid')">
                                    <AlertTriangle class="w-3.5 h-3.5" />
                                    <span>{{ t('resources.invalid') }}</span>
                                </div>
                                <div v-else-if="file.status === 'valid'"
                                    class="flex items-center gap-1.5 text-emerald-500 dark:text-emerald-400 text-xs">
                                    <CheckCircle2 class="w-3.5 h-3.5" />
                                    <span>{{ t('resources.pending') }}</span>
                                </div>
                                <div v-else-if="file.status === 'importing'"
                                    class="flex items-center gap-1.5 text-blue-500 dark:text-blue-400 text-xs">
                                    <Loader2 class="w-3.5 h-3.5 animate-spin" />
                                    <span>{{ t('resources.importing') }}</span>
                                </div>
                                <div v-else-if="file.status === 'success'"
                                    class="flex items-center gap-1.5 text-emerald-500 dark:text-emerald-400 text-xs">
                                    <CheckCircle2 class="w-3.5 h-3.5" />
                                    <span>{{ t('resources.importSuccess') }}</span>
                                </div>
                                <div v-else-if="file.status === 'error'"
                                    class="flex items-center gap-1.5 text-red-500 dark:text-red-400 text-xs"
                                    :title="file.errorMsg">
                                    <X class="w-3.5 h-3.5" />
                                    <span>{{ t('resources.importFailed') }}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Footer -->
            <div
                class="p-5 border-t border-slate-100 dark:border-slate-700 bg-slate-50 dark:bg-slate-800/50 flex justify-end gap-3">
                <button @click="closeUploadCharacterCardDialog" :disabled="isImporting"
                    class="px-5 py-2.5 rounded-xl font-bold text-slate-500 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors disabled:opacity-50 text-sm">
                    {{ t('common.cancel') }}
                </button>
                <button @click="importCards" :disabled="!hasValidFiles || isAnyVerifying || isImporting"
                    class="px-5 py-2.5 rounded-xl font-bold text-white transition-all shadow-md text-sm flex items-center gap-2"
                    :class="[
                        (!hasValidFiles || isAnyVerifying || isImporting)
                            ? 'bg-blue-300 cursor-not-allowed shadow-none'
                            : 'bg-blue-500 hover:bg-blue-600 active:scale-95 shadow-blue-500/20'
                    ]">
                    <Loader2 v-if="isImporting" class="w-4 h-4 animate-spin" />
                    <span>{{ isImporting ? t('resources.importing') + '...' : t('resources.startImport') }}</span>
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
    background-color: #cbd5e1;
    border-radius: 20px;
}
</style>
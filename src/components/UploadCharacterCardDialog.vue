<script lang="ts" setup>
import { ref, watch, onUnmounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { uploadCharacterCardState, closeUploadCharacterCardDialog } from '../lib/useUploadCharacterCard';
import { X, UploadCloud, FileImage, CheckCircle2, AlertTriangle, Loader2 } from 'lucide-vue-next';
import { toast } from 'vue-sonner';
import { UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { getCharacterInfo } from 'gstinfo';

const isDragging = ref(false);
const isImporting = ref(false);

interface CharacterCardFile {
    id: string;
    path: string;
    name: string;
    status: 'pending' | 'verifying' | 'valid' | 'invalid' | 'importing' | 'success' | 'error';
    characterName?: string;
    errorMsg?: string;
}

const selectedFiles = ref<CharacterCardFile[]>([]);

let unlistenDrop: UnlistenFn | null = null;

watch(() => uploadCharacterCardState.show, async (newVal) => {
    if (newVal) {
        resetState();
        setupDragDrop();
    } else {
        cleanupDragDrop();
    }
});

const resetState = () => {
    selectedFiles.value = [];
    isImporting.value = false;
    isDragging.value = false;
};

const setupDragDrop = async () => {
    const appWindow = getCurrentWebviewWindow();
    
    unlistenDrop = await appWindow.onDragDropEvent((event) => {
        if (!uploadCharacterCardState.show || isImporting.value) return;
        
        if (event.payload.type === 'drop') {
            isDragging.value = false;
            const paths = event.payload.paths;
            if (paths && paths.length > 0) {
                const pngPaths = paths.filter(p => p.toLowerCase().endsWith('.png'));
                if (pngPaths.length > 0) {
                    handleFilesSelected(pngPaths);
                } else {
                    toast.error('请上传 .png 格式的角色卡图片');
                }
            }
        } else if (event.payload.type === 'enter') {
            isDragging.value = true;
        } else if (event.payload.type === 'leave') {
            isDragging.value = false;
        }
    });
};

const cleanupDragDrop = () => {
    if (unlistenDrop) {
        unlistenDrop();
        unlistenDrop = null;
    }
};

onUnmounted(() => {
    cleanupDragDrop();
});

const handleFilesSelected = async (filePaths: string[]) => {
    const newFiles: CharacterCardFile[] = filePaths.map(filePath => ({
        id: Math.random().toString(36).substring(2, 9),
        path: filePath,
        name: filePath.split(/[/\\]/).pop() || 'unknown.png',
        status: 'verifying'
    }));

    selectedFiles.value = newFiles;

    for (const file of selectedFiles.value) {
        try {
            const bytes = await invoke<number[]>('read_local_file', { path: file.path });
            const u8 = new Uint8Array(bytes);
            
            const info = await getCharacterInfo(u8);
            if (!info || !info.name) {
                throw new Error('该图片不包含有效的角色卡数据');
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
            await invoke('import_character_card', { sourcePath: file.path });
            file.status = 'success';
            successCount++;
        } catch (e) {
            file.status = 'error';
            file.errorMsg = String(e);
        }
    }
    
    isImporting.value = false;

    if (successCount > 0) {
        toast.success(`成功导入 ${successCount} 个角色卡`);
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
        toast.error('导入角色卡失败，请查看详细信息');
    }
};

</script>

<template>
    <div :class="[
        'absolute inset-0 z-[300] flex items-center justify-center px-4 transition-all duration-300',
        uploadCharacterCardState.show ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'
    ]">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-md" @click="!isImporting && closeUploadCharacterCardDialog()"></div>

        <!-- Modal Content -->
        <div :class="[
            'modal-content relative bg-white w-full max-w-lg rounded-3xl shadow-2xl border border-slate-100 overflow-hidden transition-all duration-300 transform flex flex-col',
            uploadCharacterCardState.show ? 'scale-100 translate-y-0' : 'scale-95 translate-y-8'
        ]">
            <!-- Header -->
            <div class="px-6 py-5 border-b border-slate-100 flex items-center justify-between">
                <h3 class="text-lg font-bold text-slate-800">添加角色卡</h3>
                <button 
                    @click="!isImporting && closeUploadCharacterCardDialog()"
                    :disabled="isImporting"
                    class="p-1.5 text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-xl transition-colors disabled:opacity-50"
                >
                    <X class="w-5 h-5" />
                </button>
            </div>

            <!-- Body -->
            <div class="p-6 space-y-6">
                <!-- Empty / Dragging State -->
                <div v-if="selectedFiles.length === 0 || isDragging">
                    <label class="block text-sm font-bold text-slate-700 mb-2">角色卡图片 (.png)</label>
                    <div 
                        @click="selectFile"
                        :class="[
                            'border-2 border-dashed rounded-2xl p-8 text-center transition-all cursor-pointer relative overflow-hidden',
                            isDragging ? 'border-blue-500 bg-blue-50' : 'border-slate-200 bg-slate-50 hover:bg-slate-100 hover:border-slate-300',
                            isImporting ? 'pointer-events-none opacity-60' : ''
                        ]"
                    >
                        <div class="flex flex-col items-center pointer-events-none">
                            <div class="w-12 h-12 rounded-full bg-white shadow-sm flex items-center justify-center text-slate-400 mb-3">
                                <UploadCloud class="w-6 h-6" />
                            </div>
                            <p class="text-slate-700 font-medium">点击选择或拖拽文件到此处</p>
                            <p class="text-slate-400 text-xs mt-1">支持选择多个 .png 格式的角色卡图片</p>
                        </div>
                    </div>
                </div>

                <!-- Single File UI -->
                <div v-else-if="selectedFiles.length === 1 && !isDragging">
                    <label class="block text-sm font-bold text-slate-700 mb-2">角色卡图片 (.png)</label>
                    <div 
                        @click="selectFile"
                        :class="[
                            'border-2 border-dashed rounded-2xl p-8 text-center transition-all cursor-pointer relative overflow-hidden border-slate-200 bg-slate-50 hover:bg-slate-100 hover:border-slate-300',
                            isImporting ? 'pointer-events-none opacity-60' : ''
                        ]"
                    >
                        <div class="flex flex-col items-center pointer-events-none">
                            <div class="w-12 h-12 rounded-full bg-blue-100 flex items-center justify-center text-blue-500 mb-3">
                                <FileImage class="w-6 h-6" />
                            </div>
                            <p class="text-slate-700 font-medium truncate max-w-[250px]">{{ selectedFiles[0].name }}</p>
                            <p class="text-blue-500 text-xs mt-1">点击重新选择</p>
                        </div>
                    </div>

                    <!-- Validation Result -->
                    <div class="mt-6 rounded-xl p-4 transition-all" :class="[
                        selectedFiles[0].status === 'verifying' || selectedFiles[0].status === 'importing' ? 'bg-slate-50' : 
                        selectedFiles[0].status === 'invalid' || selectedFiles[0].status === 'error' ? 'bg-red-50' : 'bg-emerald-50'
                    ]">
                        <div v-if="selectedFiles[0].status === 'verifying'" class="flex items-center gap-3 text-slate-500">
                            <Loader2 class="w-5 h-5 animate-spin" />
                            <span class="text-sm font-medium">正在验证角色卡...</span>
                        </div>
                        
                        <div v-else-if="selectedFiles[0].status === 'invalid'" class="flex items-start gap-3 text-red-600">
                            <AlertTriangle class="w-5 h-5 shrink-0 mt-0.5" />
                            <div>
                                <p class="text-sm font-bold">无效的角色卡</p>
                                <p class="text-xs mt-1 opacity-80">{{ selectedFiles[0].errorMsg }}</p>
                            </div>
                        </div>

                        <div v-else-if="selectedFiles[0].status === 'error'" class="flex items-start gap-3 text-red-600">
                            <X class="w-5 h-5 shrink-0 mt-0.5" />
                            <div>
                                <p class="text-sm font-bold">导入失败</p>
                                <p class="text-xs mt-1 opacity-80">{{ selectedFiles[0].errorMsg }}</p>
                            </div>
                        </div>

                        <div v-else-if="selectedFiles[0].status === 'importing'" class="flex items-center gap-3 text-blue-600">
                            <Loader2 class="w-5 h-5 animate-spin" />
                            <span class="text-sm font-medium">正在导入...</span>
                        </div>

                        <div v-else-if="selectedFiles[0].status === 'success'" class="flex items-center gap-3 text-emerald-600">
                            <CheckCircle2 class="w-5 h-5" />
                            <span class="text-sm font-medium">导入成功</span>
                        </div>
                        
                        <div v-else-if="selectedFiles[0].status === 'valid'" class="flex items-start gap-3 text-emerald-700">
                            <CheckCircle2 class="w-5 h-5 shrink-0 mt-0.5" />
                            <div class="flex-1 min-w-0">
                                <p class="text-sm font-bold">有效的角色卡</p>
                                <div class="mt-2 space-y-1 text-xs opacity-90">
                                    <p v-if="selectedFiles[0].characterName"><span class="font-semibold">角色名称：</span>{{ selectedFiles[0].characterName }}</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Multiple Files UI -->
                <div v-else-if="selectedFiles.length > 1 && !isDragging" class="space-y-4">
                    <div class="flex items-center justify-between">
                        <label class="block text-sm font-bold text-slate-700">已选择 {{ selectedFiles.length }} 张图片</label>
                        <button @click="selectFile" :disabled="isImporting" class="text-xs text-blue-500 hover:text-blue-600 font-medium disabled:opacity-50">
                            重新选择
                        </button>
                    </div>
                    
                    <div class="max-h-[240px] overflow-y-auto pr-2 space-y-2 custom-scrollbar">
                        <div v-for="file in selectedFiles" :key="file.id" 
                            class="bg-slate-50 rounded-xl p-3 flex items-center justify-between border border-slate-100"
                        >
                            <div class="flex items-center gap-3 min-w-0 flex-1">
                                <div class="w-8 h-8 rounded-lg bg-white flex items-center justify-center text-slate-400 shrink-0">
                                    <FileImage class="w-4 h-4" />
                                </div>
                                <div class="min-w-0 flex-1">
                                    <p class="text-sm font-bold text-slate-700 truncate" :title="file.characterName || file.name">
                                        {{ file.characterName || file.name }}
                                    </p>
                                    <p class="text-[10px] text-red-500 truncate" v-if="file.status === 'invalid' || file.status === 'error'" :title="file.errorMsg">
                                        {{ file.errorMsg }}
                                    </p>
                                    <p class="text-[10px] text-slate-400 truncate" v-else>
                                        {{ file.name }}
                                    </p>
                                </div>
                            </div>
                            
                            <!-- Status Indicator -->
                            <div class="shrink-0 ml-3 flex items-center justify-end min-w-[70px]">
                                <div v-if="file.status === 'verifying'" class="flex items-center gap-1.5 text-slate-400 text-xs">
                                    <Loader2 class="w-3.5 h-3.5 animate-spin" />
                                    <span>验证中</span>
                                </div>
                                <div v-else-if="file.status === 'invalid'" class="flex items-center gap-1.5 text-red-500 text-xs" title="无效的图片">
                                    <AlertTriangle class="w-3.5 h-3.5" />
                                    <span>无效</span>
                                </div>
                                <div v-else-if="file.status === 'valid'" class="flex items-center gap-1.5 text-emerald-500 text-xs">
                                    <CheckCircle2 class="w-3.5 h-3.5" />
                                    <span>待导入</span>
                                </div>
                                <div v-else-if="file.status === 'importing'" class="flex items-center gap-1.5 text-blue-500 text-xs">
                                    <Loader2 class="w-3.5 h-3.5 animate-spin" />
                                    <span>导入中</span>
                                </div>
                                <div v-else-if="file.status === 'success'" class="flex items-center gap-1.5 text-emerald-500 text-xs">
                                    <CheckCircle2 class="w-3.5 h-3.5" />
                                    <span>成功</span>
                                </div>
                                <div v-else-if="file.status === 'error'" class="flex items-center gap-1.5 text-red-500 text-xs" :title="file.errorMsg">
                                    <X class="w-3.5 h-3.5" />
                                    <span>失败</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Footer -->
            <div class="p-5 border-t border-slate-100 bg-slate-50 flex justify-end gap-3">
                <button 
                    @click="closeUploadCharacterCardDialog"
                    :disabled="isImporting"
                    class="px-5 py-2.5 rounded-xl font-bold text-slate-500 hover:bg-slate-200 transition-colors disabled:opacity-50 text-sm"
                >
                    取消
                </button>
                <button 
                    @click="importCards"
                    :disabled="!hasValidFiles || isAnyVerifying || isImporting"
                    class="px-5 py-2.5 rounded-xl font-bold text-white transition-all shadow-md text-sm flex items-center gap-2"
                    :class="[
                        (!hasValidFiles || isAnyVerifying || isImporting)
                            ? 'bg-blue-300 cursor-not-allowed shadow-none'
                            : 'bg-blue-500 hover:bg-blue-600 active:scale-95 shadow-blue-500/20'
                    ]"
                >
                    <Loader2 v-if="isImporting" class="w-4 h-4 animate-spin" />
                    <span>{{ isImporting ? '导入中...' : '开始导入' }}</span>
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

<script lang="ts" setup>
import { ref, watch, onUnmounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { uploadWorldInfoState, closeUploadWorldInfoDialog } from '../lib/useUploadWorldInfo';
import { X, UploadCloud, FileJson, CheckCircle2, AlertTriangle, Loader2 } from 'lucide-vue-next';
import { toast } from 'vue-sonner';
import { UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

const isDragging = ref(false);
const isImporting = ref(false);

interface WorldInfoFile {
    id: string;
    path: string;
    name: string;
    status: 'pending' | 'verifying' | 'valid' | 'invalid' | 'importing' | 'success' | 'error';
    worldName?: string;
    errorMsg?: string;
}

const selectedFiles = ref<WorldInfoFile[]>([]);

let unlistenDrop: UnlistenFn | null = null;

watch(() => uploadWorldInfoState.show, async (newVal) => {
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
        if (!uploadWorldInfoState.show || isImporting.value) return;
        
        if (event.payload.type === 'drop') {
            isDragging.value = false;
            const paths = event.payload.paths;
            if (paths && paths.length > 0) {
                const jsonPaths = paths.filter(p => p.toLowerCase().endsWith('.json'));
                if (jsonPaths.length > 0) {
                    handleFilesSelected(jsonPaths);
                } else {
                    toast.error('请上传 .json 格式的世界书文件');
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
    const newFiles: WorldInfoFile[] = filePaths.map(filePath => ({
        id: Math.random().toString(36).substring(2, 9),
        path: filePath,
        name: filePath.split(/[/\\]/).pop() || 'unknown.json',
        status: 'verifying'
    }));

    selectedFiles.value = newFiles;

    for (const file of selectedFiles.value) {
        try {
            const bytes = await invoke<number[]>('read_local_file', { path: file.path });
            const u8 = new Uint8Array(bytes);
            let content = new TextDecoder('utf-8').decode(u8);
            
            // Remove BOM if present
            if (content.charCodeAt(0) === 0xFEFF) {
                content = content.slice(1);
            }
            // Basic cleanup for trailing non-json characters or weird whitespace
            content = content.trim();

            const data = JSON.parse(content);
            
            const entries = data.entries;
            
            if (!entries || (Array.isArray(entries) && entries.length === 0) || (typeof entries === 'object' && Object.keys(entries).length === 0)) {
                throw new Error('未检测到世界书词条数据');
            }
            
            file.worldName = data.name || file.name;
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
                name: 'JSON',
                extensions: ['json']
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

const importWorlds = async () => {
    const validFiles = selectedFiles.value.filter(f => f.status === 'valid');
    if (validFiles.length === 0 || isImporting.value || isAnyVerifying.value) return;
    
    isImporting.value = true;
    let successCount = 0;

    for (const file of validFiles) {
        file.status = 'importing';
        try {
            await invoke('import_world_info', { sourcePath: file.path });
            file.status = 'success';
            successCount++;
        } catch (e) {
            file.status = 'error';
            file.errorMsg = String(e);
        }
    }
    
    isImporting.value = false;

    if (successCount > 0) {
        toast.success(`成功导入 ${successCount} 个世界书`);
        if (uploadWorldInfoState.onSuccess) {
            uploadWorldInfoState.onSuccess();
        }
        
        if (successCount === validFiles.length) {
            setTimeout(() => {
                closeUploadWorldInfoDialog();
            }, 1000);
        }
    } else {
        toast.error('导入世界书失败，请查看详细信息');
    }
};

</script>

<template>
    <div :class="[
        'absolute inset-0 z-[300] flex items-center justify-center px-4 transition-all duration-300',
        uploadWorldInfoState.show ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'
    ]">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-md" @click="!isImporting && closeUploadWorldInfoDialog()"></div>

        <!-- Modal Content -->
        <div :class="[
            'modal-content relative bg-white w-full max-w-lg rounded-3xl shadow-2xl border border-slate-100 overflow-hidden transition-all duration-300 transform flex flex-col',
            uploadWorldInfoState.show ? 'scale-100 translate-y-0' : 'scale-95 translate-y-8'
        ]">
            <!-- Header -->
            <div class="px-6 py-5 border-b border-slate-100 flex items-center justify-between">
                <h3 class="text-lg font-bold text-slate-800">添加世界书</h3>
                <button 
                    @click="!isImporting && closeUploadWorldInfoDialog()"
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
                    <label class="block text-sm font-bold text-slate-700 mb-2">世界书文件 (.json)</label>
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
                            <p class="text-slate-400 text-xs mt-1">支持选择多个 .json 格式的世界书文件</p>
                        </div>
                    </div>
                </div>

                <!-- Single File UI -->
                <div v-else-if="selectedFiles.length === 1 && !isDragging">
                    <label class="block text-sm font-bold text-slate-700 mb-2">世界书文件 (.json)</label>
                    <div 
                        @click="selectFile"
                        :class="[
                            'border-2 border-dashed rounded-2xl p-8 text-center transition-all cursor-pointer relative overflow-hidden border-slate-200 bg-slate-50 hover:bg-slate-100 hover:border-slate-300',
                            isImporting ? 'pointer-events-none opacity-60' : ''
                        ]"
                    >
                        <div class="flex flex-col items-center pointer-events-none">
                            <div class="w-12 h-12 rounded-full bg-blue-100 flex items-center justify-center text-blue-500 mb-3">
                                <FileJson class="w-6 h-6" />
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
                            <span class="text-sm font-medium">正在验证世界书...</span>
                        </div>
                        
                        <div v-else-if="selectedFiles[0].status === 'invalid'" class="flex items-start gap-3 text-red-600">
                            <AlertTriangle class="w-5 h-5 shrink-0 mt-0.5" />
                            <div>
                                <p class="text-sm font-bold">无效的世界书</p>
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
                                <p class="text-sm font-bold">有效的世界书</p>
                                <div class="mt-2 space-y-1 text-xs opacity-90">
                                    <p v-if="selectedFiles[0].worldName"><span class="font-semibold">名称：</span>{{ selectedFiles[0].worldName }}</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Multiple Files UI -->
                <div v-else-if="selectedFiles.length > 1 && !isDragging" class="space-y-4">
                    <div class="flex items-center justify-between">
                        <label class="block text-sm font-bold text-slate-700">已选择 {{ selectedFiles.length }} 个文件</label>
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
                                    <FileJson class="w-4 h-4" />
                                </div>
                                <div class="min-w-0 flex-1">
                                    <p class="text-sm font-bold text-slate-700 truncate" :title="file.worldName || file.name">
                                        {{ file.worldName || file.name }}
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
                                <div v-else-if="file.status === 'invalid'" class="flex items-center gap-1.5 text-red-500 text-xs" title="无效的文件">
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
                    @click="closeUploadWorldInfoDialog"
                    :disabled="isImporting"
                    class="px-5 py-2.5 rounded-xl font-bold text-slate-500 hover:bg-slate-200 transition-colors disabled:opacity-50 text-sm"
                >
                    取消
                </button>
                <button 
                    @click="importWorlds"
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
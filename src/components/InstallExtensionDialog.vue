<script lang="ts" setup>
import { ref, watch, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { installExtensionState, closeInstallExtensionDialog } from '../lib/useExtensionInstall';
import { openRepairGitDialog, repairGitDialogState } from '../lib/useRepairGitDialog';
import { X, UploadCloud, FileArchive, CheckCircle2, AlertTriangle, Loader2, Terminal, Copy, Trash2, GitBranchPlus, Settings } from 'lucide-vue-next';
import { toast } from 'vue-sonner';
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import * as fflate from 'fflate';

const { t } = useI18n();
const router = useRouter();

const scope = ref<'user' | 'global'>('user');
const installMode = ref<'offline' | 'online'>('online');
const gitUrl = ref('');
const gitBranch = ref('');
const gitLogs = ref<string[]>([]);
const showLogModal = ref(false);
const logContainer = ref<HTMLElement | null>(null);
const isDragging = ref(false);
const isInstalling = ref(false);
const installResult = ref<'success' | 'error' | null>(null);

// Git 安装检测
const gitInstalled = ref(true); // 默认 true，检测后更新
const gitChecking = ref(false);

const updateGitStatus = async () => {
    gitChecking.value = true;
    try {
        const gitInfo: any = await invoke('check_git');
        gitInstalled.value = gitInfo?.source !== 'none';
    } catch (e) {
        gitInstalled.value = false;
    } finally {
        gitChecking.value = false;
    }
};

let unlisten: UnlistenFn | null = null;
onMounted(async () => {
    unlisten = await listen<string>('git-install-log', (event) => {
        gitLogs.value.push(event.payload);
        nextTick(() => {
            if (logContainer.value) {
                logContainer.value.scrollTop = logContainer.value.scrollHeight;
            }
        });
    });

    // 检测 Git 是否已安装（复用 get_git_exe 逻辑，system/local 均视为可用）
    await updateGitStatus();
});

onUnmounted(() => {
    if (unlisten) unlisten();
});

interface ExtensionManifest {
    display_name?: string;
    author?: string;
    version?: string;
    description?: string;
    homePage?: string;
    auto_update?: boolean;
}

interface ExtensionFile {
    id: string;
    path?: string;
    name: string;
    status: 'pending' | 'verifying' | 'valid' | 'invalid' | 'installing' | 'success' | 'error';
    manifestInfo?: ExtensionManifest;
    errorMsg?: string;
    rawFile?: File;
}

const selectedFiles = ref<ExtensionFile[]>([]);

watch(() => installExtensionState.show, async (newVal) => {
    if (newVal) {
        resetState();
        // 每次打开弹窗时重新检测 Git，确保安装 MinGit 后状态及时更新
        await updateGitStatus();
    }
});

const resetState = () => {
    scope.value = 'user';
    installMode.value = 'online';
    gitUrl.value = '';
    gitBranch.value = '';
    gitLogs.value = [];
    showLogModal.value = false;
    selectedFiles.value = [];
    isInstalling.value = false;
    isDragging.value = false;
    installResult.value = null;
};

// ✅ 和角色卡一致的拖拽
const onDragOver = (e: DragEvent) => {
    e.preventDefault();
    isDragging.value = true;
};

const onDragLeave = (e: DragEvent) => {
    e.preventDefault();
    isDragging.value = false;
};

const onDrop = (e: DragEvent) => {
    e.preventDefault();
    isDragging.value = false;

    const files = Array.from(e.dataTransfer?.files || [])
        .filter(f => f.name.toLowerCase().endsWith('.zip'));

    if (files.length === 0) {
        toast.error(t('resources.uploadPngError').replace('.png', '.zip'));
        return;
    }

    const newFiles: ExtensionFile[] = files.map(file => ({
        id: crypto.randomUUID(),
        name: file.name,
        status: 'verifying',
        rawFile: file
    }));

    processFiles(newFiles);
};

const handleFilesSelected = (filePaths: string[]) => {
    const newFiles: ExtensionFile[] = filePaths.map(filePath => ({
        id: crypto.randomUUID(),
        path: filePath,
        name: filePath.split(/[/\\]/).pop() || 'unknown.zip',
        status: 'verifying'
    }));

    processFiles(newFiles);
};
const processFiles = async (newFiles: ExtensionFile[]) => {
    selectedFiles.value = newFiles;

    await Promise.all(selectedFiles.value.map(async (file) => {
        try {
            if (file.rawFile) {
                const buffer = await file.rawFile.arrayBuffer();
                const bytes = new Uint8Array(buffer);

                // 在前端使用 fflate 预判识别，避免通过 IPC 传输大文件导致卡死
                const manifestEntry = await new Promise<Uint8Array | null>((resolve) => {
                    fflate.unzip(bytes, {
                        filter: (info) => info.name === 'manifest.json' || info.name.endsWith('/manifest.json')
                    }, (err, unzipped) => {
                        if (err) {
                            resolve(null);
                            return;
                        }
                        const first = Object.values(unzipped)[0];
                        resolve(first || null);
                    });
                });

                if (!manifestEntry) {
                    throw new Error(t('extensions.manifestNotFound'));
                }

                const manifestText = new TextDecoder().decode(manifestEntry);
                const manifest = JSON.parse(manifestText);
                
                file.manifestInfo = manifest;
                file.status = 'valid';
            } else if (file.path) {
                const result = await invoke<ExtensionManifest>('verify_extension_zip', {
                    zipPath: file.path
                });
                file.manifestInfo = result;
                file.status = 'valid';
            }
        } catch (e) {
            file.errorMsg = String(e);
            file.status = 'invalid';
        }
    }));
};

const selectFile = async () => {
    if (isInstalling.value) return;

    try {
        const selected = await open({
            multiple: true,
            filters: [{ name: 'Zip', extensions: ['zip'] }]
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
    if (installMode.value === 'offline') {
        return selectedFiles.value.some(f => f.status === 'valid');
    } else {
        // 在线安装需要 Git，检测阶段和未安装时将按钮禁用
        if (gitChecking.value || !gitInstalled.value) return false;
        return gitUrl.value.trim().length > 0;
    }
});
const isAnyVerifying = computed(() => selectedFiles.value.some(f => f.status === 'verifying'));

const copyLogs = async () => {
    try {
        await writeText(gitLogs.value.join('\n'));
        toast.success(t('common.copySuccess'));
    } catch (e) {
        toast.error(String(e));
    }
};

const install = async () => {
    if (installMode.value === 'online') {
        if (!gitUrl.value.trim()) return;
        
        if (scope.value === 'global' && !installExtensionState.version) {
            toast.error(t('extensions.selectVersionWarning'));
            return;
        }

        isInstalling.value = true;
        gitLogs.value = [];
        showLogModal.value = true;
        try {
            await invoke('install_extension_git', {
                url: gitUrl.value.trim(),
                branchOpt: gitBranch.value.trim() || null,
                scope: scope.value,
                version: installExtensionState.version
            });
            toast.success(t('resources.importSuccess'));
            installExtensionState.onSuccess?.();
            setTimeout(() => {
                showLogModal.value = false;
                closeInstallExtensionDialog();
            }, 1500);
        } catch (e) {
            toast.error(String(e));
        } finally {
            isInstalling.value = false;
        }
        return;
    }

    const validFiles = selectedFiles.value.filter(f => f.status === 'valid');
    if (!validFiles.length || isInstalling.value || isAnyVerifying.value) return;

    if (scope.value === 'global' && !installExtensionState.version) {
        toast.error(t('extensions.selectVersionWarning'));
        return;
    }

    isInstalling.value = true;
    let successCount = 0;

    // 读取 Rust 端 config 的 auto_repair_git（通过 localStorage 同步的 camelCase key）
    const autoRepair = localStorage.getItem('autoRepairGit') === 'true';

    // 判断扩展是否满足 Git 修复条件（与 Rust 端 repair_extension_git 的 can_repair 逻辑一致）
    // 条件：auto_update === true 或 homePage 是标准 git 仓库地址
    const canRepairExtension = (manifest?: ExtensionManifest): boolean => {
        if (!manifest) return false;
        if (manifest.auto_update === true) return true;
        const hp = (manifest.homePage ?? '').toLowerCase();
        if (hp && (
            hp.includes('github.com') ||
            hp.includes('gitee.com') ||
            hp.includes('gitcode.com') ||
            hp.includes('gitlab.com') ||
            hp.endsWith('.git')
        )) {
            return true;
        }
        return false;
    };

    // 是否含有至少一个满足修复条件的扩展
    const hasThirdParty = validFiles.some(f => canRepairExtension(f.manifestInfo));

    // 如果开启自动修复，且含有满足修复条件的扩展，先打开 RepairGitDialog
    if (autoRepair && hasThirdParty) {
        // 取第一个满足修复条件的文件名称用于弹窗标题
        const firstRepairable = validFiles.find(f => canRepairExtension(f.manifestInfo));
        const firstName = firstRepairable?.manifestInfo?.display_name || firstRepairable?.name || '';
        openRepairGitDialog(firstName);
    }

    for (const file of validFiles) {
        file.status = 'installing';
        try {
            if (file.rawFile) {
                const buffer = await file.rawFile.arrayBuffer();
                const bytes = new Uint8Array(buffer);

                await invoke('install_extension_zip_from_bytes', {
                    bytes,
                    filename: file.name,
                    scope: scope.value,
                    version: installExtensionState.version
                });
            } else if (file.path) {
                await invoke('install_extension_zip', {
                    zipPath: file.path,
                    scope: scope.value,
                    version: installExtensionState.version
                });
            }

            file.status = 'success';
            successCount++;
        } catch (e) {
            file.status = 'error';
            file.errorMsg = String(e);
        }
    }

    isInstalling.value = false;

    if (successCount > 0) {
        toast.success(t('resources.installSuccessCount', { count: successCount }));
        installExtensionState.onSuccess?.();

        if (autoRepair && hasThirdParty) {
            // invoke 已串行等待修复完毕，更新 RepairGitDialog 状态
            repairGitDialogState.isRepairing = false;
            repairGitDialogState.result = successCount === validFiles.length ? 'success' : 'error';
            // 关闭安装弹窗，修复弹窗留给用户手动关闭
            closeInstallExtensionDialog();
        } else {
            setTimeout(() => {
                closeInstallExtensionDialog();
            }, 1000);
        }
    } else {
        if (autoRepair && hasThirdParty) {
            // 安装全部失败，修复弹窗标记失败
            repairGitDialogState.isRepairing = false;
            repairGitDialogState.result = 'error';
            closeInstallExtensionDialog();
        }
        toast.error(t('resources.importFailedMsg', { type: t('extensions.title') }));
    }
};
</script>

<template>
    <div 
        v-if="installExtensionState.show"
        class="absolute inset-0 z-[300] flex items-center justify-center px-4 animate-in fade-in duration-200"
    >
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-md pointer-events-auto" @click="!isInstalling && closeInstallExtensionDialog()"></div>

        <!-- Modal Content -->
        <div 
            class="modal-content relative bg-white w-full max-w-lg max-h-[620px] rounded-3xl shadow-2xl border border-slate-100 overflow-hidden flex flex-col pointer-events-auto animate-in zoom-in-95 duration-200"
            @click.stop
        >
                <!-- Header -->
                <div class="px-6 py-5 border-b border-slate-100 flex items-center justify-between">
                    <h3 class="text-lg font-bold text-slate-800">{{ t('extensions.installExtension') }}</h3>
                    <button 
                        @click="!isInstalling && closeInstallExtensionDialog()"
                        :disabled="isInstalling"
                        class="p-1.5 text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-xl transition-colors disabled:opacity-50"
                    >
                        <X class="w-5 h-5" />
                    </button>
                </div>

                <!-- Body -->
                <div class="p-6 space-y-6 flex-1 overflow-y-auto no-scrollbar">
                    <!-- Location Selector -->
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-bold text-slate-700 mb-2">{{ t('resources.installMode') }}</label>
                            <div class="flex p-1 bg-slate-100 rounded-xl">
                                <button 
                                    @click="installMode = 'online'"
                                    :class="[
                                        'flex-1 py-1.5 text-xs font-medium rounded-lg transition-all',
                                        installMode === 'online' ? 'bg-white text-slate-800 shadow-sm' : 'text-slate-500 hover:text-slate-700'
                                    ]"
                                >
                                    {{ t('resources.onlineInstall') }}
                                </button>
                                <button 
                                    @click="installMode = 'offline'"
                                    :class="[
                                        'flex-1 py-1.5 text-xs font-medium rounded-lg transition-all',
                                        installMode === 'offline' ? 'bg-white text-slate-800 shadow-sm' : 'text-slate-500 hover:text-slate-700'
                                    ]"
                                >
                                    {{ t('resources.offlineInstall') }}
                                </button>
                            </div>
                        </div>

                        <div>
                            <label class="block text-sm font-bold text-slate-700 mb-2">{{ t('resources.installLocation') }}</label>
                            <div class="flex p-1 bg-slate-100 rounded-xl">
                                <button 
                                    @click="scope = 'user'"
                                    :class="[
                                        'flex-1 py-1.5 text-xs font-medium rounded-lg transition-all',
                                        scope === 'user' ? 'bg-white text-slate-800 shadow-sm' : 'text-slate-500 hover:text-slate-700'
                                    ]"
                                >
                                    {{ t('resources.currentUserScope') }}
                                </button>
                                <button 
                                    @click="scope = 'global'"
                                    :class="[
                                        'flex-1 py-1.5 text-xs font-medium rounded-lg transition-all',
                                        scope === 'global' ? 'bg-white text-slate-800 shadow-sm' : 'text-slate-500 hover:text-slate-700'
                                    ]"
                                >
                                    {{ t('resources.globalScope') }}
                                </button>
                            </div>
                        </div>
                    </div>

                    <div class="px-3 py-2 bg-blue-50/50 border border-blue-100/50 rounded-xl">
                        <p class="text-[10px] text-blue-600/80 leading-relaxed italic">
                            {{ installMode === 'online' ? t('resources.onlineInstallDesc') : t('resources.offlineInstallDesc') }}
                        </p>
                    </div>

                    <div v-if="installMode === 'online'" class="space-y-4 pt-2">
                        <!-- Git 检测中 -->
                        <div v-if="gitChecking" class="flex flex-col items-center justify-center py-8 px-4 gap-4 bg-blue-50 border border-blue-100 rounded-2xl">
                            <div class="w-14 h-14 rounded-full bg-blue-100 flex items-center justify-center text-blue-500">
                                <Loader2 class="w-7 h-7 animate-spin" />
                            </div>
                            <div class="text-center space-y-1">
                                <p class="text-sm font-bold text-blue-900">{{ t('resources.gitChecking') }}</p>
                                <p class="text-xs text-blue-600/90 leading-relaxed max-w-[300px]">{{ t('resources.gitRequiredDesc') }}</p>
                            </div>
                        </div>

                        <!-- Git 未安装警告 -->
                        <div v-else-if="!gitInstalled" class="flex flex-col items-center justify-center py-8 px-4 gap-4 bg-amber-50 border border-amber-200 rounded-2xl">

                            <div class="w-14 h-14 rounded-full bg-amber-100 flex items-center justify-center text-amber-500">
                                <GitBranchPlus class="w-7 h-7" />
                            </div>
                            <div class="text-center space-y-1">
                                <p class="text-sm font-bold text-amber-800">{{ t('resources.gitRequiredTitle') }}</p>
                                <p class="text-xs text-amber-600/90 leading-relaxed max-w-[300px]">{{ t('resources.gitRequiredDesc') }}</p>
                            </div>
                            <button
                                @click="closeInstallExtensionDialog(); router.push('/settings')"
                                class="flex items-center gap-2 px-4 py-2 rounded-xl bg-amber-500 hover:bg-amber-600 text-white text-xs font-bold transition-all shadow-sm shadow-amber-500/20 active:scale-95"
                            >
                                <Settings class="w-3.5 h-3.5" />
                                {{ t('resources.goToSettings') }}
                            </button>
                        </div>

                        <!-- 正常在线安装表单 -->
                        <template v-else>
                        <div>
                            <label class="block text-sm font-bold text-slate-700 mb-2">{{ t('resources.gitUrl') }}</label>
                            <input 
                                v-model="gitUrl"
                                type="text"
                                :placeholder="t('resources.gitUrlPlaceholder')"
                                class="w-full px-4 py-2.5 bg-slate-50 border border-slate-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all"
                                :disabled="isInstalling"
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-bold text-slate-700 mb-2">{{ t('resources.branch') }}</label>
                            <input 
                                v-model="gitBranch"
                                type="text"
                                :placeholder="t('resources.branchPlaceholder')"
                                class="w-full px-4 py-2.5 bg-slate-50 border border-slate-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all"
                                :disabled="isInstalling"
                            />
                        </div>
                        </template>
                    </div>

                    <!-- Empty / Dragging State -->
                    <div v-if="installMode === 'offline' && selectedFiles.length === 0" @dragover="onDragOver" @dragleave="onDragLeave" @drop="onDrop">
                        <label class="block text-sm font-bold text-slate-700 mb-2">{{ t('resources.extensionPackage') }}</label>
                        <div 
                            @click="selectFile"
                            :class="[
                                'border-2 border-dashed rounded-2xl p-8 text-center transition-all cursor-pointer relative overflow-hidden',
                                isDragging ? 'border-blue-500 bg-blue-50' : 'border-slate-200 bg-slate-50 hover:bg-slate-100 hover:border-slate-300',
                                isInstalling ? 'pointer-events-none opacity-60' : ''
                            ]"
                        >
                            <div class="flex flex-col items-center pointer-events-none">
                                <div class="w-12 h-12 rounded-full bg-white shadow-sm flex items-center justify-center text-slate-400 mb-3">
                                    <UploadCloud class="w-6 h-6" />
                                </div>
                                <p class="text-slate-700 font-medium">{{ t('resources.clickOrDrag') }}</p>
                                <p class="text-slate-400 text-xs mt-1">{{ t('resources.supportMultipleExtensions') }}</p>
                            </div>
                        </div>
                    </div>

                    <!-- Single File UI -->
                    <div v-else-if="installMode === 'offline' && selectedFiles.length === 1 && !isDragging">
                        <label class="block text-sm font-bold text-slate-700 mb-2">{{ t('resources.extensionPackage') }}</label>
                        <div 
                            @click="selectFile"
                            :class="[
                                'border-2 border-dashed rounded-2xl p-8 text-center transition-all cursor-pointer relative overflow-hidden border-slate-200 bg-slate-50 hover:bg-slate-100 hover:border-slate-300',
                                isInstalling ? 'pointer-events-none opacity-60' : ''
                            ]"
                        >
                            <div class="flex flex-col items-center pointer-events-none">
                                <div class="w-12 h-12 rounded-full bg-blue-100 flex items-center justify-center text-blue-500 mb-3">
                                    <FileArchive class="w-6 h-6" />
                                </div>
                                <p class="text-slate-700 font-medium truncate max-w-[250px]">{{ selectedFiles[0].name }}</p>
                                <p class="text-blue-500 text-xs mt-1">{{ t('resources.clickToReselect') }}</p>
                            </div>
                        </div>

                        <!-- Validation Result -->
                        <div class="mt-6 rounded-xl p-4 transition-all" :class="[
                            selectedFiles[0].status === 'verifying' || selectedFiles[0].status === 'installing' ? 'bg-slate-50' : 
                            selectedFiles[0].status === 'invalid' || selectedFiles[0].status === 'error' ? 'bg-red-50' : 'bg-emerald-50'
                        ]">
                            <div v-if="selectedFiles[0].status === 'verifying'" class="flex items-center gap-3 text-slate-500">
                                <Loader2 class="w-5 h-5 animate-spin" />
                                <span class="text-sm font-medium">{{ t('resources.verifyingExtension') }}</span>
                            </div>
                            
                            <div v-else-if="selectedFiles[0].status === 'invalid'" class="flex items-start gap-3 text-red-600">
                                <AlertTriangle class="w-5 h-5 shrink-0 mt-0.5" />
                                <div>
                                    <p class="text-sm font-bold">{{ t('extensions.invalidExtension') }}</p>
                                    <p class="text-xs mt-1 opacity-80">{{ selectedFiles[0].errorMsg }}</p>
                                </div>
                            </div>

                            <div v-else-if="selectedFiles[0].status === 'error'" class="flex items-start gap-3 text-red-600">
                                <X class="w-5 h-5 shrink-0 mt-0.5" />
                                <div>
                                    <p class="text-sm font-bold">{{ t('common.failed') }}</p>
                                    <p class="text-xs mt-1 opacity-80">{{ selectedFiles[0].errorMsg }}</p>
                                </div>
                            </div>

                            <div v-else-if="selectedFiles[0].status === 'installing'" class="flex items-center gap-3 text-blue-600">
                                <Loader2 class="w-5 h-5 animate-spin" />
                                <span class="text-sm font-medium">{{ t('resources.installingExtension') }}</span>
                            </div>

                            <div v-else-if="selectedFiles[0].status === 'success'" class="flex items-center gap-3 text-emerald-600">
                                <CheckCircle2 class="w-5 h-5" />
                                <span class="text-sm font-medium">{{ t('common.success') }}</span>
                            </div>
                            
                            <div v-else-if="selectedFiles[0].status === 'valid' && selectedFiles[0].manifestInfo" class="flex items-start gap-3 text-emerald-700">
                                <CheckCircle2 class="w-5 h-5 shrink-0 mt-0.5" />
                                <div class="flex-1 min-w-0">
                                    <p class="text-sm font-bold">{{ t('resources.validExtension') }}</p>
                                    <div class="mt-2 space-y-1 text-xs opacity-90">
                                        <p v-if="selectedFiles[0].manifestInfo.display_name"><span class="font-semibold">{{ t('resources.extensionName') }}：</span>{{ selectedFiles[0].manifestInfo.display_name }}</p>
                                        <p v-if="selectedFiles[0].manifestInfo.version"><span class="font-semibold">{{ t('resources.extensionVersion') }}：</span>{{ selectedFiles[0].manifestInfo.version }}</p>
                                        <p v-if="selectedFiles[0].manifestInfo.author"><span class="font-semibold">{{ t('resources.extensionAuthor') }}：</span>{{ selectedFiles[0].manifestInfo.author }}</p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- Multiple Files UI -->
                    <div v-else-if="installMode === 'offline' && selectedFiles.length > 1 && !isDragging" class="space-y-4">
                        <div class="flex items-center justify-between">
                            <label class="block text-sm font-bold text-slate-700">{{ t('resources.selectedExtensions', { count: selectedFiles.length }) }}</label>
                            <button @click="selectFile" :disabled="isInstalling" class="text-xs text-blue-500 hover:text-blue-600 font-medium disabled:opacity-50">
                                {{ t('resources.reselect') }}
                            </button>
                        </div>
                        
                        <div class="max-h-[240px] overflow-y-auto pr-2 space-y-2 custom-scrollbar">
                            <div v-for="file in selectedFiles" :key="file.id" 
                                class="bg-slate-50 rounded-xl p-3 flex items-center justify-between border border-slate-100"
                            >
                                <div class="flex items-center gap-3 min-w-0 flex-1">
                                    <div class="w-8 h-8 rounded-lg bg-white flex items-center justify-center text-slate-400 shrink-0">
                                        <FileArchive class="w-4 h-4" />
                                    </div>
                                    <div class="min-w-0 flex-1">
                                        <p class="text-sm font-bold text-slate-700 truncate" :title="file.manifestInfo?.display_name || file.name">
                                            {{ file.manifestInfo?.display_name || file.name }}
                                        </p>
                                        <p class="text-[10px] text-slate-500 truncate" v-if="file.status === 'valid' || file.status === 'installing' || file.status === 'success'">
                                            <span v-if="file.manifestInfo?.version">v{{ file.manifestInfo.version }} </span>
                                            <span v-if="file.manifestInfo?.author">by {{ file.manifestInfo.author }}</span>
                                        </p>
                                        <p class="text-[10px] text-red-500 truncate" v-else-if="file.status === 'invalid' || file.status === 'error'" :title="file.errorMsg">
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
                                        <span>{{ t('resources.verifying') }}</span>
                                    </div>
                                    <div v-else-if="file.status === 'invalid'" class="flex items-center gap-1.5 text-red-500 text-xs" :title="t('extensions.invalidExtension')">
                                        <AlertTriangle class="w-3.5 h-3.5" />
                                        <span>{{ t('resources.invalid') }}</span>
                                    </div>
                                    <div v-else-if="file.status === 'valid'" class="flex items-center gap-1.5 text-emerald-500 text-xs">
                                        <CheckCircle2 class="w-3.5 h-3.5" />
                                        <span>{{ t('resources.pending') }}</span>
                                    </div>
                                    <div v-else-if="file.status === 'installing'" class="flex items-center gap-1.5 text-blue-500 text-xs">
                                        <Loader2 class="w-3.5 h-3.5 animate-spin" />
                                        <span>{{ t('resources.importing') }}</span>
                                    </div>
                                    <div v-else-if="file.status === 'success'" class="flex items-center gap-1.5 text-emerald-500 text-xs">
                                        <CheckCircle2 class="w-3.5 h-3.5" />
                                        <span>{{ t('resources.importSuccess') }}</span>
                                    </div>
                                    <div v-else-if="file.status === 'error'" class="flex items-center gap-1.5 text-red-500 text-xs" :title="file.errorMsg">
                                        <X class="w-3.5 h-3.5" />
                                        <span>{{ t('common.failed') }}</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Footer -->
                <div class="px-6 py-5 border-t border-slate-100 bg-slate-50 flex justify-end gap-3 shrink-0">
                    <button 
                        @click="closeInstallExtensionDialog"
                        :disabled="isInstalling"
                        class="px-5 py-2.5 rounded-xl font-bold text-slate-500 hover:bg-slate-200 transition-colors disabled:opacity-50 text-sm"
                    >
                        {{ t('common.cancel') }}
                    </button>
                    <button 
                        @click="install"
                        :disabled="!hasValidFiles || isAnyVerifying || isInstalling"
                        class="px-5 py-2.5 rounded-xl font-bold text-white transition-all shadow-md text-sm flex items-center gap-2"
                        :class="[
                            (!hasValidFiles || isAnyVerifying || isInstalling)
                                ? 'bg-blue-300 cursor-not-allowed shadow-none'
                                : 'bg-blue-500 hover:bg-blue-600 active:scale-95 shadow-blue-500/20'
                        ]"
                    >
                        <Loader2 v-if="isInstalling" class="w-4 h-4 animate-spin" />
                        <span>{{ isInstalling ? t('resources.installing') : t('resources.startInstalling') }}</span>
                    </button>
                </div>
        </div>

        <!-- Git Log Modal -->
        <div 
            v-if="showLogModal"
            class="absolute inset-0 z-[400] flex items-center justify-center px-4 animate-in fade-in duration-200"
        >
            <div class="absolute inset-0 bg-slate-900/60 backdrop-blur-sm pointer-events-auto"></div>
            <div class="relative bg-slate-900 w-full max-w-lg rounded-2xl shadow-2xl border border-slate-700 overflow-hidden flex flex-col pointer-events-auto animate-in zoom-in-95 duration-200">
                <div class="px-5 py-3 border-b border-slate-700 flex items-center justify-between bg-slate-800">
                    <div class="flex items-center gap-2">
                        <Terminal class="w-4 h-4 text-blue-400" />
                        <h4 class="text-xs font-bold text-slate-200">{{ t('resources.gitInstallLog') }}</h4>
                    </div>
                    <div class="flex items-center gap-3">
                        <button @click="copyLogs" class="text-[10px] text-slate-400 hover:text-white transition-colors flex items-center gap-1">
                            <Copy class="w-3 h-3" />
                            {{ t('resources.copyLog') }}
                        </button>
                        <button @click="gitLogs = []" class="text-[10px] text-slate-400 hover:text-white transition-colors flex items-center gap-1">
                            <Trash2 class="w-3 h-3" />
                            {{ t('resources.clearLog') }}
                        </button>
                    </div>
                </div>
                <div 
                    ref="logContainer"
                    class="p-4 h-[300px] overflow-y-auto font-mono text-[11px] leading-relaxed custom-scrollbar bg-slate-950"
                >
                    <div v-if="gitLogs.length === 0" class="flex flex-col items-center justify-center h-full text-slate-500 italic">
                        <Loader2 class="w-5 h-5 animate-spin mb-2 opacity-20" />
                        <span>{{ t('resources.waitingForGitOutput') }}</span>
                    </div>
                    <div v-else v-for="(log, index) in gitLogs" :key="index" class="text-slate-300 break-all mb-1">
                        <span class="text-slate-600 mr-2 select-none">[{{ index + 1 }}]</span>
                        <span>{{ log }}</span>
                    </div>
                </div>
                <div class="px-5 py-3 bg-slate-800 border-t border-slate-700 flex items-center justify-between gap-3">
                    <div v-if="isInstalling" class="flex items-center gap-2 text-blue-400 text-xs font-medium">
                        <Loader2 class="w-3.5 h-3.5 animate-spin" />
                        <span>{{ t('resources.gitCloning') }}</span>
                    </div>
                    <div v-else-if="installResult === 'success'" class="flex items-center gap-2 text-emerald-400 text-xs font-medium">
                        <CheckCircle2 class="w-3.5 h-3.5" />
                        <span>{{ t('resources.installAutoRepairDone') }}</span>
                    </div>
                    <div v-else class="flex-1"></div>
                    <button 
                        v-if="!isInstalling"
                        @click="showLogModal = false; closeInstallExtensionDialog()"
                        class="px-4 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-bold transition-all"
                    >
                        {{ t('common.close') }}
                    </button>
                </div>
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

.no-scrollbar::-webkit-scrollbar {
    display: none;
}
.no-scrollbar {
    -ms-overflow-style: none;
    scrollbar-width: none;
}
</style>
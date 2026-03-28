<template>
  <div class="space-y-6">
    <!-- Header Info -->
    <div
      class="grid grid-cols-2 gap-4"
    >
      <div
        class="bg-white dark:bg-slate-800 p-6 rounded-2xl shadow-sm border border-slate-100 dark:border-slate-700 flex items-center justify-between"
      >
        <div>
          <p class="text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider mb-1">{{ t('versions.currentVersion') }}</p>
          <div class="flex items-baseline space-x-2">
            <h2 class="text-2xl font-black text-slate-800 dark:text-slate-200">{{ currentVersion || t('versions.notSet') }}</h2>
            <span v-if="currentVersion" class="text-[10px] px-2 py-0.5 rounded-full font-bold uppercase tracking-wide flex-shrink-0"
                  :class="currentLocalPath ? 'bg-orange-100 dark:bg-orange-900/30 text-orange-600 dark:text-orange-400' : 'bg-emerald-100 dark:bg-emerald-900/30 text-emerald-600 dark:text-emerald-400'">
              {{ currentLocalPath ? t('settings.localSillytavern') : t('settings.onlineDownload') }}
            </span>
          </div>
          <p v-if="currentLocalPath" class="text-[10px] text-slate-400 dark:text-slate-500 mt-1 flex items-center max-w-[200px] truncate" :title="currentLocalPath">
            <Folder class="w-3 h-3 mr-1 flex-shrink-0" />
            <span class="truncate">{{ currentLocalPath }}</span>
          </p>
        </div>
        <div class="w-12 h-12 rounded-xl bg-blue-50 dark:bg-blue-900/30 text-blue-500 dark:text-blue-400 flex items-center justify-center">
          <CheckCircle2 class="w-6 h-6" />
        </div>
      </div>
      <div
        class="bg-white dark:bg-slate-800 p-6 rounded-2xl shadow-sm border border-slate-100 dark:border-slate-700 flex items-center justify-between"
      >
        <div>
          <p class="text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider mb-1">{{ t('versions.latestVersion') }}</p>
          <h2 class="text-2xl font-black text-slate-800 dark:text-slate-200">{{ latestVersion || t('common.loading') }}</h2>
        </div>
        <div class="w-12 h-12 rounded-xl bg-purple-50 dark:bg-purple-900/30 text-purple-500 dark:text-purple-400 flex items-center justify-center">
          <Sparkles class="w-6 h-6" />
        </div>
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex gap-6 border-b border-slate-200 dark:border-slate-700">
      <button 
        @click="activeTab = 'local'"
        class="pb-3 text-sm font-bold transition-colors relative"
        :class="activeTab === 'local' ? 'text-blue-600 dark:text-blue-400' : 'text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200'"
      >
        {{ t('versions.localInstalled') }}
        <div v-if="activeTab === 'local'" class="absolute bottom-[-1px] left-0 w-full h-0.5 bg-blue-600 dark:bg-blue-400 rounded-t-full"></div>
      </button>
      <button 
        id="tab-online"
        @click="activeTab = 'online'"
        class="pb-3 text-sm font-bold transition-colors relative"
        :class="activeTab === 'online' ? 'text-blue-600 dark:text-blue-400' : 'text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200'"
      >
        {{ t('versions.onlineDownload') }}
        <div v-if="activeTab === 'online'" class="absolute bottom-[-1px] left-0 w-full h-0.5 bg-blue-600 dark:bg-blue-400 rounded-t-full"></div>
      </button>
    </div>

    <!-- Tab Content: Local -->
    <div v-if="activeTab === 'local'" class="bg-white dark:bg-slate-800 rounded-2xl shadow-sm border border-slate-100 dark:border-slate-700 overflow-hidden">
      <div class="p-5 border-b border-slate-100 dark:border-slate-700 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <h3 class="font-bold text-slate-800 dark:text-slate-200 flex items-center gap-2">
            <FolderSearch class="w-5 h-5 text-slate-400 dark:text-slate-500" />
            {{ t('versions.localVersionList') }}
          </h3>
          <span v-if="scanManager.state.isScanning" class="text-[10px] text-blue-500 bg-blue-50 dark:bg-blue-900/30 px-2 py-0.5 rounded-full flex items-center gap-1">
            <Loader2 class="w-3 h-3 animate-spin" />
            {{ t('versions.scanning') }} {{ scanManager.state.scanTimeText }}
          </span>
          <span v-else-if="scanManager.state.progress.key" class="text-[10px] text-slate-400 dark:text-slate-500 bg-slate-50 dark:bg-slate-700/50 px-2 py-0.5 rounded-full border border-slate-100 dark:border-slate-600">
            {{ t(scanManager.state.progress.key, { count: scanManager.state.progress.count, found: scanManager.state.progress.found }) }}
          </span>
        </div>
        <div class="flex items-center gap-2">
          <button @click="handleSelectExistingTavern" class="text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 transition-colors p-2 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700" :title="t('versions.importLocalTooltip')">
            <FolderPlus class="w-4 h-4" />
          </button>
          <button @click="startScan(true)" :disabled="scanManager.state.isScanning" class="text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 transition-colors p-2 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700" :title="t('versions.startScan')">
            <Search class="w-4 h-4" :class="{ 'opacity-50': scanManager.state.isScanning }" />
          </button>
          <button v-if="scanManager.state.isScanning || scanManager.state.scanLogPaths.length > 0" @click="showScanLog" class="text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 transition-colors p-2 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700" :title="t('versions.scanLog')">
            <FileText class="w-4 h-4" />
          </button>
        </div>
      </div>

      <div v-if="scanManager.state.localList.length === 0 && !scanManager.state.isScanning" class="p-10 text-center text-slate-400 dark:text-slate-500 flex flex-col items-center justify-center">
        <FolderSearch class="w-12 h-12 text-slate-300 dark:text-slate-600 mb-3" />
        <p>{{ t('extensions.noExtensionsFound') }}</p>
        <button @click="startScan(true)" class="mt-4 px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg text-sm font-bold transition-colors shadow-sm">
          {{ t('versions.startScan') }}
        </button>
      </div>

      <div v-else class="divide-y divide-slate-50 dark:divide-slate-700">
        <div v-for="item in scanManager.state.localList" :key="item.path" class="p-5 hover:bg-slate-50/50 dark:hover:bg-slate-800/50 transition-colors group">
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
            <div class="flex items-center gap-3 min-w-0">
              <div class="w-10 h-10 rounded-xl bg-indigo-50 dark:bg-indigo-900/30 flex items-center justify-center text-indigo-500">
                <Box class="w-5 h-5" />
              </div>
              <div>
                <div class="flex items-center gap-2">
                  <span class="font-bold text-slate-800 dark:text-slate-200">{{ item.version === 'unknown' ? t('versions.unknownVersion') : 'v' + item.version }}</span>
                  <span v-if="currentVersion === item.version && currentLocalPath === item.path" class="px-2 py-0.5 rounded-full bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 text-[10px] font-bold uppercase tracking-wide">{{ t('versions.current') }}</span>
                </div>
                <div class="text-xs text-slate-400 dark:text-slate-500 mt-1 flex items-center gap-1 select-all">
                  <MapPin class="w-3 h-3" />
                  {{ item.path }}
                </div>
              </div>
            </div>

            <div class="flex flex-wrap sm:flex-nowrap items-center gap-2 mt-2 sm:mt-0 shrink-0">
              <button 
                v-if="!item.hasNodeModules"
                @click="handleInstallLocalDependencies(item)"
                class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 justify-center bg-amber-500 text-white hover:bg-amber-600 shadow-sm w-full sm:w-auto break-words text-center"
              >
                <Download class="w-4 h-4 shrink-0" />
                <span>{{ t('versions.installDeps') }}</span>
              </button>
              
              <button 
                v-else
                @click="handleSwitchLocal(item)"
                :disabled="currentVersion === item.version && currentLocalPath === item.path || switchingVersion !== null"
                class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 justify-center text-center w-full sm:w-auto break-words"
                :class="(currentVersion === item.version && currentLocalPath === item.path || switchingVersion !== null)
                  ? 'bg-slate-100 dark:bg-slate-700 text-slate-400 dark:text-slate-500 cursor-not-allowed' 
                  : 'bg-white dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-600 hover:text-slate-900 dark:hover:text-slate-100 shadow-sm'"
              >
                <Power class="w-4 h-4" :class="{ 'animate-spin': switchingVersion === item.path }" />
                {{ (currentVersion === item.version && currentLocalPath === item.path) ? t('versions.currentlyUsed') : (switchingVersion === item.path ? t('versions.switching') : t('versions.switchVersion')) }}
              </button>

              <button 
                v-if="!(currentVersion === item.version && currentLocalPath === item.path)"
                @click="handleRemoveLocal(item.path)"
                class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 justify-center text-center w-full sm:w-auto break-words bg-white dark:bg-slate-700 border border-red-100 dark:border-red-900/30 text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 hover:text-red-600 dark:hover:text-red-300 shadow-sm"
              >
                <Trash2 class="w-4 h-4" />
                {{ t('versions.removeFromLocal') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Tab Content: Online -->
    <div v-if="activeTab === 'online'" class="bg-white dark:bg-slate-800 rounded-2xl shadow-sm border border-slate-100 dark:border-slate-700 overflow-hidden">
      <div class="p-5 border-b border-slate-100 dark:border-slate-700 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <h3 class="font-bold text-slate-800 dark:text-slate-200 flex items-center gap-2">
            <History class="w-5 h-5 text-slate-400 dark:text-slate-500" />
            {{ t('versions.versionList') }}
          </h3>
          <span v-if="lastFetchTimeDisplay" class="text-[10px] text-slate-400 dark:text-slate-500 bg-slate-50 dark:bg-slate-700/50 px-2 py-0.5 rounded-full border border-slate-100 dark:border-slate-600 flex items-center gap-1">
            <Clock class="w-3 h-3" />
            {{ t('versions.lastSync') }}: {{ lastFetchTimeDisplay }}
          </span>
        </div>
        <div class="flex items-center gap-2">
          <button @click="refresh(true)" class="text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 transition-colors p-2 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700" :title="t('versions.forceRefresh')">
            <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': loading }" />
          </button>
        </div>
      </div>
      
      <div v-if="loading && releases.length === 0" class="p-10 text-center text-slate-400 dark:text-slate-500">
        <Loader2 class="w-8 h-8 animate-spin mx-auto mb-2" />
        <p>{{ t('versions.fetchingVersions') }}</p>
      </div>

      <div v-else class="divide-y divide-slate-50 dark:divide-slate-700">
        <div v-for="release in releases" :key="release.id" class="p-5 hover:bg-slate-50/50 dark:hover:bg-slate-800/50 transition-colors group">
          <div class="flex flex-col sm:flex-row sm:items-start justify-between gap-4">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <h4 class="font-bold text-slate-800 dark:text-slate-200">{{ release.name || release.tag_name }}</h4>
                <span v-if="release.tag_name === latestVersion" class="px-2 py-0.5 rounded-full bg-purple-100 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400 text-[10px] font-bold uppercase tracking-wide">Latest</span>
                <span v-if="isInstalledOnline(release.tag_name)" class="px-2 py-0.5 rounded-full bg-emerald-100 dark:bg-emerald-900/30 text-emerald-600 dark:text-emerald-400 text-[10px] font-bold uppercase tracking-wide">Installed</span>
                <span v-if="currentVersion === release.tag_name && currentLocalPath === ''" class="px-2 py-0.5 rounded-full bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 text-[10px] font-bold uppercase tracking-wide">Current</span>
              </div>
              
              <div class="flex items-center gap-4 text-xs text-slate-400 dark:text-slate-500 mb-3">
                <span class="flex items-center gap-1">
                   <Calendar class="w-3 h-3" />
                   {{ t('versions.releasedOn') }} {{ formatDate(release.published_at) }}
                </span>
                <span class="flex items-center gap-1">
                   <Clock class="w-3 h-3" />
                   {{ t('versions.createdOn') }} {{ formatDate(release.created_at) }}
                </span>
              </div>

              <!-- Body with hover expansion -->
              <div class="text-sm text-slate-600 dark:text-slate-400 relative group/body cursor-default">
                 <div class="line-clamp-2 whitespace-pre-wrap group-hover/body:line-clamp-none transition-all duration-300">
                    {{ release.body }}
                 </div>
                 <div class="absolute -bottom-4 left-0 w-full h-4 bg-gradient-to-t from-white dark:from-slate-800 to-transparent group-hover/body:hidden"></div>
              </div>
            </div>

            <div class="flex flex-wrap sm:flex-nowrap items-center gap-2 mt-2 sm:mt-0 shrink-0 pt-1">
               <button 
                 v-if="isInstalledOnline(release.tag_name) && !hasDependenciesOnline(release.tag_name)"
                 @click="handleInstallDependencies(release.tag_name)"
                 class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 justify-center bg-amber-500 text-white hover:bg-amber-600 shadow-sm w-full sm:w-auto break-words text-center"
               >
                 <Download class="w-4 h-4 shrink-0" />
                 <span>{{ t('versions.installDeps') }}</span>
               </button>

               <button 
                 v-else-if="isInstalledOnline(release.tag_name)"
                 :id="'btn-switch-' + release.tag_name"
                 @click="handleSwitchOnline(release.tag_name)"
                 :disabled="(currentVersion === release.tag_name && currentLocalPath === '') || switchingVersion !== null || deletingVersions.size > 0"
                 class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 justify-center w-full sm:w-auto break-words text-center"
                 :class="((currentVersion === release.tag_name && currentLocalPath === '') || switchingVersion !== null || deletingVersions.size > 0)
                    ? 'bg-slate-100 dark:bg-slate-700 text-slate-400 dark:text-slate-500 cursor-not-allowed' 
                    : 'bg-white dark:bg-slate-700 border border-slate-200 dark:border-slate-600 text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-600 hover:text-slate-900 dark:hover:text-slate-100 shadow-sm'"
               >
                 <Power class="w-4 h-4 shrink-0" :class="{ 'animate-spin': switchingVersion === release.tag_name }" />
                 <span>{{ (currentVersion === release.tag_name && currentLocalPath === '') ? t('versions.currentlyUsed') : (switchingVersion === release.tag_name ? t('versions.switching') : t('versions.switchVersion')) }}</span>
               </button>

               <button 
                 v-if="isInstalledOnline(release.tag_name) && !(currentVersion === release.tag_name && currentLocalPath === '')"
                 @click="handleDeleteOnline(release.tag_name)"
                 :disabled="deletingVersions.has(release.tag_name) || switchingVersion !== null"
                 class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 justify-center w-full sm:w-auto break-words text-center bg-white dark:bg-slate-700 border shadow-sm"
                 :class="(deletingVersions.has(release.tag_name) || switchingVersion !== null)
                    ? 'border-slate-100 dark:border-slate-600 text-slate-400 dark:text-slate-500 cursor-not-allowed'
                    : 'border-red-100 dark:border-red-900/30 text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 hover:text-red-600 dark:hover:text-red-300'"
               >
                 <Loader2 v-if="deletingVersions.has(release.tag_name)" class="w-4 h-4 animate-spin shrink-0" />
                 <Trash2 v-else class="w-4 h-4 shrink-0" />
                 <span>{{ t('versions.deleteVersion') }}</span>
                 <span v-if="deletingVersions.has(release.tag_name)">
                   {{ t('versions.deleting') }}
                 </span>
               </button>

               <button 
                 v-else-if="!isInstalledOnline(release.tag_name)"
                 :id="'btn-install-' + release.tag_name"
                 @click="handleInstall(release)"
                 class="px-4 py-2 rounded-xl text-sm font-bold transition-all flex items-center gap-2 justify-center w-full sm:w-auto break-words text-center bg-slate-900 dark:bg-slate-700 text-white hover:bg-slate-800 dark:hover:bg-slate-600 shadow-lg dark:shadow-slate-900/50 active:scale-95"
               >
                 <Download class="w-4 h-4 shrink-0" />
                 <span>{{ t('versions.downloadInstall') }}</span>
               </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Scan Log Dialog -->
    <Teleport to="body">
      <div v-if="scanLogVisible" class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/40" @click.self="scanLogVisible = false">
        <div class="bg-white dark:bg-slate-800 rounded-2xl shadow-2xl border border-slate-200 dark:border-slate-700 w-full max-w-3xl max-h-[80vh] mx-4 flex flex-col animate-in fade-in zoom-in-95 duration-200">
            <div class="flex items-center justify-between px-6 py-4 border-b border-slate-100 dark:border-slate-700 flex-shrink-0">
            <h3 class="text-base font-bold text-slate-800 dark:text-slate-200">{{ t('versions.scanLog') }}</h3>
            <div class="flex items-center gap-3">
              <span v-if="scanManager.state.isScanning" class="text-xs text-blue-500 flex items-center gap-1">
                <Loader2 class="w-3 h-3 animate-spin" />
                {{ scanManager.state.scanLogPaths.length }} {{ t('common.items') }}
              </span>
              <button @click="scrollToBottom" class="text-slate-400 hover:text-blue-500 dark:text-slate-500 dark:hover:text-blue-400 transition-colors p-1 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700" title="Scroll to bottom">
                <ArrowDown class="w-4 h-4" />
              </button>
              <button @click="scanLogVisible = false" class="text-slate-400 hover:text-slate-600 dark:text-slate-500 dark:hover:text-slate-300 transition-colors p-1 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-700">
                <X class="w-4 h-4" />
              </button>
            </div>
          </div>
          <div class="flex-1 overflow-auto px-6 py-4 min-h-0">
            <div v-if="scanLogRefreshing && scanManager.state.scanLogPaths.length === 0" class="flex items-center justify-center py-10 text-slate-400 dark:text-slate-500">
              <Loader2 class="w-5 h-5 animate-spin mr-2" />
              {{ t('versions.scanLogLoading') }}
            </div>
            <div v-else-if="scanManager.state.scanLogPaths.length === 0" class="flex items-center justify-center py-10 text-slate-400 dark:text-slate-500">
              <FileText class="w-8 h-8 mr-2 opacity-40" />
              {{ t('versions.scanLogEmpty') }}
            </div>
            <div v-else ref="scanLogContainer" class="text-xs font-mono text-slate-600 dark:text-slate-400 bg-slate-50 dark:bg-slate-900 rounded-xl p-4 overflow-auto max-h-[60vh]">
              <div v-for="(p, i) in scanManager.state.scanLogPaths" :key="i" class="leading-relaxed break-all whitespace-nowrap overflow-hidden text-ellipsis" :title="p">
                <span class="text-slate-400 dark:text-slate-600 select-none mr-2">{{ String(i + 1).padStart(4, ' ') }}</span>
                <span class="text-slate-500 mr-1">{{ t('versions.scanningDir') }}</span>{{ p }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useI18n } from 'vue-i18n';
import { 
    CheckCircle2, Sparkles, History, RefreshCw, Loader2, 
    Calendar, Clock, Download, Power, Trash2, FolderPlus, FolderSearch, Search, Box, MapPin, Folder, FileText, X, ArrowDown
} from 'lucide-vue-next';
import { scanManager, LocalTavernItem } from '../lib/useScan';
import { installState, resetInstallState } from '../lib/useInstall';
import { Dialog } from '../lib/useDialog';
import { toast } from 'vue-sonner';
import { useReleases } from '../lib/useReleases';
import { updateOneClickMessage, startOneClickSetup, simulateClickEffect } from '../lib/useOneClick';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

const { releases, loading, latestVersion, lastFetchTime, fetchReleases } = useReleases();

const isOneClickSetup = computed(() => route.query.action === 'one_click_setup_st');

interface ReleaseAsset {
    name: string;
    browser_download_url: string;
}

interface Release {
    id: number;
    tag_name: string;
    name: string;
    body: string;
    created_at: string;
    published_at: string;
    zipball_url: string;
    assets: ReleaseAsset[];
}

interface InstalledVersionInfo {
    version: string;
    hasNodeModules: boolean;
    isLink?: boolean;
}

const activeTab = ref<'local' | 'online'>('local');
const installedVersions = ref<InstalledVersionInfo[]>([]);
const currentVersion = ref('');
const currentLocalPath = ref('');

// Synchronous cache loading for current version to prevent blink
try {
    const cachedConfig = localStorage.getItem('app_settings_config_cache');
    if (cachedConfig) {
        const config = JSON.parse(cachedConfig);
        if (config.sillytavern && config.sillytavern.version) {
            const versionObj = config.sillytavern.version;
            currentVersion.value = versionObj.version || '';
            currentLocalPath.value = versionObj.path || '';
        }
    }
} catch (e) {
    console.error('Failed to parse config cache:', e);
}

const switchingVersion = ref<string | null>(null);
const deletingVersions = ref<Set<string>>(new Set());
const lastFetchTimeDisplay = computed(() => {
    if (!lastFetchTime.value) return '';
    return formatDate(new Date(lastFetchTime.value).toISOString());
});

const loadAppConfig = async () => {
    try {
        const config: any = await invoke('get_app_config');

        // Get current version from config
        if (config.sillytavern && config.sillytavern.version) {
            const versionObj = config.sillytavern.version;
            currentVersion.value = versionObj.version || '';
            currentLocalPath.value = versionObj.path || '';
        }
    } catch (e) {
        console.error('Failed to load app config:', e);
    }
};

// 扫描日志
const scanLogVisible = ref(false);
const scanLogRefreshing = ref(false);
const scanLogContainer = ref<HTMLElement | null>(null);

const showScanLog = () => {
    scanLogVisible.value = true;
    scanLogRefreshing.value = true;
    // 下一帧滚动到底部
    setTimeout(() => {
        scanLogRefreshing.value = false;
        scrollToBottom();
    }, 50);
};

const scrollToBottom = () => {
    if (scanLogContainer.value) {
        scanLogContainer.value.scrollTop = scanLogContainer.value.scrollHeight;
    }
};

// 扫描过程中自动滚动到底部
watch(() => scanManager.state.scanLogPaths.length, () => {
    if (scanLogVisible.value) {
        scrollToBottom();
    }
});

const startScan = async (manual = false) => {
    if (manual) {
        Dialog.info({
            title: t('versions.startScan'),
            msg: t('versions.scanTimeWarning'),
            confirmText: t('common.confirm'),
            cancelText: t('common.cancel'),
            onConfirm: async () => {
                try {
                    await scanManager.startScan(true);
                } catch (e) {
                    toast.error('Failed to start scan: ' + String(e));
                }
            }
        });
    } else {
        try {
            await scanManager.startScan(false);
        } catch (e) {
            toast.error('Failed to start scan: ' + String(e));
        }
    }
};

const handleSelectExistingTavern = async () => {
    try {
        const selected = await open({
            multiple: false,
            directory: false,
            filters: [{ name: 'JSON', extensions: ['json'] }],
            title: t('versions.importLocalTooltip')
        });
        
        if (!selected) return;
        
        const packageJsonPath = Array.isArray(selected) ? selected[0] : selected;
        
        if (!packageJsonPath.toLowerCase().endsWith('package.json')) {
            toast.error(t('versions.notTavernDir'));
            return;
        }

        // Parse version
        const dirPath = packageJsonPath.substring(0, packageJsonPath.lastIndexOf(packageJsonPath.includes('\\') ? '\\' : '/'));
        
        // Prevent duplicates
        if (scanManager.state.localList.some(item => item.path === dirPath)) {
            toast.info(t('versions.alreadyInLocal'));
            return;
        }

        try {
            const fileBytes = await invoke<number[]>('read_local_file', { path: packageJsonPath });
            const fileContent = new TextDecoder().decode(new Uint8Array(fileBytes));
            const packageData = JSON.parse(fileContent);
            if (packageData.name?.toLowerCase() !== 'sillytavern') {
                toast.error(t('versions.notTavernDir'));
                return;
            }
            
            const item: LocalTavernItem = {
                path: dirPath,
                version: packageData.version || 'unknown',
                hasNodeModules: await invoke('check_local_tavern_dependencies', { path: dirPath })
            };
            
            scanManager.state.localList.push(item);
            await scanManager.saveLocalList();
            toast.success(t('versions.addedToLocal'));
        } catch(e) {
            toast.error('Failed to read package.json: ' + String(e));
        }

    } catch (e) {
        toast.error(String(e));
    }
};

const handleRemoveLocal = async (path: string) => {
    scanManager.state.localList = scanManager.state.localList.filter(item => item.path !== path);
    await scanManager.saveLocalList();
};

const handleSwitchLocal = async (item: LocalTavernItem) => {
    if (switchingVersion.value) return;
    switchingVersion.value = item.path;
    try {
        const payload = { version: item.version, path: item.path };
        await invoke('switch_sillytavern_version', { version: payload });
        currentVersion.value = item.version;
        currentLocalPath.value = item.path;
        
        // Update cache
        try {
            const freshConfig: any = await invoke('get_app_config');
            localStorage.setItem('app_settings_config_cache', JSON.stringify(freshConfig));
        } catch (e) {
            console.error('更新缓存失败:', e);
        }
        
        toast.success(t('versions.switchedTo', { version: item.version === 'unknown' ? t('versions.unknownVersion') : 'Local v' + item.version }));
    } catch (e) {
        toast.error(t('versions.switchFailed') + ': ' + String(e));
    } finally {
        switchingVersion.value = null;
    }
};

const isInstalledOnline = (tagName: string) => installedVersions.value.some(v => v.version === tagName);
const hasDependenciesOnline = (tagName: string) => {
    const v = installedVersions.value.find(v => v.version === tagName);
    return v ? v.hasNodeModules : false;
};

const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString('zh-CN', {
        year: 'numeric', month: '2-digit', day: '2-digit',
        hour: '2-digit', minute: '2-digit'
    });
};

const refresh = async (forceUpdate = false) => {
    try {
        const [installed] = await Promise.all([invoke('get_installed_versions_info')]);
        installedVersions.value = installed as InstalledVersionInfo[];
        
        await loadAppConfig();

        // 使用统一的获取逻辑
        await fetchReleases(forceUpdate);
        
    } catch (e) {
        if (!isOneClickSetup.value) {
            toast.error(t('versions.fetchFailed') + ': ' + String(e));
        } else {
            updateOneClickMessage(t('versions.fetchFailed') + ': ' + String(e));
        }
    } finally {
        if (isOneClickSetup.value && releases.value.length > 0 && installState.status !== 'downloading') {
            const releaseToInstall = releases.value[0];
            updateOneClickMessage(t('oneClick.preparingDownload', { version: releaseToInstall.tag_name }));
            setTimeout(() => {
                simulateClickEffect('btn-install-' + releaseToInstall.tag_name);
                handleInstall(releaseToInstall);
            }, 1000);
        }
    }
};

const handleSwitchOnline = async (version: string) => {
    if (switchingVersion.value || deletingVersions.value.size > 0) return;
    switchingVersion.value = version;
    try {
        const payload = { version: version, path: '' };
        await invoke('switch_sillytavern_version', { version: payload });
        
        // Update cache from backend
        try {
            const freshConfig: any = await invoke('get_app_config');
            localStorage.setItem('app_settings_config_cache', JSON.stringify(freshConfig));
            if (freshConfig?.sillytavern?.version) {
                currentVersion.value = freshConfig.sillytavern.version.version;
                currentLocalPath.value = freshConfig.sillytavern.version.path;
            } else {
                currentVersion.value = version;
                currentLocalPath.value = '';
            }
        } catch (e) {
            console.error('获取配置更新缓存失败:', e);
            currentVersion.value = version;
            currentLocalPath.value = '';
        }
        
        if (!isOneClickSetup.value) {
            toast.success(t('versions.switchedTo', { version }));
        } else {
            updateOneClickMessage(t('versions.switchedTo', { version }));
        }
    } catch (e) {
        if (!isOneClickSetup.value) {
            toast.error(t('versions.switchFailed') + ': ' + String(e));
        } else {
            updateOneClickMessage(t('versions.switchFailed') + ': ' + String(e));
        }
    } finally {
        switchingVersion.value = null;
    }
};

const handleDeleteOnline = (version: string) => {
    Dialog.warning({
        title: t('versions.confirmDelete'),
        msg: t('versions.confirmDeleteMsg', { version }),
        confirmText: t('common.delete'),
        cancelText: t('common.cancel'),
        onConfirm: async () => {
            installState.show = true;
            installState.version = version;
            installState.status = 'deleting';
            installState.operation = 'delete';
            installState.progress = 0;
            installState.logs = [t('versions.startDeleting', { version })];

            deletingVersions.value.add(version);
            try {
                await invoke('delete_sillytavern_version', { version });
                installState.logs.push(t('versions.deleteSuccess', { version }));
                installState.status = 'done';
                installState.progress = 1;
                await refresh();
            } catch (e) {
                installState.status = 'error';
                installState.logs.push(t('versions.deleteFailed') + ': ' + String(e));
            } finally {
                deletingVersions.value.delete(version);
                Dialog.close();
            }
        }
    });
};

const handleInstallLocalDependencies = (item: LocalTavernItem) => {
    Dialog.info({
        title: t('versions.installDeps'),
        msg: t('versions.installDepsLocalWarning'),
        confirmText: t('common.confirm'),
        cancelText: t('common.cancel'),
        onConfirm: async () => {
            installState.show = true;
            installState.version = item.path; // use path as version for backend
            installState.status = 'downloading';
            installState.operation = 'install';
            installState.progress = 0;
            installState.logs = [t('versions.startInstallingDeps', { version: item.path })];

            try {
                await invoke('install_sillytavern_dependencies', { version: item.path });
            } catch (e) {
                installState.status = 'error';
                installState.logs.push(`${t('common.error')}: ${String(e)}`);
            }
        }
    });
};

const handleInstallDependencies = async (version: string) => {
    installState.show = true;
    installState.version = version;
    installState.status = 'downloading';
    installState.operation = 'install';
    installState.progress = 0;
    installState.logs = [t('versions.startInstallingDeps', { version })];

    try {
        await invoke('install_sillytavern_dependencies', { version });
    } catch (e) {
        installState.status = 'error';
        installState.logs.push(`${t('common.error')}: ${String(e)}`);
    }
};

const handleInstall = async (release: Release) => {
    installState.show = true;
    installState.version = release.tag_name;
    installState.status = 'downloading';
    installState.operation = 'install';
    installState.progress = 0;
    installState.logs = [t('versions.startInstalling', { version: release.tag_name })];

    let downloadUrl = `https://github.com/SillyTavern/SillyTavern/archive/refs/tags/${release.tag_name}.zip`;
    try {
        const config: any = await invoke('get_app_config');
        if (config.githubProxy && config.githubProxy.enable && config.githubProxy.url) {
            let proxyUrl = config.githubProxy.url;
            if (!proxyUrl.endsWith('/')) proxyUrl += '/';
            downloadUrl = `${proxyUrl}${downloadUrl}`;
            installState.logs.push(t('versions.usingProxy') + `: ${proxyUrl}`);
        }
    } catch (e) { }

    try {
        await invoke('install_sillytavern_version', { version: release.tag_name, url: downloadUrl });
        // Update checkpoint after successful install/extraction
        const freshConfig: any = await invoke('get_app_config');
        freshConfig.setupCheckpoint = `ST_INSTALLED:${release.tag_name}`;
        await invoke('save_app_config', { config: freshConfig });
    } catch (e) {
        installState.status = 'error';
        installState.logs.push(`${t('common.error')}: ${String(e)}`);
    }
};

onMounted(async () => {
    if (isOneClickSetup.value) {
        setTimeout(async () => {
            startOneClickSetup(t('oneClick.fetchingVersions'));
            simulateClickEffect('tab-online');
            activeTab.value = 'online';
            await scanManager.init();
            await scanManager.loadConfig();
            await loadAppConfig();
            refresh();
            startScan(); // auto scan if empty
        }, 1000);
    } else {
        await scanManager.init();
        await scanManager.loadConfig();
        await loadAppConfig();
        refresh();
        startScan(); // auto scan if empty
    }
});

watch(() => installState.status, (newStatus) => {
    if (isOneClickSetup.value) {
        if (newStatus === 'downloading') {
            updateOneClickMessage(t('oneClick.downloadingTavern'));
        } else if (newStatus === 'extracting') {
            updateOneClickMessage(t('oneClick.extractingTavern'));
        } else if (newStatus === 'installing') {
            updateOneClickMessage(t('oneClick.installingDeps'));
        } else if (newStatus === 'error') {
            updateOneClickMessage(t('oneClick.error'));
        }
    }

    if (newStatus === 'done') {
        // Find if this was a local installation and update its status
        const localItem = scanManager.state.localList.find(item => item.path === installState.version);
        if (localItem) {
            localItem.hasNodeModules = true;
            scanManager.saveLocalList();
        }

        invoke('get_installed_versions_info').then(installed => {
            installedVersions.value = installed as InstalledVersionInfo[];
        }).catch(e => {
            console.error(e);
        });
        
        if (isOneClickSetup.value) {
            const versionToSwitch = installState.version;
            updateOneClickMessage(t('oneClick.downloadSuccess'));
            
            invoke('get_app_config').then(async (config: any) => {
                config.initialSetupCompleted = true;
                config.setupCheckpoint = 'DONE';
                await invoke('save_app_config', { config });
                localStorage.setItem('app_settings_config_cache', JSON.stringify(config));
                
                setTimeout(() => {
                    simulateClickEffect('btn-dialog-close');
                    setTimeout(() => resetInstallState(), 200); // 稍微延迟以显示点击效果
                    
                    setTimeout(async () => {
                        if (versionToSwitch) {
                            simulateClickEffect('btn-switch-' + versionToSwitch);
                            await handleSwitchOnline(versionToSwitch);
                        }
                        
                        setTimeout(() => {
                            router.push('/console?action=one_click_start');
                        }, 3000);
                    }, 3000);
                }, 3000);
            }).catch(e => {
                console.error("Failed to complete one click setup:", e);
                updateOneClickMessage(String(e));
            });
        } else {
            // 这里修改：如果是手动安装依赖等操作，也在完成后等待3秒才关闭弹窗
            if (installState.operation === 'install' || installState.operation === 'unbind') {
                setTimeout(() => {
                    if (installState.status === 'done') { // 确保当前依然是完成状态
                        resetInstallState();
                    }
                }, 3000);
            }
        }
    }
});
</script>

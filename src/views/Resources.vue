<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { PhUserSquare, PhArrowsClockwise, PhTrash, PhPlus, PhGlobe } from '@phosphor-icons/vue'
import { ChevronLeft, ChevronRight, CheckSquare, Square, BookOpen } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { openCharacterCardDialog } from '../lib/useCharacterCardDialog'
import { openUploadCharacterCardDialog } from '../lib/useUploadCharacterCard'
import { openWorldInfoDialog } from '../lib/useWorldInfoDialog'
import { openUploadWorldInfoDialog } from '../lib/useUploadWorldInfo'
import { Dialog } from '../lib/useDialog'

const { t } = useI18n()
const activeTab = ref<'characters' | 'worlds'>('characters')

interface CharacterCardFile {
  fileName: string
  size: number
  modifiedMs: number | null
}

interface WorldInfoFile {
  fileName: string
  size: number
  modifiedMs: number | null
}

const loading = ref(false)
const errorMsg = ref('')
const characterCards = ref<CharacterCardFile[]>([])
const worldInfos = ref<WorldInfoFile[]>([])
const thumbUrlByFileName = ref<Record<string, string>>({})
const thumbLoadingByFileName = ref<Record<string, boolean>>({})

// 批量删除/多选相关
const isSelectMode = ref(false)
const selectedFiles = ref<Set<string>>(new Set())

const toggleSelectMode = () => {
  isSelectMode.value = !isSelectMode.value
  if (!isSelectMode.value) {
    selectedFiles.value.clear()
  }
}

const toggleSelectFile = (fileName: string, event: Event) => {
  event.stopPropagation()
  const newSet = new Set(selectedFiles.value)
  if (newSet.has(fileName)) {
    newSet.delete(fileName)
  } else {
    newSet.add(fileName)
  }
  selectedFiles.value = newSet
}

const selectAllOnPage = () => {
  const newSet = new Set(selectedFiles.value)
  let allSelected = true
  const currentItems = activeTab.value === 'characters' ? paginatedCards.value : paginatedWorlds.value
  
  for (const item of currentItems) {
    if (!newSet.has(item.fileName)) {
      allSelected = false
      break
    }
  }

  if (allSelected) {
    for (const item of currentItems) {
      newSet.delete(item.fileName)
    }
  } else {
    for (const item of currentItems) {
      newSet.add(item.fileName)
    }
  }
  selectedFiles.value = newSet
}

const handleItemClick = (fileName: string, event: Event) => {
  if (isSelectMode.value) {
    toggleSelectFile(fileName, event)
  } else {
    if (activeTab.value === 'characters') {
      openCharacterCardDialog(fileName)
    } else {
      openWorldInfoDialog(fileName)
    }
  }
}

const deleteSelected = async () => {
  if (selectedFiles.value.size === 0) return
  const isChar = activeTab.value === 'characters'
  const itemName = isChar ? t('resources.characterCard') : t('resources.worldInfo')
  
  Dialog.warning({
    title: t('resources.confirmDelete'),
    msg: t('resources.confirmDeleteMultiple', { count: selectedFiles.value.size, type: itemName }),
    confirmText: t('common.delete'),
    cancelText: t('common.cancel'),
    onConfirm: async () => {
      loading.value = true
      try {
        if (isChar) {
          await invoke('delete_character_cards', { fileNames: Array.from(selectedFiles.value) })
        } else {
          await invoke('delete_world_infos', { fileNames: Array.from(selectedFiles.value) })
        }
        selectedFiles.value.clear()
        isSelectMode.value = false
        if (isChar) {
          await loadCharacterCards()
        } else {
          await loadWorldInfos()
        }
      } catch (e: any) {
        errorMsg.value = t('resources.deleteFailed') + ': ' + (e?.message || String(e))
        loading.value = false
      }
    }
  })
}

const deleteSingle = async (fileName: string, event: Event) => {
  event.stopPropagation()
  const isChar = activeTab.value === 'characters'
  const itemName = isChar ? t('resources.characterCard') : t('resources.worldInfo')
  
  Dialog.warning({
    title: t('resources.confirmDelete'),
    msg: t('resources.confirmDeleteSingle', { type: itemName, name: fileName }),
    confirmText: t('common.delete'),
    cancelText: t('common.cancel'),
    onConfirm: async () => {
      loading.value = true
      try {
        if (isChar) {
          await invoke('delete_character_cards', { fileNames: [fileName] })
          await loadCharacterCards()
        } else {
          await invoke('delete_world_infos', { fileNames: [fileName] })
          await loadWorldInfos()
        }
      } catch (e: any) {
        errorMsg.value = t('resources.deleteFailed') + ': ' + (e?.message || String(e))
        loading.value = false
      }
    }
  })
}

const importCard = () => {
  openUploadCharacterCardDialog(() => {
    loadCharacterCards()
  })
}

const importWorld = () => {
  openUploadWorldInfoDialog(() => {
    loadWorldInfos()
  })
}

// 监听弹窗导入成功事件（可以利用自定义事件或直接通过重新加载数据来实现）
window.addEventListener('character-card-imported', () => {
  loadCharacterCards()
})

const currentPage = ref(1)
const pageSize = computed(() => activeTab.value === 'characters' ? 10 : 20)

const totalCount = computed(() => activeTab.value === 'characters' ? characterCards.value.length : worldInfos.value.length)
const totalPages = computed(() => Math.max(1, Math.ceil(totalCount.value / pageSize.value)))

const paginatedCards = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return characterCards.value.slice(start, start + pageSize.value)
})

const paginatedWorlds = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return worldInfos.value.slice(start, start + pageSize.value)
})

const prevPage = () => {
  if (currentPage.value > 1) {
    currentPage.value--
  }
}

const nextPage = () => {
  if (currentPage.value < totalPages.value) {
    currentPage.value++
  }
}

const revokeAllThumbs = () => {
  for (const url of Object.values(thumbUrlByFileName.value)) {
    URL.revokeObjectURL(url)
  }
  thumbUrlByFileName.value = {}
  thumbLoadingByFileName.value = {}
}

const formatSize = (bytes: number) => {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  let v = bytes
  let idx = 0
  while (v >= 1024 && idx < units.length - 1) {
    v /= 1024
    idx++
  }
  return `${v.toFixed(idx === 0 ? 0 : 1)} ${units[idx]}`
}

const loadThumbnail = async (fileName: string) => {
  if (thumbUrlByFileName.value[fileName]) return
  if (thumbLoadingByFileName.value[fileName]) return
  thumbLoadingByFileName.value = { ...thumbLoadingByFileName.value, [fileName]: true }
  try {
    const bytes = await invoke<number[]>('read_character_card_png', { fileName })
    const u8 = new Uint8Array(bytes)
    const url = URL.createObjectURL(new Blob([u8], { type: 'image/png' }))
    thumbUrlByFileName.value = { ...thumbUrlByFileName.value, [fileName]: url }
  } catch {
  } finally {
    const next = { ...thumbLoadingByFileName.value }
    delete next[fileName]
    thumbLoadingByFileName.value = next
  }
}

const loadThumbnailsWithLimit = async (fileNames: string[], limit = 4) => {
  const queue = [...fileNames]
  const workers = Array.from({ length: Math.max(1, limit) }).map(async () => {
    while (queue.length > 0) {
      const name = queue.shift()
      if (!name) return
      await loadThumbnail(name)
    }
  })
  await Promise.all(workers)
}

const loadCharacterCards = async () => {
  loading.value = true
  errorMsg.value = ''
  revokeAllThumbs()
  try {
    const list = await invoke<CharacterCardFile[]>('list_character_card_pngs')
    // 按照修改时间从新到旧排序
    list.sort((a, b) => (b.modifiedMs || 0) - (a.modifiedMs || 0))
    characterCards.value = list
    currentPage.value = 1 // 重置页码
    void loadThumbnailsWithLimit(list.map(i => i.fileName))
  } catch (e: any) {
    errorMsg.value = e?.message ? String(e.message) : String(e)
    characterCards.value = []
  } finally {
    loading.value = false
  }
}

const loadWorldInfos = async () => {
  loading.value = true
  errorMsg.value = ''
  try {
    const list = await invoke<WorldInfoFile[]>('list_world_infos')
    list.sort((a, b) => (b.modifiedMs || 0) - (a.modifiedMs || 0))
    worldInfos.value = list
    currentPage.value = 1
  } catch (e: any) {
    errorMsg.value = e?.message ? String(e.message) : String(e)
    worldInfos.value = []
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await loadCharacterCards()
  await loadWorldInfos()
})

onUnmounted(() => {
  revokeAllThumbs()
})
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="flex items-center justify-between mb-6 px-1">
      <h1 class="text-2xl font-bold dark:text-slate-100">{{ t('resources.title') }}</h1>
      <div class="flex items-center gap-2">
        <button
          v-if="activeTab === 'characters'"
          @click="importCard()"
          class="px-3 py-2 rounded-lg text-sm font-medium transition-colors bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 hover:bg-blue-100 dark:hover:bg-blue-900/50 hover:text-blue-700 dark:hover:text-blue-300 flex items-center gap-2 border border-blue-200/50 dark:border-blue-800/50"
          type="button"
        >
          <PhPlus :size="16" weight="bold" />
          {{ t('resources.addCharacterCard') }}
        </button>
        <button
          v-if="activeTab === 'worlds'"
          @click="importWorld()"
          class="px-3 py-2 rounded-lg text-sm font-medium transition-colors bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 hover:bg-blue-100 dark:hover:bg-blue-900/50 hover:text-blue-700 dark:hover:text-blue-300 flex items-center gap-2 border border-blue-200/50 dark:border-blue-800/50"
          type="button"
        >
          <PhPlus :size="16" weight="bold" />
          {{ t('resources.addWorldInfo') }}
        </button>
        <button
          @click="activeTab === 'characters' ? loadCharacterCards() : loadWorldInfos()"
          class="px-3 py-2 rounded-lg text-sm font-medium transition-colors bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-slate-100 hover:bg-slate-50 dark:hover:bg-slate-700 flex items-center gap-2"
          type="button"
        >
          <PhArrowsClockwise :size="16" weight="duotone" :class="loading ? 'animate-spin' : ''" />
          {{ t('common.refresh') }}
        </button>
      </div>
    </div>

    <div class="flex space-x-1 bg-slate-100 dark:bg-slate-800 p-1 rounded-xl w-fit mb-6 shrink-0">
      <button
        @click="activeTab = 'characters'"
        :class="[
          'px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 flex items-center gap-2',
          activeTab === 'characters'
            ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-sm'
            : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-700/50'
        ]"
        type="button"
      >
        <PhUserSquare :size="16" weight="duotone" />
        {{ t('resources.characterCards') }}
      </button>
      <button
        @click="activeTab = 'worlds'"
        :class="[
          'px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 flex items-center gap-2',
          activeTab === 'worlds'
            ? 'bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-sm'
            : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-700/50'
        ]"
        type="button"
      >
        <PhGlobe :size="16" weight="duotone" />
        {{ t('resources.worldInfos') }}
      </button>
    </div>

    <div class="flex-1 overflow-y-auto px-1 pb-10 min-h-0 relative">
      <div v-if="activeTab === 'characters'" class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
        <div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 shadow-sm">
          <div class="flex items-center justify-between gap-4">
            <div class="flex items-center gap-3 min-w-0">
              <div class="text-sm font-medium text-slate-700 dark:text-slate-300">{{ t('resources.characterCards') }}</div>
              <div class="w-px h-4 bg-slate-200 dark:bg-slate-600"></div>
              
              <button
                @click="toggleSelectMode"
                class="text-xs font-medium px-2 py-1 rounded transition-colors"
                :class="isSelectMode ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400' : 'text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 hover:text-slate-700 dark:hover:text-slate-200'"
              >
                {{ isSelectMode ? t('resources.exitSelection') : t('resources.batchOperations') }}
              </button>
              
              <template v-if="isSelectMode">
                <button
                  @click="selectAllOnPage"
                  class="text-xs font-medium px-2 py-1 rounded text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 hover:text-slate-700 dark:hover:text-slate-200 transition-colors"
                >
                  {{ t('resources.selectAllOnPage') }}
                </button>
                <div v-if="selectedFiles.size > 0" class="flex items-center gap-2">
                  <span class="text-xs text-slate-500 dark:text-slate-400">{{ t('resources.selectedItems', { count: selectedFiles.size }) }}</span>
                  <button
                    @click="deleteSelected"
                    class="text-xs font-medium px-2 py-1 rounded bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400 hover:bg-red-100 dark:hover:bg-red-900/50 transition-colors flex items-center gap-1"
                  >
                    <PhTrash :size="14" />
                    {{ t('resources.deleteSelected') }}
                  </button>
                </div>
              </template>
            </div>
            <div class="text-xs text-slate-500 dark:text-slate-400 font-medium shrink-0">{{ t('resources.totalCards', { count: totalCount }) }}</div>
          </div>
        </div>

        <div v-if="errorMsg" class="bg-red-50 dark:bg-red-900/20 border border-red-100 dark:border-red-800 rounded-xl p-4 text-sm text-red-600 dark:text-red-400">
          {{ errorMsg }}
        </div>

        <div v-else-if="!loading && totalCount === 0" class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-8 shadow-sm text-center text-slate-400">
          <div class="text-sm font-medium text-slate-500 dark:text-slate-400">{{ t('resources.noCharacterCards') }}</div>
          <div class="text-xs text-slate-400 mt-1">{{ t('resources.noCharacterCardsHint') }}</div>
        </div>

        <div v-else class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
          <button
            v-for="card in paginatedCards"
            :key="card.fileName"
            type="button"
            class="w-full text-left flex flex-col h-full relative group"
            @click="handleItemClick(card.fileName, $event)"
          >
            <!-- 多选状态遮罩与复选框 -->
            <div
              v-if="isSelectMode"
              class="absolute inset-0 z-10 rounded-2xl border-2 transition-all pointer-events-none"
              :class="selectedFiles.has(card.fileName) ? 'border-blue-500 bg-blue-500/10' : 'border-transparent group-hover:border-slate-300'"
            >
              <div class="absolute top-3 left-3 bg-white rounded shadow-sm">
                <CheckSquare v-if="selectedFiles.has(card.fileName)" class="w-5 h-5 text-blue-500" />
                <Square v-else class="w-5 h-5 text-slate-300" />
              </div>
            </div>

            <!-- 删除单张卡片按钮 (非选择模式下 hover 显示) -->
            <button
              v-if="!isSelectMode"
              @click="deleteSingle(card.fileName, $event)"
              class="absolute top-3 right-3 z-10 p-1.5 bg-white/90 backdrop-blur text-red-500 hover:bg-red-50 hover:text-red-600 rounded-lg opacity-0 group-hover:opacity-100 transition-opacity shadow-sm"
              :title="t('common.delete')"
            >
              <PhTrash :size="16" weight="bold" />
            </button>

            <div class="bg-white dark:bg-slate-800 rounded-2xl border border-slate-200 dark:border-slate-700 shadow-sm hover:shadow-soft transition-shadow overflow-hidden flex-1 flex flex-col">
              <div class="bg-slate-100 dark:bg-slate-700 aspect-2/3 shrink-0">
                <img
                  v-if="thumbUrlByFileName[card.fileName]"
                  :src="thumbUrlByFileName[card.fileName]"
                  class="w-full h-full object-cover block"
                  :alt="card.fileName"
                  loading="lazy"
                />
                <div v-else class="w-full h-full flex items-center justify-center text-slate-400 dark:text-slate-500 text-xs font-medium">
                  {{ t('common.loading') }}
                </div>
              </div>
              <div class="p-3 flex-1 flex flex-col justify-between">
                <div class="text-sm font-semibold text-slate-800 dark:text-slate-100 line-clamp-2 leading-tight">{{ card.fileName }}</div>
                <div class="text-xs text-slate-500 dark:text-slate-400 mt-2 shrink-0">{{ formatSize(card.size) }}</div>
              </div>
            </div>
          </button>
        </div>

        <!-- Pagination Controls -->
        <div v-if="totalPages > 1" class="p-4 border-t border-slate-100 dark:border-slate-700 flex items-center justify-between bg-slate-50/50 dark:bg-slate-800/50 rounded-xl mt-6">
          <span class="text-sm text-slate-500 dark:text-slate-400">
            {{ t('resources.totalCards', { count: totalCount }) }}
          </span>
          <div class="flex items-center gap-2">
            <button 
              @click="prevPage" 
              :disabled="currentPage === 1"
              class="p-1.5 rounded-lg border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-300 hover:bg-white dark:hover:bg-slate-700 hover:text-slate-900 dark:hover:text-slate-100 disabled:opacity-50 disabled:cursor-not-allowed transition-colors bg-slate-50 dark:bg-slate-800"
            >
              <ChevronLeft class="w-4 h-4" />
            </button>
            <span class="text-sm font-medium text-slate-700 dark:text-slate-300 min-w-12 text-center">
              {{ currentPage }} / {{ totalPages }}
            </span>
            <button 
              @click="nextPage" 
              :disabled="currentPage === totalPages"
              class="p-1.5 rounded-lg border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-300 hover:bg-white dark:hover:bg-slate-700 hover:text-slate-900 dark:hover:text-slate-100 disabled:opacity-50 disabled:cursor-not-allowed transition-colors bg-slate-50 dark:bg-slate-800"
            >
              <ChevronRight class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      <div v-else-if="activeTab === 'worlds'" class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
        <div class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-4 shadow-sm">
          <div class="flex items-center justify-between gap-4">
            <div class="flex items-center gap-3 min-w-0">
              <div class="text-sm font-medium text-slate-700 dark:text-slate-300">{{ t('resources.worldInfos') }}</div>
              <div class="w-px h-4 bg-slate-200 dark:bg-slate-600"></div>
              
              <button
                @click="toggleSelectMode"
                class="text-xs font-medium px-2 py-1 rounded transition-colors"
                :class="isSelectMode ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400' : 'text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 hover:text-slate-700 dark:hover:text-slate-200'"
              >
                {{ isSelectMode ? t('resources.exitSelection') : t('resources.batchOperations') }}
              </button>
              
              <template v-if="isSelectMode">
                <button
                  @click="selectAllOnPage"
                  class="text-xs font-medium px-2 py-1 rounded text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 hover:text-slate-700 dark:hover:text-slate-200 transition-colors"
                >
                  {{ t('resources.selectAllOnPage') }}
                </button>
                <div v-if="selectedFiles.size > 0" class="flex items-center gap-2">
                  <span class="text-xs text-slate-500 dark:text-slate-400">{{ t('resources.selectedItems', { count: selectedFiles.size }) }}</span>
                  <button
                    @click="deleteSelected"
                    class="text-xs font-medium px-2 py-1 rounded bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400 hover:bg-red-100 dark:hover:bg-red-900/50 transition-colors flex items-center gap-1"
                  >
                    <PhTrash :size="14" />
                    {{ t('resources.deleteSelected') }}
                  </button>
                </div>
              </template>
            </div>
            <div class="text-xs text-slate-500 dark:text-slate-400 font-medium shrink-0">{{ t('resources.totalInfos', { count: totalCount }) }}</div>
          </div>
        </div>

        <div v-if="errorMsg" class="bg-red-50 dark:bg-red-900/20 border border-red-100 dark:border-red-800 rounded-xl p-4 text-sm text-red-600 dark:text-red-400">
          {{ errorMsg }}
        </div>

        <div v-else-if="!loading && totalCount === 0" class="bg-white dark:bg-slate-800 rounded-xl border border-slate-200 dark:border-slate-700 p-8 shadow-sm text-center text-slate-400">
          <div class="text-sm font-medium text-slate-500 dark:text-slate-400">{{ t('resources.noWorldInfos') }}</div>
          <div class="text-xs text-slate-400 mt-1">{{ t('resources.noWorldInfosHint') }}</div>
        </div>

        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
          <button
            v-for="world in paginatedWorlds"
            :key="world.fileName"
            type="button"
            class="w-full text-left relative group"
            @click="handleItemClick(world.fileName, $event)"
          >
            <div
              v-if="isSelectMode"
              class="absolute top-1/2 -translate-y-1/2 right-4 z-10 bg-white rounded shadow-sm transition-all pointer-events-none"
            >
              <CheckSquare v-if="selectedFiles.has(world.fileName)" class="w-5 h-5 text-blue-500" />
              <Square v-else class="w-5 h-5 text-slate-300" />
            </div>

            <button
              v-if="!isSelectMode"
              @click="deleteSingle(world.fileName, $event)"
              class="absolute top-1/2 -translate-y-1/2 right-3 z-10 p-1.5 bg-white/90 dark:bg-slate-800/90 backdrop-blur text-red-500 hover:bg-red-50 dark:hover:bg-red-900/30 hover:text-red-600 dark:hover:text-red-400 rounded-lg opacity-0 group-hover:opacity-100 transition-opacity shadow-sm"
              :title="t('common.delete')"
            >
              <PhTrash :size="16" weight="bold" />
            </button>

            <div class="bg-white dark:bg-slate-800 rounded-xl border transition-all p-4 flex items-center gap-4"
              :class="[
                isSelectMode && selectedFiles.has(world.fileName) ? 'border-blue-500 dark:border-blue-400 bg-blue-50/30 dark:bg-blue-900/20' : 'border-slate-200 dark:border-slate-700 hover:shadow-soft',
                isSelectMode ? 'pr-12' : ''
              ]"
            >
              <div class="w-10 h-10 rounded-lg bg-blue-50 dark:bg-blue-900/30 flex items-center justify-center shrink-0">
                <BookOpen class="w-5 h-5 text-blue-500 dark:text-blue-400" />
              </div>
              <div class="flex-1 min-w-0">
                <div class="text-sm font-semibold text-slate-800 dark:text-slate-100 truncate">{{ world.fileName }}</div>
                <div class="text-xs text-slate-500 dark:text-slate-400 mt-1">{{ formatSize(world.size) }}</div>
              </div>
            </div>
          </button>
        </div>

        <div v-if="totalPages > 1" class="p-4 border-t border-slate-100 dark:border-slate-700 flex items-center justify-between bg-slate-50/50 dark:bg-slate-800/50 rounded-xl mt-6">
          <span class="text-sm text-slate-500 dark:text-slate-400">
            {{ t('resources.totalInfos', { count: totalCount }) }}
          </span>
          <div class="flex items-center gap-2">
            <button 
              @click="prevPage" 
              :disabled="currentPage === 1"
              class="p-1.5 rounded-lg border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-300 hover:bg-white dark:hover:bg-slate-700 hover:text-slate-900 dark:hover:text-slate-100 disabled:opacity-50 disabled:cursor-not-allowed transition-colors bg-slate-50 dark:bg-slate-800"
            >
              <ChevronLeft class="w-4 h-4" />
            </button>
            <span class="text-sm font-medium text-slate-700 dark:text-slate-300 min-w-12 text-center">
              {{ currentPage }} / {{ totalPages }}
            </span>
            <button 
              @click="nextPage" 
              :disabled="currentPage === totalPages"
              class="p-1.5 rounded-lg border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-300 hover:bg-white dark:hover:bg-slate-700 hover:text-slate-900 dark:hover:text-slate-100 disabled:opacity-50 disabled:cursor-not-allowed transition-colors bg-slate-50 dark:bg-slate-800"
            >
              <ChevronRight class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

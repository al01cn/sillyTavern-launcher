<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { getCharacterInfo } from 'gstinfo'
import { PhUserSquare, PhArrowsClockwise, PhTrash, PhPlus } from '@phosphor-icons/vue'
import { ChevronLeft, ChevronRight, CheckSquare, Square } from 'lucide-vue-next'
import { openCharacterCardDialog, openImportCharacterCardDialog } from '../lib/useCharacterCardDialog'
import { Dialog } from '../lib/useDialog'

const activeTab = ref<'characters'>('characters')

interface CharacterCardFile {
  fileName: string
  size: number
  modifiedMs: number | null
}

const loading = ref(false)
const errorMsg = ref('')
const characterCards = ref<CharacterCardFile[]>([])
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
  
  for (const card of paginatedCards.value) {
    if (!newSet.has(card.fileName)) {
      allSelected = false
      break
    }
  }

  if (allSelected) {
    for (const card of paginatedCards.value) {
      newSet.delete(card.fileName)
    }
  } else {
    for (const card of paginatedCards.value) {
      newSet.add(card.fileName)
    }
  }
  selectedFiles.value = newSet
}

const handleCardClick = (fileName: string, event: Event) => {
  if (isSelectMode.value) {
    toggleSelectFile(fileName, event)
  } else {
    openCharacterCardDialog(fileName)
  }
}

const deleteSelected = async () => {
  if (selectedFiles.value.size === 0) return
  if (!confirm(`确定要删除选中的 ${selectedFiles.value.size} 个角色卡吗？此操作不可恢复。`)) return
  
  loading.value = true
  try {
    await invoke('delete_character_cards', { fileNames: Array.from(selectedFiles.value) })
    selectedFiles.value.clear()
    isSelectMode.value = false
    await loadCharacterCards()
  } catch (e: any) {
    errorMsg.value = `删除失败: ${e?.message || String(e)}`
    loading.value = false
  }
}

const deleteSingle = async (fileName: string, event: Event) => {
  event.stopPropagation()
  if (!confirm(`确定要删除角色卡 "${fileName}" 吗？此操作不可恢复。`)) return
  
  loading.value = true
  try {
    await invoke('delete_character_cards', { fileNames: [fileName] })
    await loadCharacterCards()
  } catch (e: any) {
    errorMsg.value = `删除失败: ${e?.message || String(e)}`
    loading.value = false
  }
}

const importCard = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Image',
        extensions: ['png']
      }]
    })
    
    if (selected) {
      loading.value = true
      // 1. 读取本地文件
      const bytes = await invoke<number[]>('read_local_file', { path: selected })
      const u8 = new Uint8Array(bytes)
      
      // 2. 尝试解析角色卡信息
      try {
        const info = await getCharacterInfo(u8)
        if (!info || !info.name) {
          throw new Error('未获取到有效的角色卡信息')
        }
        
        // 3. 解析成功，打开添加角色卡确认弹窗
        const fileName = selected.split(/[/\\]/).pop() || 'unknown.png'
        openImportCharacterCardDialog(selected, fileName)
      } catch (err: any) {
        Dialog.error({
          title: '导入失败',
          context: '该图片不包含有效的角色卡数据（V2/V3规范），请选择正确的角色卡图片。'
        })
      }
    }
  } catch (e: any) {
    Dialog.error({
      title: '错误',
      context: `读取文件失败: ${e?.message || String(e)}`
    })
  } finally {
    loading.value = false
  }
}

// 监听弹窗导入成功事件（可以利用自定义事件或直接通过重新加载数据来实现）
window.addEventListener('character-card-imported', () => {
  loadCharacterCards()
})

const currentPage = ref(1)
const pageSize = 10

const totalCount = computed(() => characterCards.value.length)
const totalPages = computed(() => Math.max(1, Math.ceil(totalCount.value / pageSize)))

const paginatedCards = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return characterCards.value.slice(start, start + pageSize)
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

onMounted(async () => {
  await loadCharacterCards()
})

onUnmounted(() => {
  revokeAllThumbs()
})
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="flex items-center justify-between mb-6 px-1">
      <h1 class="text-2xl font-bold">资源管理</h1>
      <div class="flex items-center gap-2">
        <button
          @click="importCard()"
          class="px-3 py-2 rounded-lg text-sm font-medium transition-colors bg-blue-50 text-blue-600 hover:bg-blue-100 hover:text-blue-700 flex items-center gap-2 border border-blue-200/50"
          type="button"
        >
          <PhPlus :size="16" weight="bold" />
          添加角色卡
        </button>
        <button
          @click="loadCharacterCards()"
          class="px-3 py-2 rounded-lg text-sm font-medium transition-colors bg-white border border-slate-200 text-slate-600 hover:text-slate-900 hover:bg-slate-50 flex items-center gap-2"
          type="button"
        >
          <PhArrowsClockwise :size="16" weight="duotone" :class="loading ? 'animate-spin' : ''" />
          刷新
        </button>
      </div>
    </div>

    <div class="flex space-x-1 bg-slate-100 p-1 rounded-xl w-fit mb-6 shrink-0">
      <button
        @click="activeTab = 'characters'"
        :class="[
          'px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 flex items-center gap-2',
          activeTab === 'characters'
            ? 'bg-white text-slate-900 shadow-sm'
            : 'text-slate-500 hover:text-slate-700 hover:bg-slate-200/50'
        ]"
        type="button"
      >
        <PhUserSquare :size="16" weight="duotone" />
        角色卡
      </button>
    </div>

    <div class="flex-1 overflow-y-auto px-1 pb-10 min-h-0 relative">
      <div v-if="activeTab === 'characters'" class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
        <div class="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
          <div class="flex items-center justify-between gap-4">
            <div class="flex items-center gap-3 min-w-0">
              <div class="text-sm font-medium text-slate-700">角色卡</div>
              <div class="w-px h-4 bg-slate-200"></div>
              
              <button
                @click="toggleSelectMode"
                class="text-xs font-medium px-2 py-1 rounded transition-colors"
                :class="isSelectMode ? 'bg-blue-50 text-blue-600' : 'text-slate-500 hover:bg-slate-100 hover:text-slate-700'"
              >
                {{ isSelectMode ? '退出选择' : '批量操作' }}
              </button>
              
              <template v-if="isSelectMode">
                <button
                  @click="selectAllOnPage"
                  class="text-xs font-medium px-2 py-1 rounded text-slate-500 hover:bg-slate-100 hover:text-slate-700 transition-colors"
                >
                  本页全选/取消
                </button>
                <div v-if="selectedFiles.size > 0" class="flex items-center gap-2">
                  <span class="text-xs text-slate-500">已选 {{ selectedFiles.size }} 项</span>
                  <button
                    @click="deleteSelected"
                    class="text-xs font-medium px-2 py-1 rounded bg-red-50 text-red-600 hover:bg-red-100 transition-colors flex items-center gap-1"
                  >
                    <PhTrash :size="14" />
                    删除选中
                  </button>
                </div>
              </template>
            </div>
            <div class="text-xs text-slate-500 font-medium shrink-0">共 {{ totalCount }} 张</div>
          </div>
        </div>

        <div v-if="errorMsg" class="bg-red-50 border border-red-100 rounded-xl p-4 text-sm text-red-600">
          {{ errorMsg }}
        </div>

        <div v-else-if="!loading && totalCount === 0" class="bg-white rounded-xl border border-slate-200 p-8 shadow-sm text-center text-slate-400">
          <div class="text-sm font-medium text-slate-500">未找到角色卡</div>
          <div class="text-xs text-slate-400 mt-1">请确认目录下存在 .png 角色卡文件</div>
        </div>

        <div v-else class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
          <button
            v-for="card in paginatedCards"
            :key="card.fileName"
            type="button"
            class="w-full text-left flex flex-col h-full relative group"
            @click="handleCardClick(card.fileName, $event)"
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
              title="删除角色卡"
            >
              <PhTrash :size="16" weight="bold" />
            </button>

            <div class="bg-white rounded-2xl border border-slate-200 shadow-sm hover:shadow-soft transition-shadow overflow-hidden flex-1 flex flex-col">
              <div class="bg-slate-100 aspect-[2/3] shrink-0">
                <img
                  v-if="thumbUrlByFileName[card.fileName]"
                  :src="thumbUrlByFileName[card.fileName]"
                  class="w-full h-full object-cover block"
                  :alt="card.fileName"
                  loading="lazy"
                />
                <div v-else class="w-full h-full flex items-center justify-center text-slate-400 text-xs font-medium">
                  正在加载...
                </div>
              </div>
              <div class="p-3 flex-1 flex flex-col justify-between">
                <div class="text-sm font-semibold text-slate-800 line-clamp-2 leading-tight">{{ card.fileName }}</div>
                <div class="text-xs text-slate-500 mt-2 shrink-0">{{ formatSize(card.size) }}</div>
              </div>
            </div>
          </button>
        </div>

        <!-- Pagination Controls -->
        <div v-if="totalPages > 1" class="p-4 border-t border-slate-100 flex items-center justify-between bg-slate-50/50 rounded-xl mt-6">
          <span class="text-sm text-slate-500">
            共 {{ totalCount }} 个角色卡
          </span>
          <div class="flex items-center gap-2">
            <button 
              @click="prevPage" 
              :disabled="currentPage === 1"
              class="p-1.5 rounded-lg border border-slate-200 text-slate-600 hover:bg-white hover:text-slate-900 disabled:opacity-50 disabled:cursor-not-allowed transition-colors bg-slate-50"
            >
              <ChevronLeft class="w-4 h-4" />
            </button>
            <span class="text-sm font-medium text-slate-700 min-w-[3rem] text-center">
              {{ currentPage }} / {{ totalPages }}
            </span>
            <button 
              @click="nextPage" 
              :disabled="currentPage === totalPages"
              class="p-1.5 rounded-lg border border-slate-200 text-slate-600 hover:bg-white hover:text-slate-900 disabled:opacity-50 disabled:cursor-not-allowed transition-colors bg-slate-50"
            >
              <ChevronRight class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

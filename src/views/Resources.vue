<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { PhUserSquare, PhArrowsClockwise } from '@phosphor-icons/vue'
import { openCharacterCardDialog } from '../lib/useCharacterCardDialog'

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

const totalCount = computed(() => characterCards.value.length)

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
    characterCards.value = list
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
      <button
        @click="loadCharacterCards()"
        class="px-3 py-2 rounded-lg text-sm font-medium transition-colors bg-white border border-slate-200 text-slate-600 hover:text-slate-900 hover:bg-slate-50 flex items-center gap-2"
        type="button"
      >
        <PhArrowsClockwise :size="16" weight="duotone" :class="loading ? 'animate-spin' : ''" />
        刷新
      </button>
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
        角色卡管理
      </button>
    </div>

    <div class="flex-1 overflow-y-auto px-1 pb-10 min-h-0 relative">
      <div v-if="activeTab === 'characters'" class="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
        <div class="bg-white rounded-xl border border-slate-200 p-4 shadow-sm">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <div class="text-sm font-medium text-slate-700">角色卡</div>
              <div class="text-xs text-slate-500 mt-1">读取目录：data/st_data/characters（仅 *.png，不包含子文件夹）</div>
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

        <div v-else class="columns-2 md:columns-3 lg:columns-4 gap-4">
          <button
            v-for="card in characterCards"
            :key="card.fileName"
            type="button"
            class="mb-4 inline-block w-full text-left"
            @click="openCharacterCardDialog(card.fileName)"
          >
            <div class="bg-white rounded-2xl border border-slate-200 shadow-sm hover:shadow-soft transition-shadow overflow-hidden">
              <div class="bg-slate-100">
                <img
                  v-if="thumbUrlByFileName[card.fileName]"
                  :src="thumbUrlByFileName[card.fileName]"
                  class="w-full h-auto block"
                  :alt="card.fileName"
                  loading="lazy"
                />
                <div v-else class="h-36 flex items-center justify-center text-slate-400 text-xs font-medium">
                  正在加载...
                </div>
              </div>
              <div class="p-3">
                <div class="text-sm font-semibold text-slate-800 truncate">{{ card.fileName }}</div>
                <div class="text-xs text-slate-500 mt-1">{{ formatSize(card.size) }}</div>
              </div>
            </div>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

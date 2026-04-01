<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getWorldInfo } from 'gstinfo'
import { X, Loader2, BookOpen, Tags } from 'lucide-vue-next'
import { worldInfoDialogState, closeWorldInfoDialog } from '../lib/useWorldInfoDialog'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const loading = ref(false)
const errorMsg = ref('')
const info = ref<any>(null)

const title = computed(() => {
  if (!info.value) return worldInfoDialogState.fileName
  const raw = info.value.raw || {}
  const data = raw.originalData || raw
  return data.name || worldInfoDialogState.fileName
})

const description = computed(() => {
  if (!info.value) return ''
  const raw = info.value.raw || {}
  const data = raw.originalData || raw
  return data.description || data.comment || ''
})

type WorldEntry = {
  uid?: number | string
  keys: string[]
  comment: string
  content: string
  enabled: boolean | null
  position: string
  insertionOrder: number | string | null
}

const entries = computed<WorldEntry[]>(() => {
  if (!info.value) return []

  const raw = info.value.raw || {}
  const data = raw.originalData || raw
  const rawEntries = data.entries

  if (!rawEntries) return []

  let entriesArray: any[] = []

  if (Array.isArray(rawEntries)) {
    entriesArray = rawEntries
  } else if (typeof rawEntries === 'object') {
    entriesArray = Object.values(rawEntries)
  }

  return entriesArray.map((entry: any) => {
    let keys: string[] = []
    if (Array.isArray(entry.key)) {
      keys = entry.key
    } else if (Array.isArray(entry.keys)) {
      keys = entry.keys
    } else if (typeof entry.key === 'string') {
      keys = entry.key
        .split(',')
        .map((k: string) => k.trim())
        .filter(Boolean)
    }

    let enabled: boolean | null = null
    if (typeof entry.enabled === 'boolean') {
      enabled = entry.enabled
    } else if (typeof entry.disable === 'boolean') {
      enabled = !entry.disable
    } else if (entry.enabled !== undefined) {
      enabled = entry.enabled !== false
    }

    return {
      uid: entry.uid || entry.id,
      keys,
      comment: entry.comment || entry.name || '',
      content: entry.content || '',
      enabled,
      position: entry.position || 'before_char',
      insertionOrder: entry.insertion_order || entry.insertionOrder || null,
    }
  })
})

const translatePosition = (pos: string) => {
  if (!pos) return '—'
  const key = String(pos).toLowerCase()

  const positionMap: Record<string, string> = {
    before_char: t('resources.positionBeforeChar'),
    after_char: t('resources.positionAfterChar'),
    '0': t('resources.positionBeforeChar'),
    '1': t('resources.positionAfterChar'),
    '2': t('resources.positionAfterAN'),
    before_example: t('resources.positionBeforeExample'),
    after_example: t('resources.positionAfterExample'),
    before_prompt: t('resources.positionBeforePrompt'),
    after_prompt: t('resources.positionAfterPrompt'),
    before_author: t('resources.positionBeforeAuthor'),
    after_author: t('resources.positionAfterAN'),
    at_depth: t('resources.positionAtDepth'),
  }

  return positionMap[key] || pos
}

const loadData = async (fileName: string) => {
  if (!fileName) return
  loading.value = true
  errorMsg.value = ''
  info.value = null

  try {
    const jsonStr = await invoke<string>('read_world_info', { fileName })
    const blob = new Blob([jsonStr], { type: 'application/json' })
    const parsed = await getWorldInfo(blob as any)
    info.value = parsed
    console.log(info.value)
  } catch (e: any) {
    errorMsg.value = `${t('resources.loadFailed')}: ${e?.message || String(e)}`
  } finally {
    loading.value = false
  }
}

watch(
  () => worldInfoDialogState.visible,
  visible => {
    if (visible && worldInfoDialogState.fileName) {
      loadData(worldInfoDialogState.fileName)
    } else {
      info.value = null
      errorMsg.value = ''
    }
  },
)
</script>

<template>
  <div
    :class="[
      'absolute inset-0 z-[320] flex items-center justify-center px-4 transition-all duration-300',
      worldInfoDialogState.visible ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none',
    ]"
  >
    <!-- Backdrop click to close -->
    <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-md" @click="closeWorldInfoDialog"></div>

    <div
      :class="[
        'relative bg-white w-full max-w-3xl rounded-4xl shadow-modal border border-slate-100 overflow-hidden transition-all duration-300 transform flex flex-col',
        worldInfoDialogState.visible ? 'scale-100 translate-y-0' : 'scale-95 translate-y-8',
      ]"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-slate-100 bg-white z-10 shrink-0">
        <h2 class="text-xl font-bold text-slate-800 flex items-center gap-2 truncate">
          <BookOpen class="w-5 h-5 text-blue-500" />
          {{ title }}
        </h2>
        <button
          class="p-2 text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-lg transition-colors"
          :title="t('common.close')"
          @click="closeWorldInfoDialog"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 p-6 max-h-[75vh] overflow-hidden flex flex-col relative bg-slate-50/50">
        <div v-if="loading" class="absolute inset-0 flex items-center justify-center bg-white/50 backdrop-blur-sm z-10">
          <div class="flex flex-col items-center gap-3">
            <Loader2 class="w-8 h-8 text-blue-500 animate-spin" />
            <span class="text-sm font-medium text-slate-500">{{ t('resources.parsingWorld') }}</span>
          </div>
        </div>

        <div
          v-else-if="errorMsg"
          class="bg-red-50 border border-red-100 rounded-xl p-4 text-sm text-red-600 flex items-start gap-3"
        >
          <X class="w-5 h-5 text-red-500 shrink-0 mt-0.5" />
          <div class="flex-1 whitespace-pre-wrap">{{ errorMsg }}</div>
        </div>

        <div v-else-if="info" class="flex-1 overflow-y-auto pr-4 pb-2 -mr-4 custom-scrollbar space-y-8">
          <!-- 基本信息 -->
          <div v-if="description" class="bg-white p-5 rounded-xl border border-slate-200 shadow-sm">
            <div class="text-sm font-medium text-slate-500 mb-2 flex items-center gap-2">
              <BookOpen class="w-4 h-4" />
              {{ t('resources.description') }}
            </div>
            <div class="text-slate-700 whitespace-pre-wrap text-sm leading-relaxed">
              {{ description }}
            </div>
          </div>

          <!-- 词条列表 -->
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <h3 class="text-lg font-bold text-slate-800 flex items-center gap-2">
                <Tags class="w-5 h-5 text-purple-500" />
                {{ t('resources.entryList') }}
              </h3>
              <span class="text-sm font-medium text-slate-500 bg-slate-100 px-3 py-1 rounded-full">
                {{ t('resources.totalEntries', { count: entries.length }) }}
              </span>
            </div>

            <div v-if="entries.length > 0" class="space-y-2">
              <details
                v-for="(entry, idx) in entries"
                :key="idx"
                class="bg-white border border-slate-200 rounded-xl overflow-hidden"
              >
                <summary
                  class="px-3 py-2 cursor-pointer select-none flex items-center justify-between gap-3 bg-slate-50/80"
                >
                  <div class="min-w-0">
                    <div class="text-xs font-semibold text-slate-800 truncate">
                      #{{ idx + 1 }}
                      <span class="text-slate-500 font-medium">
                        {{ entry.keys.length ? entry.keys.join(', ') : t('resources.noKeys') }}
                      </span>
                    </div>
                    <div class="text-[11px] text-slate-500 truncate mt-0.5">
                      {{ entry.comment || t('resources.noComment') }}
                    </div>
                  </div>
                  <span
                    :class="[
                      'shrink-0 px-2 py-1 rounded-lg text-[11px] font-bold border',
                      entry.enabled === null
                        ? 'bg-slate-50 text-slate-500 border-slate-200'
                        : entry.enabled
                          ? 'bg-emerald-50 text-emerald-700 border-emerald-100'
                          : 'bg-slate-50 text-slate-500 border-slate-200',
                    ]"
                  >
                    {{
                      entry.enabled === null
                        ? t('resources.unknownStatus')
                        : entry.enabled
                          ? t('resources.enabled')
                          : t('resources.disabled')
                    }}
                  </span>
                </summary>

                <div class="p-3 border-t border-slate-200">
                  <div class="grid grid-cols-2 gap-2 text-[11px] text-slate-600">
                    <div class="truncate">
                      <span class="font-semibold">{{ t('resources.position') }}：</span>
                      <span>{{ translatePosition(entry.position) }}</span>
                    </div>
                    <div class="truncate">
                      <span class="font-semibold">{{ t('resources.insertionOrder') }}：</span>
                      <span>{{ entry.insertionOrder ?? '—' }}</span>
                    </div>
                  </div>

                  <div class="mt-2 text-[11px] text-slate-600">
                    <span class="font-semibold">{{ t('resources.keywords') }}：</span>
                    <span>{{ entry.keys.length ? entry.keys.join(', ') : '—' }}</span>
                  </div>

                  <div class="mt-2 text-[11px] text-slate-600">
                    <span class="font-semibold">{{ t('resources.comment') }}：</span>
                    <span>{{ entry.comment || '—' }}</span>
                  </div>

                  <div class="mt-2 text-[11px] text-slate-600 font-semibold">{{ t('resources.content') }}：</div>
                  <div class="mt-1 text-xs text-slate-700 whitespace-pre-wrap leading-relaxed">
                    {{ entry.content || '—' }}
                  </div>
                </div>
              </details>
            </div>

            <div v-else class="text-center py-12 bg-white rounded-xl border border-slate-200 border-dashed">
              <BookOpen class="w-10 h-10 text-slate-300 mx-auto mb-3" />
              <div class="text-slate-500 font-medium">{{ t('resources.noEntries') }}</div>
            </div>
          </div>
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
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background-color: #94a3b8;
}
</style>

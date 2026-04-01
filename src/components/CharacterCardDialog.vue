<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCharacterInfo, getValueByPath } from 'gstinfo'
import { X, Loader2, FileImage, Tags, User, BookOpen } from 'lucide-vue-next'
import { characterCardDialogState, closeCharacterCardDialog } from '../lib/useCharacterCardDialog'
import { Dialog } from '../lib/useDialog'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

type CharacterInfo = Awaited<ReturnType<typeof getCharacterInfo>>

const loading = ref(false)
const isImporting = ref(false)
const errorMsg = ref('')
const imageUrl = ref('')
const info = ref<CharacterInfo | null>(null)

const getFirstValueByPath = <T,>(obj: any, paths: string[], defaultValue: T): T => {
  for (const path of paths) {
    const v = getValueByPath<T>(obj, path, defaultValue)
    if (v !== defaultValue && v !== null && v !== undefined) return v
  }
  return defaultValue
}

const formatDateTime = (input: unknown) => {
  if (typeof input === 'number' && Number.isFinite(input)) {
    return new Date(input).toLocaleString('zh-CN')
  }
  if (typeof input === 'string' && input.trim()) {
    const d = new Date(input)
    if (!Number.isNaN(d.getTime())) return d.toLocaleString('zh-CN')
    return input
  }
  return ''
}

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

const translatePosition = (pos: string) => {
  if (!pos) return '—'
  const key = pos.toLowerCase()
  return positionMap[key] || pos
}

const title = computed(() => {
  if (!info.value) return characterCardDialogState.fileName
  const name =
    getValueByPath<string>(info.value as any, 'name', '') || getValueByPath<string>(info.value as any, 'data.name', '')
  return name || characterCardDialogState.fileName
})

const spec = computed(() => {
  if (!info.value) return ''
  return getFirstValueByPath<string>(info.value as any, ['spec', 'raw.spec', 'data.spec'], '')
})

const specVersion = computed(() => {
  if (!info.value) return ''
  return getFirstValueByPath<string>(
    info.value as any,
    ['spec_version', 'specVersion', 'raw.spec_version', 'raw.specVersion'],
    '',
  )
})

const createDate = computed(() => {
  if (!info.value) return ''
  const raw = getFirstValueByPath<any>(
    info.value as any,
    ['data.create_date', 'data.createDate', 'raw.create_date', 'createDate', 'data.create_time', 'data.createTime'],
    null,
  )
  return formatDateTime(raw)
})

const description = computed(() => {
  if (!info.value) return ''
  const v = getFirstValueByPath<string>(
    info.value as any,
    ['data.description', 'description', 'raw.data.description', 'raw.description'],
    '',
  )
  return typeof v === 'string' ? v : ''
})

const worldEntriesCount = computed<number>(() => {
  if (!info.value) return 0
  const v = getValueByPath<number | undefined>(info.value as any, 'worldInfo.entries.length', 0 as any)
  return typeof v === 'number' && Number.isFinite(v) ? v : 0
})

const worldInfoName = computed(() => {
  if (!info.value) return ''
  return getFirstValueByPath<string>(info.value as any, ['worldInfo.name', 'worldInfo.worldName'], '')
})

const hasCharacterBook = computed(() => {
  if (!info.value) return false
  const book = getFirstValueByPath<any>(
    info.value as any,
    ['data.character_book', 'data.characterBook', 'raw.data.character_book', 'raw.data.characterBook'],
    null,
  )
  return !!book
})

const characterBookName = computed(() => {
  if (!info.value) return ''
  return getFirstValueByPath<string>(
    info.value as any,
    [
      'data.character_book.name',
      'data.characterBook.name',
      'raw.data.character_book.name',
      'raw.data.characterBook.name',
    ],
    '',
  )
})

const characterBookEntriesCount = computed(() => {
  if (!info.value) return 0
  const entries = getFirstValueByPath<any>(
    info.value as any,
    [
      'data.character_book.entries',
      'data.characterBook.entries',
      'raw.data.character_book.entries',
      'raw.data.characterBook.entries',
    ],
    null,
  )
  if (Array.isArray(entries)) return entries.length
  return 0
})

const hasWorldInfo = computed(() => {
  if (!info.value) return false
  return worldEntriesCount.value > 0 || !!worldInfoName.value
})

const creatorName = computed(() => {
  if (!info.value) return ''
  return (
    getValueByPath<string>(info.value as any, 'data.creator', '') ||
    getValueByPath<string>(info.value as any, 'creator', '') ||
    ''
  )
})

const preferredBookSource = computed<'character_book' | 'worldInfo' | 'none'>(() => {
  if (!info.value) return 'none'
  if (hasCharacterBook.value) return 'character_book'
  if (hasWorldInfo.value) return 'worldInfo'
  return 'none'
})

const preferredBookLabel = computed(() => {
  if (preferredBookSource.value === 'character_book') return 'Character Book'
  if (preferredBookSource.value === 'worldInfo') return 'WorldInfo'
  return t('resources.none')
})

const preferredBookName = computed(() => {
  if (preferredBookSource.value === 'character_book') return characterBookName.value
  if (preferredBookSource.value === 'worldInfo') return worldInfoName.value
  return ''
})

const preferredBookEntriesCount = computed(() => {
  if (preferredBookSource.value === 'character_book') return characterBookEntriesCount.value
  if (preferredBookSource.value === 'worldInfo') return worldEntriesCount.value
  return 0
})

type WorldBookEntry = {
  comment: string
  content: string
  enabled: boolean | null
  insertionOrder: number | string | null
  position: string
  keys: string[]
}

const preferredBookEntries = computed<WorldBookEntry[]>(() => {
  if (!info.value) return []

  let rawEntries: any = []
  if (preferredBookSource.value === 'character_book') {
    rawEntries = getFirstValueByPath<any>(
      info.value as any,
      [
        'data.character_book.entries',
        'data.characterBook.entries',
        'raw.data.character_book.entries',
        'raw.data.characterBook.entries',
      ],
      [],
    )
  } else if (preferredBookSource.value === 'worldInfo') {
    rawEntries = getFirstValueByPath<any>(info.value as any, ['worldInfo.entries', 'raw.worldInfo.entries'], [])
  }

  if (!Array.isArray(rawEntries)) return []

  const normalizeBool = (v: any): boolean | null => {
    if (typeof v === 'boolean') return v
    if (typeof v === 'number') return v !== 0
    if (typeof v === 'string') {
      const s = v.trim().toLowerCase()
      if (s === 'true') return true
      if (s === 'false') return false
    }
    return null
  }

  const normalizeKeys = (v: any): string[] => {
    if (Array.isArray(v))
      return v
        .map(x => String(x))
        .map(s => s.trim())
        .filter(Boolean)
    if (typeof v === 'string')
      return v
        .split(',')
        .map(s => s.trim())
        .filter(Boolean)
    return []
  }

  return rawEntries.map((entry: any) => {
    const keysRaw = getFirstValueByPath<any>(entry, ['keys', 'key', 'keywords'], [])
    const insertionRaw = getFirstValueByPath<any>(entry, ['insertion_order', 'insertionOrder'], null)
    const enabledRaw = getFirstValueByPath<any>(entry, ['enabled', 'is_enabled', 'isEnabled'], null)
    const positionRaw = getFirstValueByPath<any>(entry, ['position'], '')
    const commentRaw = getFirstValueByPath<any>(entry, ['comment'], '')
    const contentRaw = getFirstValueByPath<any>(entry, ['content'], '')

    return {
      comment: typeof commentRaw === 'string' ? commentRaw : String(commentRaw ?? ''),
      content: typeof contentRaw === 'string' ? contentRaw : String(contentRaw ?? ''),
      enabled: normalizeBool(enabledRaw),
      insertionOrder: typeof insertionRaw === 'number' || typeof insertionRaw === 'string' ? insertionRaw : null,
      position: typeof positionRaw === 'string' ? positionRaw : String(positionRaw ?? ''),
      keys: normalizeKeys(keysRaw),
    } satisfies WorldBookEntry
  })
})

const tags = computed(() => {
  if (!info.value) return []
  const value = getValueByPath<string[] | string>(info.value as any, 'data.tags', []) as any
  if (Array.isArray(value)) return value.slice(0, 30)
  if (typeof value === 'string')
    return value
      .split(',')
      .map(s => s.trim())
      .filter(Boolean)
      .slice(0, 30)
  return []
})

const revokeImageUrl = () => {
  if (imageUrl.value) {
    URL.revokeObjectURL(imageUrl.value)
    imageUrl.value = ''
  }
}

const handleImport = async () => {
  if (!characterCardDialogState.importSourcePath) return

  isImporting.value = true
  try {
    await invoke('import_character_card', { sourcePath: characterCardDialogState.importSourcePath })
    window.dispatchEvent(new Event('character-card-imported'))
    Dialog.success({
      title: t('resources.importCardSuccess'),
      context: t('resources.importCardSuccessMsg', { name: characterCardDialogState.fileName }),
    })
    closeCharacterCardDialog()
  } catch (e: any) {
    Dialog.error({
      title: t('resources.importCardFailed'),
      context: e?.message || String(e),
    })
  } finally {
    isImporting.value = false
  }
}

const loadDetail = async () => {
  loading.value = true
  errorMsg.value = ''
  info.value = null
  revokeImageUrl()

  try {
    let bytes: number[]
    if (characterCardDialogState.isImportMode && characterCardDialogState.importSourcePath) {
      bytes = await invoke<number[]>('read_local_file', { path: characterCardDialogState.importSourcePath })
    } else {
      bytes = await invoke<number[]>('read_character_card_png', { fileName: characterCardDialogState.fileName })
    }

    const u8 = new Uint8Array(bytes)
    imageUrl.value = URL.createObjectURL(new Blob([u8], { type: 'image/png' }))
    info.value = (await getCharacterInfo(u8)) as CharacterInfo
  } catch (e: any) {
    errorMsg.value = e?.message ? String(e.message) : String(e)
  } finally {
    loading.value = false
  }
}

watch(
  () => [characterCardDialogState.show, characterCardDialogState.fileName] as const,
  async ([show]) => {
    if (!show) {
      loading.value = false
      errorMsg.value = ''
      info.value = null
      revokeImageUrl()
      return
    }
    await loadDetail()
  },
)

onUnmounted(() => {
  revokeImageUrl()
})
</script>

<template>
  <div
    :class="[
      'absolute inset-0 z-[320] flex items-center justify-center px-4 transition-all duration-300',
      characterCardDialogState.show ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none',
    ]"
  >
    <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-md" @click="closeCharacterCardDialog()"></div>

    <div
      :class="[
        'relative bg-white w-full max-w-3xl rounded-4xl shadow-modal border border-slate-100 overflow-hidden transition-all duration-300 transform',
        characterCardDialogState.show ? 'scale-100 translate-y-0' : 'scale-95 translate-y-8',
      ]"
    >
      <button
        type="button"
        class="absolute top-4 right-4 p-1.5 text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-xl transition-colors z-10"
        @click="closeCharacterCardDialog()"
      >
        <X class="w-5 h-5" />
      </button>

      <div class="p-6">
        <div class="flex items-start gap-5 max-h-[75vh] overflow-hidden">
          <div class="w-40 shrink-0 sticky top-0">
            <div class="bg-slate-100 rounded-2xl overflow-hidden border border-slate-200">
              <div v-if="!imageUrl" class="h-52 flex items-center justify-center text-slate-400">
                <FileImage class="w-7 h-7" />
              </div>
              <img v-else :src="imageUrl" class="w-full h-auto block" :alt="t('resources.characterCard')" />
            </div>
          </div>

          <div class="flex-1 min-w-0 flex flex-col overflow-hidden max-h-[calc(75vh-3rem)]">
            <div class="shrink-0 mb-4 pr-6">
              <div class="text-xl font-black text-slate-800 truncate">{{ title }}</div>
              <div class="text-xs text-slate-500 mt-1 truncate">{{ characterCardDialogState.fileName }}</div>
            </div>

            <div class="flex-1 overflow-y-auto pr-4 pb-2 -mr-4 custom-scrollbar">
              <div class="grid grid-cols-2 gap-3">
                <div class="bg-slate-50 border border-slate-200 rounded-xl p-3">
                  <div class="flex items-center gap-2 text-slate-700 text-sm font-semibold">
                    <User class="w-4 h-4 text-blue-500" />
                    {{ t('resources.author') }}
                  </div>
                  <div class="text-xs text-slate-500 mt-1 truncate">
                    {{ info ? creatorName || t('resources.unknown') : '—' }}
                  </div>
                </div>

                <div class="bg-slate-50 border border-slate-200 rounded-xl p-3">
                  <div class="flex items-center gap-2 text-slate-700 text-sm font-semibold">
                    <BookOpen class="w-4 h-4 text-indigo-500" />
                    {{ t('resources.worldBookEntries') }}
                  </div>
                  <div class="text-xs text-slate-500 mt-1">
                    {{ info ? `${preferredBookLabel} / ${preferredBookEntriesCount}` : '—' }}
                  </div>
                </div>
              </div>

              <div class="mt-3 grid grid-cols-2 gap-3">
                <div class="bg-slate-50 border border-slate-200 rounded-xl p-3">
                  <div class="text-slate-700 text-sm font-semibold">{{ t('resources.createDate') }}</div>
                  <div class="text-xs text-slate-500 mt-1 truncate">
                    {{ info ? createDate || t('resources.unknown') : '—' }}
                  </div>
                </div>

                <div class="bg-slate-50 border border-slate-200 rounded-xl p-3">
                  <div class="text-slate-700 text-sm font-semibold">{{ t('resources.specVersion') }}</div>
                  <div class="text-xs text-slate-500 mt-1 truncate">
                    {{
                      info
                        ? (spec ? `${spec}` : t('resources.unknown')) + (specVersion ? ` / ${specVersion}` : '')
                        : '—'
                    }}
                  </div>
                </div>
              </div>

              <div class="mt-3 bg-slate-50 border border-slate-200 rounded-xl p-3">
                <div class="text-slate-700 text-sm font-semibold">{{ t('resources.description') }}</div>
                <div
                  v-if="info"
                  class="text-xs text-slate-500 mt-1 leading-relaxed max-h-28 overflow-y-auto whitespace-pre-wrap"
                >
                  {{ description || t('resources.none') }}
                </div>
                <div v-else class="text-xs text-slate-400 mt-1">—</div>
              </div>

              <div class="mt-4">
                <div class="flex items-center gap-2 text-slate-700 text-sm font-semibold">
                  <Tags class="w-4 h-4 text-emerald-500" />
                  {{ t('resources.tags') }}
                </div>
                <div class="mt-2 flex flex-wrap gap-2">
                  <span
                    v-for="tag in tags"
                    :key="tag"
                    class="px-2 py-1 rounded-lg text-[11px] font-semibold bg-emerald-50 text-emerald-700 border border-emerald-100"
                  >
                    {{ tag }}
                  </span>
                  <span v-if="info && tags.length === 0" class="text-xs text-slate-500">{{ t('resources.none') }}</span>
                  <span v-if="!info" class="text-xs text-slate-400">—</span>
                </div>
              </div>

              <div class="mt-4 bg-slate-50 border border-slate-200 rounded-xl p-3">
                <div class="text-slate-700 text-sm font-semibold">{{ t('resources.worldBook') }}</div>
                <div class="text-xs text-slate-500 mt-1 space-y-1">
                  <div v-if="info">
                    <span class="font-medium text-slate-600">{{ t('resources.source') }}：</span>
                    <span>{{ preferredBookLabel }}</span>
                  </div>
                  <div v-if="info">
                    <span class="font-medium text-slate-600">{{ t('resources.name') }}：</span>
                    <span>{{
                      preferredBookSource === 'none' ? t('resources.none') : preferredBookName || t('resources.unknown')
                    }}</span>
                  </div>
                  <div v-if="info">
                    <span class="font-medium text-slate-600">{{ t('resources.entries') }}：</span>
                    <span>{{ preferredBookEntriesCount }}</span>
                  </div>
                  <div v-if="!info" class="text-slate-400">—</div>
                </div>
              </div>

              <div v-if="info && preferredBookSource === 'none'" class="mt-3 text-xs text-slate-400">
                {{ t('resources.noWorldInfoDetected') }}
              </div>

              <div v-else-if="info && preferredBookEntries.length === 0" class="mt-3 text-xs text-slate-400">
                {{ t('resources.noEntriesDetected2') }}
              </div>

              <div v-else-if="info" class="mt-3 space-y-2">
                <details
                  v-for="(entry, idx) in preferredBookEntries"
                  :key="idx"
                  class="bg-white border border-slate-200 rounded-xl overflow-hidden"
                >
                  <summary class="px-3 py-2 cursor-pointer select-none flex items-center justify-between gap-3">
                    <div class="min-w-0">
                      <div class="text-xs font-semibold text-slate-800 truncate">
                        #{{ idx + 1 }}
                        <span class="text-slate-500 font-medium">
                          {{ entry.keys.length ? entry.keys.join(', ') : t('resources.noKeys') }}
                        </span>
                      </div>
                      <div class="text-[11px] text-slate-500 truncate">
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
            </div>
          </div>
        </div>

        <div v-if="loading" class="mt-6 flex items-center gap-3 text-slate-500 text-sm font-medium">
          <Loader2 class="w-4 h-4 animate-spin" />
          {{ t('resources.parsingCard') }}
        </div>

        <div v-else-if="errorMsg" class="mt-6 bg-red-50 border border-red-100 rounded-xl p-4 text-sm text-red-600">
          {{ errorMsg }}
        </div>

        <!-- Import Actions -->
        <div
          v-if="characterCardDialogState.isImportMode && !loading && !errorMsg"
          class="shrink-0 mt-6 pt-4 border-t border-slate-100 flex justify-end gap-3"
        >
          <button
            type="button"
            class="px-4 py-2 text-sm font-medium text-slate-600 bg-slate-100 hover:bg-slate-200 rounded-xl transition-colors"
            :disabled="isImporting"
            @click="closeCharacterCardDialog"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            type="button"
            class="px-6 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-xl shadow-sm transition-colors flex items-center gap-2"
            :disabled="isImporting"
            @click="handleImport"
          >
            <Loader2 v-if="isImporting" class="w-4 h-4 animate-spin" />
            {{ isImporting ? t('resources.addingCard') : t('resources.confirmAdd') }}
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
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background-color: #94a3b8;
}
</style>

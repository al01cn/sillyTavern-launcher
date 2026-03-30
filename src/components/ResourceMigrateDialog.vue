<script setup lang="ts">
import { ref, computed, watch, type Component } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import {
  PhX, PhArrowsDownUp, PhWarning, PhCheckCircle,
  PhCircleNotch, PhArrowRight, PhFolder, PhFile, PhCheck,
  PhUserSquare, PhPuzzlePiece, PhChatCircleText, PhBookOpen,
  PhPalette, PhArchiveBox, PhCaretDown, PhCaretUp, PhCrown
} from '@phosphor-icons/vue'

const { t } = useI18n()

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'migrated'): void
}>()

// ── 类型 ──────────────────────────────────────────────────────────────────────
interface MigrationSource {
  tavernPath: string
  dataPath: string
  version: string
  display: string
}

interface ConflictFile {
  relPath: string
  sourceFullPath: string
  destFullPath: string
  sourceDisplay: string
  size: number
  category: string
}

interface ProgressEvent {
  done: number
  total: number
  current: string
  finished: boolean
  error?: string
}

// ── 步骤状态（1=选择实例 2=冲突确认 3=迁移进度）────────────────────────────────
type Step = 1 | 2 | 3

const step = ref<Step>(1)

// ── Step 1：来源列表 ─────────────────────────────────────────────────────────
const sources = ref<MigrationSource[]>([])
const selectedSources = ref<Set<string>>(new Set()) // key = dataPath
const loadingSources = ref(false)
const sourcesError = ref('')

// 可排除的内容分类（与 Rust infer_category 返回值保持一致）
interface ExcludeOption {
  key: string        // 对应 infer_category 返回值
  labelKey: string   // i18n key
  icon: Component  // Phosphor 图标组件
}
const EXCLUDE_OPTIONS: ExcludeOption[] = [
  { key: '角色卡',       labelKey: 'resources.migrate.catCharacters', icon: PhUserSquare },
  { key: '扩展',         labelKey: 'resources.migrate.catExtensions', icon: PhPuzzlePiece },
  { key: '历史聊天记录', labelKey: 'resources.migrate.catChats',      icon: PhChatCircleText },
  { key: '世界书',       labelKey: 'resources.migrate.catWorlds',     icon: PhBookOpen },
  { key: '主题',         labelKey: 'resources.migrate.catThemes',     icon: PhPalette },
  { key: '备份',         labelKey: 'resources.migrate.catBackups',    icon: PhArchiveBox },
]

// 每个来源独立的排除集合：dataPath → Set<categoryKey>
const excludedPerSource = ref<Map<string, Set<string>>>(new Map())
// 哪些来源的排除面板是展开的
const expandedExclude = ref<Set<string>>(new Set())
// 优先级来源：该来源的文件最后执行，确保覆盖其他来源（null = 无特殊优先级）
const prioritySourcePath = ref<string | null>(null)

const getExcludedSet = (dataPath: string): Set<string> => {
  if (!excludedPerSource.value.has(dataPath)) {
    excludedPerSource.value.set(dataPath, new Set())
  }
  return excludedPerSource.value.get(dataPath)!
}

const toggleExclude = (dataPath: string, key: string) => {
  const s = new Set(getExcludedSet(dataPath))
  if (s.has(key)) s.delete(key)
  else s.add(key)
  const m = new Map(excludedPerSource.value)
  m.set(dataPath, s)
  excludedPerSource.value = m
}

const toggleExpandExclude = (dataPath: string) => {
  const s = new Set(expandedExclude.value)
  if (s.has(dataPath)) s.delete(dataPath)
  else s.add(dataPath)
  expandedExclude.value = s
}

const excludedCountForSource = (dataPath: string): number =>
  getExcludedSet(dataPath).size

const loadSources = async () => {
  loadingSources.value = true
  sourcesError.value = ''
  try {
    sources.value = await invoke<MigrationSource[]>('list_resource_migration_sources')
  } catch (e: any) {
    sourcesError.value = String(e?.message || e)
  } finally {
    loadingSources.value = false
  }
}

const toggleSource = (dataPath: string) => {
  const s = new Set(selectedSources.value)
  if (s.has(dataPath)) s.delete(dataPath)
  else s.add(dataPath)
  selectedSources.value = s
}

// ── Step 2：冲突文件 ─────────────────────────────────────────────────────────
const conflicts = ref<ConflictFile[]>([])
const scanningConflicts = ref(false)
// 每个冲突文件用户的决策：'overwrite' | 'skip'（默认 skip）
const conflictDecisions = ref<Record<string, 'overwrite' | 'skip'>>({})

const scanConflicts = async () => {
  scanningConflicts.value = true
  conflicts.value = []
  conflictDecisions.value = {}
  try {
    const selSources = sources.value.filter(s => selectedSources.value.has(s.dataPath))
    const sourcePaths = selSources.map(s => s.dataPath)
    const sourceDisplays = selSources.map(s => s.display)
    // 按来源顺序构建各自的排除列表
    const excludeCategoriesPerSource = selSources.map(s =>
      [...getExcludedSet(s.dataPath)]
    )
    const list = await invoke<ConflictFile[]>('scan_migration_conflicts', {
      sourcePaths,
      sourceDisplays,
      excludeCategoriesPerSource,
      prioritySourcePath: prioritySourcePath.value,
    })
    conflicts.value = list
    // 默认全部跳过
    const decisions: Record<string, 'overwrite' | 'skip'> = {}
    for (const c of list) {
      decisions[c.relPath] = 'skip'
    }
    conflictDecisions.value = decisions
  } catch (e: any) {
    sourcesError.value = String(e?.message || e)
  } finally {
    scanningConflicts.value = false
  }
}

const overwriteCount = computed(() =>
  Object.values(conflictDecisions.value).filter(v => v === 'overwrite').length
)
const skipCount = computed(() =>
  Object.values(conflictDecisions.value).filter(v => v === 'skip').length
)

const setAllDecisions = (dec: 'overwrite' | 'skip') => {
  const d: Record<string, 'overwrite' | 'skip'> = {}
  for (const c of conflicts.value) d[c.relPath] = dec
  conflictDecisions.value = d
}

// ── Step 3：迁移进度 ─────────────────────────────────────────────────────────
const progress = ref<ProgressEvent>({ done: 0, total: 0, current: '', finished: false })
const migrating = ref(false)
const migrationErrors = ref<string[]>([])
let unlistenFn: UnlistenFn | null = null

const progressPercent = computed(() => {
  if (!progress.value.total) return 0
  return Math.round((progress.value.done / progress.value.total) * 100)
})

const startMigration = async () => {
  migrating.value = true
  migrationErrors.value = []
  progress.value = { done: 0, total: 0, current: '', finished: false }
  step.value = 3

  // 监听进度事件
  unlistenFn = await listen<ProgressEvent>('resource-migration-progress', (event) => {
    progress.value = event.payload
    if (event.payload.error) {
      migrationErrors.value.push(event.payload.error)
    }
    if (event.payload.finished) {
      migrating.value = false
      // 1 秒后如果还没手动关闭则自动触发 migrated
      setTimeout(() => {
        if (step.value === 3) {
          emit('migrated')
        }
      }, 1000)
    }
  })

  try {
    const selSources = sources.value.filter(s => selectedSources.value.has(s.dataPath))
    const sourcePaths = selSources.map(s => s.dataPath)
    const sourceDisplays = selSources.map(s => s.display)
    const overwritePaths = Object.entries(conflictDecisions.value)
      .filter(([, v]) => v === 'overwrite')
      .map(([k]) => k)
    const skipPaths = Object.entries(conflictDecisions.value)
      .filter(([, v]) => v === 'skip')
      .map(([k]) => k)
    const excludeCategoriesPerSource = selSources.map(s =>
      [...getExcludedSet(s.dataPath)]
    )

    await invoke('execute_resource_migration', {
      sourcePaths,
      sourceDisplays,
      overwriteRelPaths: overwritePaths,
      skipRelPaths: skipPaths,
      excludeCategoriesPerSource,
      prioritySourcePath: prioritySourcePath.value,
    })
  } catch (e: any) {
    migrationErrors.value.push(String(e?.message || e))
    migrating.value = false
  }
}

// ── 导航 ─────────────────────────────────────────────────────────────────────
const goToStep2 = async () => {
  if (selectedSources.value.size === 0) return
  await scanConflicts()
  step.value = 2
}

const goBack = () => {
  if (step.value === 2) step.value = 1
}

const handleClose = () => {
  if (migrating.value) return
  if (unlistenFn) { unlistenFn(); unlistenFn = null }
  if (step.value === 3 && progress.value.finished) {
    emit('migrated')
  }
  emit('close')
}

// ── 打开时初始化 ──────────────────────────────────────────────────────────────
watch(() => props.open, async (val) => {
  if (val) {
    step.value = 1
    selectedSources.value = new Set()
    excludedPerSource.value = new Map()
    expandedExclude.value = new Set()
    prioritySourcePath.value = null
    conflicts.value = []
    conflictDecisions.value = {}
    migrationErrors.value = []
    progress.value = { done: 0, total: 0, current: '', finished: false }
    await loadSources()
  } else {
    if (unlistenFn) { unlistenFn(); unlistenFn = null }
  }
})

// ── 工具 ─────────────────────────────────────────────────────────────────────
const formatSize = (bytes: number) => {
  if (!bytes) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  let v = bytes, i = 0
  while (v >= 1024 && i < units.length - 1) { v /= 1024; i++ }
  return `${v.toFixed(i === 0 ? 0 : 1)} ${units[i]}`
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="open"
        class="fixed inset-0 z-50 flex items-center justify-center p-4"
        @click.self="handleClose"
      >
        <!-- 背景遮罩 -->
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" @click="handleClose" />

        <!-- 弹窗主体 -->
        <div class="relative bg-white dark:bg-slate-900 rounded-2xl shadow-2xl w-full max-w-2xl max-h-[85vh] flex flex-col overflow-hidden">

          <!-- 头部 -->
          <div class="flex items-center justify-between px-6 py-4 border-b border-slate-100 dark:border-slate-800 shrink-0">
            <div class="flex items-center gap-3">
              <div class="w-8 h-8 rounded-lg bg-violet-100 dark:bg-violet-900/40 flex items-center justify-center">
                <PhArrowsDownUp :size="18" class="text-violet-600 dark:text-violet-400" />
              </div>
              <div>
                <h2 class="text-base font-bold text-slate-900 dark:text-slate-100">
                  {{ t('resources.migrate.title') }}
                </h2>
                <p class="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
                  <template v-if="step === 1">{{ t('resources.migrate.step1Subtitle') }}</template>
                  <template v-else-if="step === 2">{{ t('resources.migrate.step2Subtitle') }}</template>
                  <template v-else>{{ t('resources.migrate.step3Subtitle') }}</template>
                </p>
              </div>
            </div>

            <!-- 步骤指示 -->
            <div class="flex items-center gap-2 mr-8">
              <div v-for="n in 3" :key="n" class="flex items-center gap-1">
                <div
                  class="w-6 h-6 rounded-full text-xs font-bold flex items-center justify-center transition-colors"
                  :class="step === n
                    ? 'bg-violet-600 text-white'
                    : step > n
                      ? 'bg-green-500 text-white'
                      : 'bg-slate-200 dark:bg-slate-700 text-slate-500 dark:text-slate-400'"
                >
                  <PhCheck v-if="step > n" :size="12" />
                  <span v-else>{{ n }}</span>
                </div>
                <PhArrowRight v-if="n < 3" :size="12" class="text-slate-300 dark:text-slate-600" />
              </div>
            </div>

            <button
              v-if="!migrating"
              @click="handleClose"
              class="absolute top-4 right-4 p-1.5 rounded-lg text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
            >
              <PhX :size="18" />
            </button>
          </div>

          <!-- ─── Step 1：选择来源实例 ────────────────────────────────────────── -->
          <div v-if="step === 1" class="flex flex-col flex-1 overflow-hidden">
            <div class="px-6 py-4 flex-1 overflow-y-auto space-y-3">
              <p class="text-sm text-slate-600 dark:text-slate-400">
                {{ t('resources.migrate.step1Desc') }}
              </p>

              <!-- 加载中 -->
              <div v-if="loadingSources" class="flex items-center justify-center py-8 text-slate-400 gap-2">
                <PhCircleNotch :size="20" class="animate-spin" />
                <span class="text-sm">{{ t('common.loading') }}</span>
              </div>

              <!-- 错误 -->
              <div v-else-if="sourcesError" class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl p-4 text-sm text-red-600 dark:text-red-400">
                {{ sourcesError }}
              </div>

              <!-- 空 -->
              <div v-else-if="sources.length === 0" class="bg-slate-50 dark:bg-slate-800 rounded-xl p-8 text-center text-slate-400">
                <PhFolder :size="32" class="mx-auto mb-2 text-slate-300 dark:text-slate-600" />
                <div class="text-sm font-medium">{{ t('resources.migrate.noSources') }}</div>
                <div class="text-xs mt-1">{{ t('resources.migrate.noSourcesDesc') }}</div>
              </div>

              <!-- 实例列表 -->
              <template v-else>
                <div
                  v-for="src in sources"
                  :key="src.dataPath"
                  class="rounded-xl border transition-all select-none overflow-hidden"
                  :class="selectedSources.has(src.dataPath)
                    ? 'border-violet-500 bg-violet-50 dark:bg-violet-900/20'
                    : 'border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800'"
                >
                  <!-- 主行：勾选 + 信息 + 优先级角标 + 展开排除按钮 -->
                  <div
                    class="flex items-center gap-3 p-4 cursor-pointer"
                    @click="toggleSource(src.dataPath)"
                  >
                    <!-- 复选框 -->
                    <div
                      class="w-5 h-5 rounded-md border-2 flex items-center justify-center shrink-0 transition-colors"
                      :class="selectedSources.has(src.dataPath)
                        ? 'border-violet-500 bg-violet-500'
                        : 'border-slate-300 dark:border-slate-600'"
                    >
                      <PhCheck v-if="selectedSources.has(src.dataPath)" :size="12" class="text-white" />
                    </div>

                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-1.5">
                        <span class="text-sm font-semibold text-slate-800 dark:text-slate-100 truncate">
                          {{ src.version || t('resources.migrate.unknownVersion') }}
                        </span>
                        <!-- 优先级皇冠角标 -->
                        <span
                          v-if="prioritySourcePath === src.dataPath"
                          class="inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded text-xs font-bold bg-amber-100 dark:bg-amber-900/40 text-amber-600 dark:text-amber-400 shrink-0"
                        >
                          <PhCrown :size="11" weight="fill" />
                          {{ t('resources.migrate.priorityBadge') }}
                        </span>
                      </div>
                      <div class="text-xs text-slate-500 dark:text-slate-400 truncate mt-0.5">
                        {{ src.dataPath }}
                      </div>
                    </div>

                    <!-- 排除设置按钮 -->
                    <button
                      class="flex items-center gap-1 px-2 py-1 rounded-lg text-xs font-medium transition-colors shrink-0"
                      :class="excludedCountForSource(src.dataPath) > 0
                        ? 'bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-300'
                        : 'bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-slate-600'"
                      @click.stop="toggleExpandExclude(src.dataPath)"
                    >
                      <span v-if="excludedCountForSource(src.dataPath) > 0">
                        {{ t('resources.migrate.excludeCount', { count: excludedCountForSource(src.dataPath) }) }}
                      </span>
                      <span v-else>{{ t('resources.migrate.excludeBtn') }}</span>
                      <PhCaretDown v-if="!expandedExclude.has(src.dataPath)" :size="12" />
                      <PhCaretUp v-else :size="12" />
                    </button>
                  </div>

                  <!-- 展开的设置面板（优先级 + 排除分类） -->
                  <div
                    v-if="expandedExclude.has(src.dataPath)"
                    class="px-4 pb-4 border-t border-slate-100 dark:border-slate-700/60 pt-3 space-y-3"
                  >
                    <!-- 优先级单选 -->
                    <div>
                      <p class="text-xs font-medium text-slate-600 dark:text-slate-300 mb-2 flex items-center gap-1">
                        <PhCrown :size="12" class="text-amber-500" />
                        {{ t('resources.migrate.priorityLabel') }}
                      </p>
                      <div class="flex gap-2">
                        <button
                          @click.stop="prioritySourcePath = prioritySourcePath === src.dataPath ? null : src.dataPath"
                          class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-medium border transition-all"
                          :class="prioritySourcePath === src.dataPath
                            ? 'bg-amber-100 dark:bg-amber-900/30 border-amber-400 dark:border-amber-500 text-amber-700 dark:text-amber-300'
                            : 'bg-white dark:bg-slate-800 border-slate-200 dark:border-slate-600 text-slate-600 dark:text-slate-300 hover:border-amber-300 dark:hover:border-amber-600'"
                        >
                          <PhCrown :size="13" :weight="prioritySourcePath === src.dataPath ? 'fill' : 'regular'" />
                          <span>{{ prioritySourcePath === src.dataPath ? t('resources.migrate.priorityOn') : t('resources.migrate.prioritySet') }}</span>
                        </button>
                      </div>
                      <p class="text-xs text-slate-400 dark:text-slate-500 mt-1.5">
                        {{ t('resources.migrate.priorityDesc') }}
                      </p>
                    </div>

                    <!-- 排除分类 -->
                    <div>
                      <p class="text-xs text-slate-500 dark:text-slate-400 mb-2">
                        {{ t('resources.migrate.excludeLabel') }}
                      </p>
                      <div class="flex flex-wrap gap-2">
                        <button
                          v-for="opt in EXCLUDE_OPTIONS"
                          :key="opt.key"
                          @click.stop="toggleExclude(src.dataPath, opt.key)"
                          class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-medium border transition-all"
                          :class="getExcludedSet(src.dataPath).has(opt.key)
                            ? 'bg-amber-100 dark:bg-amber-900/30 border-amber-400 dark:border-amber-500 text-amber-700 dark:text-amber-300'
                            : 'bg-white dark:bg-slate-800 border-slate-200 dark:border-slate-600 text-slate-600 dark:text-slate-300 hover:border-amber-300 dark:hover:border-amber-600'"
                        >
                          <component :is="opt.icon" :size="13" :weight="getExcludedSet(src.dataPath).has(opt.key) ? 'fill' : 'regular'" />
                          <span>{{ t(opt.labelKey) }}</span>
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              </template>
            </div>

            <!-- 底部按钮 -->
            <div class="px-6 py-4 border-t border-slate-100 dark:border-slate-800 shrink-0 flex justify-end gap-3">
              <button
                @click="handleClose"
                class="px-4 py-2 rounded-lg text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
              >
                {{ t('common.cancel') }}
              </button>
              <button
                @click="goToStep2"
                :disabled="selectedSources.size === 0 || scanningConflicts"
                class="px-4 py-2 rounded-lg text-sm font-bold flex items-center gap-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed bg-violet-600 text-white hover:bg-violet-700 active:bg-violet-800"
              >
                <PhCircleNotch v-if="scanningConflicts" :size="16" class="animate-spin" />
                {{ scanningConflicts ? t('resources.migrate.scanning') : t('resources.migrate.nextStep') }}
                <PhArrowRight v-if="!scanningConflicts" :size="16" />
              </button>
            </div>
          </div>

          <!-- ─── Step 2：冲突确认 ───────────────────────────────────────────── -->
          <div v-else-if="step === 2" class="flex flex-col flex-1 overflow-hidden">
            <div class="px-6 py-4 flex-1 overflow-y-auto space-y-4">

              <!-- 无冲突 -->
              <div v-if="conflicts.length === 0" class="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-xl p-4 flex items-start gap-3">
                <PhCheckCircle :size="20" class="text-green-500 shrink-0 mt-0.5" />
                <div class="text-sm text-green-700 dark:text-green-400">
                  {{ t('resources.migrate.noConflicts') }}
                </div>
              </div>

              <!-- 有冲突 -->
              <template v-else>
                <div class="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-xl p-4 flex items-start gap-3">
                  <PhWarning :size="20" class="text-amber-500 shrink-0 mt-0.5" />
                  <div class="text-sm text-amber-700 dark:text-amber-400">
                    {{ t('resources.migrate.conflictWarning', { count: conflicts.length }) }}
                  </div>
                </div>

                <!-- 全选操作 -->
                <div class="flex items-center gap-3">
                  <span class="text-xs text-slate-500 dark:text-slate-400">{{ t('resources.migrate.batchDecide') }}</span>
                  <button
                    @click="setAllDecisions('overwrite')"
                    class="text-xs px-2 py-1 rounded bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400 hover:bg-red-100 dark:hover:bg-red-900/50 transition-colors font-medium"
                  >
                    {{ t('resources.migrate.overwriteAll') }}
                  </button>
                  <button
                    @click="setAllDecisions('skip')"
                    class="text-xs px-2 py-1 rounded bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-slate-600 transition-colors font-medium"
                  >
                    {{ t('resources.migrate.skipAll') }}
                  </button>
                  <span class="text-xs text-slate-400">
                    {{ t('resources.migrate.decisionSummary', { overwrite: overwriteCount, skip: skipCount }) }}
                  </span>
                </div>

                <!-- 冲突文件列表 -->
                <div class="space-y-2">
                  <div
                    v-for="c in conflicts"
                    :key="c.relPath"
                    class="flex items-start gap-3 p-3 rounded-xl border transition-all"
                    :class="conflictDecisions[c.relPath] === 'overwrite'
                      ? 'border-red-200 dark:border-red-800 bg-red-50/50 dark:bg-red-900/10'
                      : 'border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800'"
                  >
                    <div class="w-8 h-8 rounded-lg bg-slate-100 dark:bg-slate-700 flex items-center justify-center shrink-0 mt-0.5">
                      <PhFile :size="16" class="text-slate-500" />
                    </div>
                    <div class="flex-1 min-w-0">
                      <!-- 分类 + 文件名 -->
                      <div class="flex items-center gap-2 flex-wrap">
                        <span class="text-xs px-1.5 py-0.5 rounded bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 font-medium">
                          {{ c.category }}
                        </span>
                        <span class="text-sm font-medium text-slate-800 dark:text-slate-100 truncate">
                          {{ c.relPath.split('/').pop() || c.relPath }}
                        </span>
                        <span class="text-xs text-slate-400">{{ formatSize(c.size) }}</span>
                      </div>
                      <!-- 来源信息 -->
                      <div class="text-xs text-slate-500 dark:text-slate-400 mt-1 truncate">
                        {{ t('resources.migrate.conflictFrom') }}{{ c.sourceDisplay }}
                      </div>
                    </div>

                    <!-- 决策切换 -->
                    <div class="flex items-center gap-1 shrink-0">
                      <button
                        @click="conflictDecisions[c.relPath] = 'overwrite'"
                        class="text-xs px-2 py-1 rounded font-medium transition-colors"
                        :class="conflictDecisions[c.relPath] === 'overwrite'
                          ? 'bg-red-500 text-white'
                          : 'bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 hover:bg-red-50 dark:hover:bg-red-900/30 hover:text-red-600 dark:hover:text-red-400'"
                      >
                        {{ t('resources.migrate.overwrite') }}
                      </button>
                      <button
                        @click="conflictDecisions[c.relPath] = 'skip'"
                        class="text-xs px-2 py-1 rounded font-medium transition-colors"
                        :class="conflictDecisions[c.relPath] === 'skip'
                          ? 'bg-slate-500 text-white'
                          : 'bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-slate-600'"
                      >
                        {{ t('resources.migrate.skip') }}
                      </button>
                    </div>
                  </div>
                </div>
              </template>

              <!-- settings.json 合并提示 -->
              <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-xl p-3 text-xs text-blue-700 dark:text-blue-400 flex items-start gap-2">
                <PhCheckCircle :size="14" class="shrink-0 mt-0.5" />
                {{ t('resources.migrate.settingsJsonNote') }}
              </div>
            </div>

            <!-- 底部按钮 -->
            <div class="px-6 py-4 border-t border-slate-100 dark:border-slate-800 flex justify-between gap-3 shrink-0">
              <button
                @click="goBack"
                class="px-4 py-2 rounded-lg text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
              >
                {{ t('common.back') }}
              </button>
              <button
                @click="startMigration"
                class="px-4 py-2 rounded-lg text-sm font-bold flex items-center gap-2 transition-colors bg-violet-600 text-white hover:bg-violet-700 active:bg-violet-800"
              >
                <PhArrowsDownUp :size="16" />
                {{ t('resources.migrate.confirmMerge') }}
              </button>
            </div>
          </div>

          <!-- ─── Step 3：迁移进度 ───────────────────────────────────────────── -->
          <div v-else-if="step === 3" class="flex flex-col flex-1 overflow-hidden">
            <div class="px-6 py-6 flex-1 overflow-y-auto space-y-5">

              <!-- 完成状态 -->
              <div v-if="progress.finished" class="text-center space-y-3">
                <div class="w-16 h-16 rounded-full bg-green-100 dark:bg-green-900/40 flex items-center justify-center mx-auto">
                  <PhCheckCircle :size="36" class="text-green-500" />
                </div>
                <p class="text-base font-bold text-slate-800 dark:text-slate-100">
                  {{ t('resources.migrate.mergeSuccess') }}
                </p>
                <p class="text-sm text-slate-500 dark:text-slate-400">
                  {{ t('resources.migrate.mergeSuccessDesc', { done: progress.done }) }}
                </p>
              </div>

              <!-- 进行中 -->
              <template v-else>
                <div class="text-center">
                  <PhCircleNotch :size="36" class="animate-spin text-violet-500 mx-auto mb-3" />
                  <p class="text-sm font-medium text-slate-700 dark:text-slate-300">
                    {{ t('resources.migrate.merging') }}
                  </p>
                </div>
              </template>

              <!-- 进度条 -->
              <div class="space-y-2">
                <div class="flex justify-between text-xs text-slate-500 dark:text-slate-400">
                  <span>{{ progress.done }} / {{ progress.total }} {{ t('resources.migrate.files') }}</span>
                  <span>{{ progressPercent }}%</span>
                </div>
                <div class="h-2 bg-slate-200 dark:bg-slate-700 rounded-full overflow-hidden">
                  <div
                    class="h-full bg-violet-500 rounded-full transition-all duration-300"
                    :style="{ width: progressPercent + '%' }"
                  />
                </div>
                <div v-if="progress.current" class="text-xs text-slate-400 dark:text-slate-500 truncate">
                  {{ progress.current }}
                </div>
              </div>

              <!-- 错误列表 -->
              <div v-if="migrationErrors.length > 0" class="space-y-1 max-h-32 overflow-y-auto">
                <div
                  v-for="(err, i) in migrationErrors"
                  :key="i"
                  class="text-xs text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 rounded-lg px-3 py-1.5"
                >
                  {{ err }}
                </div>
              </div>
            </div>

            <!-- 底部按钮 -->
            <div class="px-6 py-4 border-t border-slate-100 dark:border-slate-800 flex justify-end shrink-0">
              <button
                @click="handleClose"
                :disabled="migrating"
                class="px-4 py-2 rounded-lg text-sm font-bold transition-colors disabled:opacity-50 disabled:cursor-not-allowed bg-violet-600 text-white hover:bg-violet-700 active:bg-violet-800"
              >
                {{ progress.finished ? t('resources.migrate.close') : t('common.cancel') }}
              </button>
            </div>
          </div>

        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.2s;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
</style>

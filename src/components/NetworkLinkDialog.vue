<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { openUrl } from '@tauri-apps/plugin-opener'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { toast } from 'vue-sonner'
import QRCode from 'qrcode'
import {
  PhX,
  PhCopy,
  PhGlobe,
  PhWifiHigh,
  PhArrowClockwise,
  PhSpinner,
  PhCheckCircle,
  PhWarning,
} from '@phosphor-icons/vue'
import { publicSessionId } from '../lib/consoleState'

const { t } = useI18n()

// ─── 可用性检测结果缓存（模块级，按 publicSessionId 缓存）─────────────────────
interface AvailabilityCache {
  sessionId: number
  noChrome: boolean
  itdogUrlV4: string
  itdogUrlV6: string
  ipv4Total: number
  ipv4Timeout: number
  ipv6Total: number
  ipv6Timeout: number
  preferred: 'ipv4' | 'ipv6' | null
  statusText: string
}
let availabilityCache: AvailabilityCache | null = null

// ─── Props ──────────────────────────────────────────────────────────────────
const props = defineProps<{
  open: boolean
  mode: 'lan' | 'public' // 局域网 or 公网
  port: number
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

// ─── State ──────────────────────────────────────────────────────────────────
const ipv4 = ref('')
const ipv6 = ref('')
const loading = ref(false)

// 可用性检测相关
const checkingAvailability = ref(false)
const preferred = ref<'ipv4' | 'ipv6' | null>(null)
const ipv4Total = ref(0)
const ipv4Timeout = ref(0)
const ipv6Total = ref(0)
const ipv6Timeout = ref(0)
const availabilityChecked = ref(false)
// 无 Chrome 模式：直接提供 itdog 链接
const noChrome = ref(false)
const itdogUrlV4 = ref('')
const itdogUrlV6 = ref('')
// 实时检测状态文字（给用户看的进度提示）
const checkStatusText = ref('')

const qrCanvas4 = ref<HTMLCanvasElement | null>(null)
const qrCanvas6 = ref<HTMLCanvasElement | null>(null)

// ─── 监听 itdog 检测进度事件 ──────────────────────────────────────────────────
let unlistenProgress: (() => void) | null = null
let waitingTimer: ReturnType<typeof setInterval> | null = null
let waitingProto = ''
let waitingSeconds = 0

const clearWaitingTimer = () => {
  if (waitingTimer) {
    clearInterval(waitingTimer)
    waitingTimer = null
  }
}

const setupProgressListener = async () => {
  if (unlistenProgress) return
  unlistenProgress = await listen<{
    phase: string
    proto?: string
    ip?: string
    total?: number
    timeout?: number
    message?: string
  }>('itdog-check-progress', event => {
    const { phase, proto, ip, total, timeout, message } = event.payload
    switch (phase) {
      case 'start':
        clearWaitingTimer()
        checkStatusText.value = `正在打开 ${proto} 检测页面... (${ip})`
        break
      case 'injecting':
        clearWaitingTimer()
        checkStatusText.value = `${proto} 检测中，正在注入脚本...`
        break
      case 'waiting':
        clearWaitingTimer()
        waitingProto = proto ?? ''
        waitingSeconds = 0
        checkStatusText.value = `${waitingProto} 检测中，等待 itdog 节点返回数据...`
        waitingTimer = setInterval(() => {
          waitingSeconds += 10
          checkStatusText.value = `${waitingProto} 检测中... (已等待 ${waitingSeconds}s，约需 10s)`
        }, 10000)
        break
      case 'done': {
        clearWaitingTimer()
        const rate = total && total > 0 ? Math.round(((timeout ?? 0) / total) * 100) : 0
        if (rate === 0 && total && total > 0) {
          checkStatusText.value = `${proto} 检测完成：${total} 个节点，可用率 100%`
        } else {
          checkStatusText.value = `${proto} 检测完成：${total} 个节点，超时率 ${rate}%`
        }
        break
      }
      case 'timeout':
        clearWaitingTimer()
        checkStatusText.value = `${proto} 检测超时，未获取到数据`
        break
      case 'error':
        clearWaitingTimer()
        checkStatusText.value = `${proto} 检测出错：${message ?? '未知错误'}`
        break
    }
  })
}

onMounted(() => {
  setupProgressListener()
})

onUnmounted(() => {
  clearWaitingTimer()
  unlistenProgress?.()
  unlistenProgress = null
})

// ─── Helpers ─────────────────────────────────────────────────────────────────
const buildUrl = (host: string, port: number) => {
  return `http://${host}:${port}`
}

const renderQr = async (canvas: HTMLCanvasElement | null, url: string) => {
  if (!canvas || !url) return
  try {
    await QRCode.toCanvas(canvas, url, {
      width: 160,
      margin: 2,
      color: { dark: '#1e293b', light: '#f8fafc' },
      errorCorrectionLevel: 'M',
    })
  } catch (e) {
    console.error('QR render error', e)
  }
}

// 超时率 → 百分比文字
const timeoutRateText = (total: number, timeout: number) => {
  if (total === 0) return '—'
  const rate = Math.round((timeout / total) * 100)
  if (rate === 0) return '100%' // 0 超时 = 100% 可用
  return `${rate}%`
}

// 超时率 → 颜色 class
const timeoutRateColor = (total: number, timeout: number) => {
  if (total === 0) return 'text-slate-400'
  const rate = timeout / total
  if (rate <= 0.1) return 'text-emerald-500'
  if (rate <= 0.3) return 'text-amber-500'
  return 'text-red-500'
}

// ─── 获取 IP ─────────────────────────────────────────────────────────────────
const fetchIps = async () => {
  loading.value = true
  ipv4.value = ''
  ipv6.value = ''
  preferred.value = null
  ipv4Total.value = 0
  ipv4Timeout.value = 0
  ipv6Total.value = 0
  ipv6Timeout.value = 0
  availabilityChecked.value = false
  noChrome.value = false
  itdogUrlV4.value = ''
  itdogUrlV6.value = ''

  try {
    if (props.mode === 'lan') {
      const res = await invoke<{ ipv4: string[]; ipv6: string[] }>('get_local_ip_addresses')
      ipv4.value = res.ipv4?.[0] ?? ''
      ipv6.value = res.ipv6?.[0] ?? ''
    } else {
      const res = await invoke<{ ipv4: string | null; ipv6: string | null }>('get_public_ip_addresses')
      ipv4.value = res.ipv4 ?? ''
      ipv6.value = res.ipv6 ?? ''
    }
  } catch (e) {
    console.error('fetchIps error', e)
  } finally {
    loading.value = false
    await nextTick()
    if (ipv4.value) {
      await renderQr(qrCanvas4.value, buildUrl(ipv4.value, props.port))
    }
    if (ipv6.value) {
      await renderQr(qrCanvas6.value, buildUrl(ipv6.value, props.port))
    }
  }

  // 公网模式下，获取 IP 后异步启动可用性检测
  if (props.mode === 'public' && (ipv4.value || ipv6.value)) {
    startAvailabilityCheck()
  }
}

// ─── 可用性检测（itdog TCPing）───────────────────────────────────────────────
const startAvailabilityCheck = async () => {
  // 命中缓存：同一次公网服务启动内，只检测一次
  const sessionId = publicSessionId.value
  if (availabilityCache && availabilityCache.sessionId === sessionId) {
    noChrome.value = availabilityCache.noChrome
    itdogUrlV4.value = availabilityCache.itdogUrlV4
    itdogUrlV6.value = availabilityCache.itdogUrlV6
    ipv4Total.value = availabilityCache.ipv4Total
    ipv4Timeout.value = availabilityCache.ipv4Timeout
    ipv6Total.value = availabilityCache.ipv6Total
    ipv6Timeout.value = availabilityCache.ipv6Timeout
    preferred.value = availabilityCache.preferred
    availabilityChecked.value = !availabilityCache.noChrome
    checkStatusText.value = availabilityCache.statusText
    return
  }

  checkingAvailability.value = true
  availabilityChecked.value = false
  noChrome.value = false
  checkStatusText.value = '准备开始检测...'
  try {
    // ipv6 传裸 IP（去掉方括号），Rust 端负责格式化
    const v4host = ipv4.value || undefined
    const v6host = ipv6.value ? ipv6.value.replace(/^\[|\]$/g, '') : undefined

    const res = await invoke<{
      no_chrome: boolean
      itdog_url_v4?: string | null
      itdog_url_v6?: string | null
      preferred?: string | null
      ipv4_total?: number
      ipv4_timeout?: number
      ipv4_timeout_rate?: number
      ipv6_total?: number
      ipv6_timeout?: number
      ipv6_timeout_rate?: number
    }>('check_network_availability', {
      ipv4Host: v4host ?? null,
      ipv6Host: v6host ?? null,
      port: props.port,
    })

    if (res.no_chrome) {
      // 没有 Chrome，提供 itdog 链接让用户自己看
      noChrome.value = true
      itdogUrlV4.value = res.itdog_url_v4 ?? ''
      itdogUrlV6.value = res.itdog_url_v6 ?? ''
      checkStatusText.value = '未检测到 Chrome，请点击链接在浏览器中查看'
      // 写缓存
      availabilityCache = {
        sessionId,
        noChrome: true,
        itdogUrlV4: itdogUrlV4.value,
        itdogUrlV6: itdogUrlV6.value,
        ipv4Total: 0,
        ipv4Timeout: 0,
        ipv6Total: 0,
        ipv6Timeout: 0,
        preferred: null,
        statusText: checkStatusText.value,
      }
    } else {
      ipv4Total.value = res.ipv4_total ?? 0
      ipv4Timeout.value = res.ipv4_timeout ?? 0
      ipv6Total.value = res.ipv6_total ?? 0
      ipv6Timeout.value = res.ipv6_timeout ?? 0

      if (res.preferred === 'ipv4' || res.preferred === 'ipv6') {
        preferred.value = res.preferred
      }
      availabilityChecked.value = true
      // 写缓存（checkStatusText 此时已由 itdog-check-progress 事件更新为完成文字）
      availabilityCache = {
        sessionId,
        noChrome: false,
        itdogUrlV4: '',
        itdogUrlV6: '',
        ipv4Total: ipv4Total.value,
        ipv4Timeout: ipv4Timeout.value,
        ipv6Total: ipv6Total.value,
        ipv6Timeout: ipv6Timeout.value,
        preferred: preferred.value,
        statusText: checkStatusText.value,
      }
    }
  } catch (e) {
    console.error('checkAvailability error', e)
  } finally {
    checkingAvailability.value = false
  }
}

// ─── Watchers ────────────────────────────────────────────────────────────────
watch(
  () => props.open,
  val => {
    if (val) {
      fetchIps()
    }
  },
)

onMounted(() => {
  if (props.open) fetchIps()
})

// ─── Actions ─────────────────────────────────────────────────────────────────
const copyUrl = async (host: string) => {
  if (!host) return
  const url = buildUrl(host, props.port)
  try {
    await writeText(url)
    toast.success(t('networkLink.copySuccess'))
  } catch {
    toast.error(t('networkLink.copyFail'))
  }
}

const openBrowser = (host: string) => {
  if (!host) return
  openUrl(buildUrl(host, props.port)).catch(() => {})
}

const handleRefresh = () => {
  // 清缓存，强制重新检测
  availabilityCache = null
  fetchIps()
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-[990] flex items-center justify-center p-4">
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" @click="emit('close')"></div>

      <!-- Dialog -->
      <div
        class="relative bg-white dark:bg-slate-800 rounded-2xl shadow-2xl w-full max-w-lg border border-slate-200 dark:border-slate-700 animate-in zoom-in-95 duration-200 overflow-hidden"
      >
        <!-- Header -->
        <div class="flex items-center justify-between px-6 py-4 border-b border-slate-100 dark:border-slate-700">
          <div class="flex items-center gap-3">
            <div
              :class="[
                'w-9 h-9 rounded-xl flex items-center justify-center',
                mode === 'lan'
                  ? 'bg-emerald-100 dark:bg-emerald-900/40 text-emerald-600 dark:text-emerald-400'
                  : 'bg-red-100 dark:bg-red-900/40 text-red-600 dark:text-red-400',
              ]"
            >
              <PhWifiHigh v-if="mode === 'lan'" :size="20" weight="duotone" />
              <PhGlobe v-else :size="20" weight="duotone" />
            </div>
            <div>
              <h3 class="font-bold text-slate-900 dark:text-slate-100 text-base">
                {{ mode === 'lan' ? t('networkLink.lanTitle') : t('networkLink.publicTitle') }}
              </h3>
              <p class="text-xs text-slate-400 dark:text-slate-500">Port {{ port }}</p>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <button
              :disabled="loading || checkingAvailability"
              class="w-8 h-8 rounded-lg flex items-center justify-center text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all disabled:opacity-50 active:scale-95"
              @click="handleRefresh"
            >
              <PhArrowClockwise :size="16" :class="loading || checkingAvailability ? 'animate-spin' : ''" />
            </button>
            <button
              class="w-8 h-8 rounded-lg flex items-center justify-center text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all active:scale-95"
              @click="emit('close')"
            >
              <PhX :size="18" weight="bold" />
            </button>
          </div>
        </div>

        <!-- Body -->
        <div class="p-6">
          <!-- 加载中 -->
          <div v-if="loading" class="flex flex-col items-center justify-center py-10 gap-3">
            <div class="w-10 h-10 rounded-full border-2 border-t-transparent border-blue-500 animate-spin"></div>
            <p class="text-sm text-slate-400">{{ t('networkLink.fetchingIp') }}</p>
          </div>

          <!-- QR + 地址展示 -->
          <div v-else class="grid grid-cols-2 gap-6 items-stretch">
            <!-- IPv4 -->
            <div
              :class="[
                'flex flex-col items-center gap-3 rounded-xl p-2 -m-2 transition-all',
                mode === 'public' && preferred === 'ipv4'
                  ? 'ring-2 ring-blue-400/60 bg-blue-50/30 dark:bg-blue-900/10'
                  : '',
              ]"
            >
              <div class="flex items-center gap-1.5">
                <p class="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
                  {{ t('networkLink.ipv4Label') }}
                </p>
                <!-- 推荐徽章 -->
                <span
                  v-if="mode === 'public' && preferred === 'ipv4' && availabilityChecked"
                  class="px-1.5 py-0.5 rounded-full bg-blue-500 text-white text-[9px] font-bold leading-none"
                >
                  {{ t('networkLink.recommended') }}
                </span>
              </div>
              <div
                v-if="ipv4"
                :class="[
                  'rounded-xl p-3 border',
                  mode === 'public' && preferred === 'ipv4'
                    ? 'bg-white dark:bg-slate-900 border-blue-300 dark:border-blue-700'
                    : 'bg-slate-50 dark:bg-slate-900 border-slate-200 dark:border-slate-700',
                ]"
              >
                <canvas ref="qrCanvas4" class="rounded-lg"></canvas>
              </div>
              <div
                v-else
                class="w-[160px] h-[160px] bg-slate-50 dark:bg-slate-900 rounded-xl border border-dashed border-slate-300 dark:border-slate-600 flex items-center justify-center"
              >
                <span class="text-xs text-slate-400">{{ t('networkLink.ipUnavailable') }}</span>
              </div>
              <div v-if="ipv4" class="w-full space-y-1.5 flex flex-col flex-1 justify-end">
                <div
                  class="px-3 py-2 bg-slate-50 dark:bg-slate-900 rounded-lg border border-slate-200 dark:border-slate-700 text-xs text-slate-600 dark:text-slate-300 font-mono text-center break-all select-all min-h-[2.75rem] flex items-center justify-center"
                >
                  {{ buildUrl(ipv4, port) }}
                </div>
                <!-- 可用性检测结果 (仅公网模式) -->
                <div v-if="mode === 'public'" class="flex items-center justify-center gap-1 text-[10px]">
                  <template v-if="checkingAvailability && !availabilityChecked">
                    <PhSpinner :size="10" class="animate-spin text-slate-400" />
                    <span class="text-slate-400">{{ t('networkLink.checking') }}</span>
                  </template>
                  <template v-else-if="availabilityChecked && ipv4Total > 0">
                    <PhCheckCircle v-if="ipv4Timeout / ipv4Total <= 0.1" :size="10" class="text-emerald-500" />
                    <PhWarning v-else :size="10" class="text-amber-500" />
                    <span :class="timeoutRateColor(ipv4Total, ipv4Timeout)">
                      {{ ipv4Timeout === 0 ? t('networkLink.availabilityRate') : t('networkLink.timeoutRate') }}
                      {{ timeoutRateText(ipv4Total, ipv4Timeout) }}
                    </span>
                    <span class="text-slate-400">({{ ipv4Total }} {{ t('networkLink.nodes') }})</span>
                  </template>
                </div>
                <div class="flex flex-col gap-1.5">
                  <button
                    class="w-full inline-flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 text-xs font-medium hover:bg-slate-200 dark:hover:bg-slate-600 transition-all active:scale-95 leading-none"
                    @click="copyUrl(ipv4)"
                  >
                    <PhCopy :size="13" class="shrink-0" />
                    {{ t('common.copy') }}
                  </button>
                  <button
                    class="w-full inline-flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 text-xs font-medium hover:bg-slate-200 dark:hover:bg-slate-600 transition-all active:scale-95 leading-none"
                    @click="openBrowser(ipv4)"
                  >
                    <PhGlobe :size="13" class="shrink-0" />
                    {{ t('networkLink.openInBrowser') }}
                  </button>
                </div>
              </div>
            </div>

            <!-- IPv6 -->
            <div
              :class="[
                'flex flex-col items-center gap-3 rounded-xl p-2 -m-2 transition-all',
                mode === 'public' && preferred === 'ipv6'
                  ? 'ring-2 ring-blue-400/60 bg-blue-50/30 dark:bg-blue-900/10'
                  : '',
              ]"
            >
              <div class="flex items-center gap-1.5">
                <p class="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
                  {{ t('networkLink.ipv6Label') }}
                </p>
                <span
                  v-if="mode === 'public' && preferred === 'ipv6' && availabilityChecked"
                  class="px-1.5 py-0.5 rounded-full bg-blue-500 text-white text-[9px] font-bold leading-none"
                >
                  {{ t('networkLink.recommended') }}
                </span>
              </div>
              <div
                v-if="ipv6"
                :class="[
                  'rounded-xl p-3 border',
                  mode === 'public' && preferred === 'ipv6'
                    ? 'bg-white dark:bg-slate-900 border-blue-300 dark:border-blue-700'
                    : 'bg-slate-50 dark:bg-slate-900 border-slate-200 dark:border-slate-700',
                ]"
              >
                <canvas ref="qrCanvas6" class="rounded-lg"></canvas>
              </div>
              <div
                v-else
                class="w-[160px] h-[160px] bg-slate-50 dark:bg-slate-900 rounded-xl border border-dashed border-slate-300 dark:border-slate-600 flex items-center justify-center"
              >
                <span class="text-xs text-slate-400">{{ t('networkLink.ipUnavailable') }}</span>
              </div>
              <div v-if="ipv6" class="w-full space-y-1.5 flex flex-col flex-1 justify-end">
                <div
                  class="px-3 py-2 bg-slate-50 dark:bg-slate-900 rounded-lg border border-slate-200 dark:border-slate-700 text-xs text-slate-600 dark:text-slate-300 font-mono text-center break-all select-all min-h-[2.75rem] flex items-center justify-center"
                >
                  {{ buildUrl(ipv6, port) }}
                </div>
                <!-- 可用性检测结果 (仅公网模式) -->
                <div v-if="mode === 'public'" class="flex items-center justify-center gap-1 text-[10px]">
                  <template v-if="checkingAvailability && !availabilityChecked">
                    <PhSpinner :size="10" class="animate-spin text-slate-400" />
                    <span class="text-slate-400">{{ t('networkLink.checking') }}</span>
                  </template>
                  <template v-else-if="availabilityChecked && ipv6Total > 0">
                    <PhCheckCircle v-if="ipv6Timeout / ipv6Total <= 0.1" :size="10" class="text-emerald-500" />
                    <PhWarning v-else :size="10" class="text-amber-500" />
                    <span :class="timeoutRateColor(ipv6Total, ipv6Timeout)">
                      {{ ipv6Timeout === 0 ? t('networkLink.availabilityRate') : t('networkLink.timeoutRate') }}
                      {{ timeoutRateText(ipv6Total, ipv6Timeout) }}
                    </span>
                    <span class="text-slate-400">({{ ipv6Total }} {{ t('networkLink.nodes') }})</span>
                  </template>
                </div>
                <div class="flex flex-col gap-1.5">
                  <button
                    class="w-full inline-flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 text-xs font-medium hover:bg-slate-200 dark:hover:bg-slate-600 transition-all active:scale-95 leading-none"
                    @click="copyUrl(ipv6)"
                  >
                    <PhCopy :size="13" class="shrink-0" />
                    {{ t('common.copy') }}
                  </button>
                  <button
                    class="w-full inline-flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 text-xs font-medium hover:bg-slate-200 dark:hover:bg-slate-600 transition-all active:scale-95 leading-none"
                    @click="openBrowser(ipv6)"
                  >
                    <PhGlobe :size="13" class="shrink-0" />
                    {{ t('networkLink.openInBrowser') }}
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- 扫码提示 / 检测进度 / no_chrome 提示 -->
          <div class="mt-4 text-center">
            <!-- no_chrome 模式：引导用户打开 itdog -->
            <div v-if="mode === 'public' && noChrome" class="space-y-2">
              <p
                class="text-xs text-amber-500 dark:text-amber-400 font-medium flex items-center justify-center gap-1.5"
              >
                <PhWarning :size="13" class="shrink-0" />
                未检测到 Chrome，无法自动检测可用性
              </p>
              <div class="flex items-center justify-center gap-2 flex-wrap">
                <button
                  v-if="itdogUrlV4"
                  class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 text-xs font-medium hover:bg-blue-200 dark:hover:bg-blue-800/40 transition-all active:scale-95"
                  @click="openUrl(itdogUrlV4).catch(() => {})"
                >
                  <PhGlobe :size="12" class="shrink-0" />
                  在浏览器查看 IPv4
                </button>
                <button
                  v-if="itdogUrlV6"
                  class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 text-xs font-medium hover:bg-blue-200 dark:hover:bg-blue-800/40 transition-all active:scale-95"
                  @click="openUrl(itdogUrlV6).catch(() => {})"
                >
                  <PhGlobe :size="12" class="shrink-0" />
                  在浏览器查看 IPv6
                </button>
              </div>
            </div>
            <!-- 检测进行中 -->
            <p
              v-else-if="mode === 'public' && checkingAvailability"
              class="text-xs text-slate-400 dark:text-slate-500 flex items-center justify-center gap-1.5"
            >
              <PhSpinner :size="12" class="animate-spin shrink-0" />
              <span class="truncate max-w-xs" :title="checkStatusText || t('networkLink.checkingAvailability')">
                {{ checkStatusText || t('networkLink.checkingAvailability') }}
              </span>
            </p>
            <!-- 默认提示 -->
            <p v-else class="text-xs text-slate-400 dark:text-slate-500">
              {{ t('networkLink.scanQrCode') }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

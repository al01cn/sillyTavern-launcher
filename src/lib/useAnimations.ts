import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// 动画状态接口
export interface AnimationState {
  enabled: boolean
  isLowPerformance: boolean
  hasShownPerformanceTip: boolean
}

// 全局动画状态
const animationState = ref<AnimationState>({
  enabled: true,
  isLowPerformance: false,
  hasShownPerformanceTip: false
})

// 是否已经初始化
let initialized = false

/**
 * 检测是否为低配设备
 * 通过检测内存和CPU核心数来判断
 */
function detectLowPerformanceDevice(): boolean {
  const memory = (navigator as any).deviceMemory
  const cpuCores = navigator.hardwareConcurrency

  const isLowMemory = memory !== undefined && memory < 4
  const isLowCpu = cpuCores !== undefined && cpuCores <= 2

  return isLowMemory || isLowCpu
}

/**
 * 初始化动画状态
 * 从后端配置和 localStorage 加载设置
 */
async function initAnimationState(): Promise<void> {
  if (initialized) return

  animationState.value.isLowPerformance = detectLowPerformanceDevice()

  const hasShownTip = localStorage.getItem('app_performance_tip_shown')
  animationState.value.hasShownPerformanceTip = hasShownTip === 'true'

  try {
    const config: any = await invoke('get_app_config')
    if (config.enableAnimations !== undefined) {
      animationState.value.enabled = config.enableAnimations
    }
  } catch (e) {
    console.log('Failed to load animation config from backend, using default')
  }

  if (animationState.value.isLowPerformance && !animationState.value.hasShownPerformanceTip) {
    localStorage.setItem('app_is_low_performance', 'true')
  }

  initialized = true
}

/**
 * 标记性能提示已显示
 */
function markPerformanceTipShown(): void {
  animationState.value.hasShownPerformanceTip = true
  localStorage.setItem('app_performance_tip_shown', 'true')
}

/**
 * 设置动画开关状态
 */
function setAnimationsEnabled(enabled: boolean): void {
  animationState.value.enabled = enabled
}

// composable 主函数
export function useAnimations() {
  onMounted(() => {
    initAnimationState()
  })

  return {
    animationsEnabled: computed(() => animationState.value.enabled),
    isLowPerformanceDevice: computed(() => animationState.value.isLowPerformance),
    shouldShowPerformanceTip: computed(() =>
      animationState.value.isLowPerformance && !animationState.value.hasShownPerformanceTip
    ),
    animationState: computed(() => animationState.value),
    initAnimationState,
    setAnimationsEnabled,
    markPerformanceTipShown,
  }
}

export default useAnimations

<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps({
  text: {
    type: String,
    required: true,
  },
  placement: {
    type: String as () => 'top' | 'bottom' | 'left' | 'right',
    default: 'top',
  },
  delay: {
    type: Number,
    default: 80,
  },
})

const visible = ref(false)
const timer = ref<number | null>(null)
const triggerRef = ref<HTMLElement | null>(null)

// fixed 定位坐标
const x = ref(0)
const y = ref(0)
// 由 placement 决定的 translate 字符串
const transformVal = ref('')

const OFFSET = 8

const updatePosition = () => {
  if (!triggerRef.value) return
  const rect = triggerRef.value.getBoundingClientRect()

  switch (props.placement) {
    case 'bottom':
      x.value = rect.left + rect.width / 2
      y.value = rect.bottom + OFFSET
      transformVal.value = 'translateX(-50%)'
      break
    case 'left':
      x.value = rect.left - OFFSET
      y.value = rect.top + rect.height / 2
      transformVal.value = 'translate(-100%, -50%)'
      break
    case 'right':
      x.value = rect.right + OFFSET
      y.value = rect.top + rect.height / 2
      transformVal.value = 'translateY(-50%)'
      break
    case 'top':
    default:
      x.value = rect.left + rect.width / 2
      y.value = rect.top - OFFSET
      transformVal.value = 'translate(-50%, -100%)'
      break
  }
}

const show = () => {
  if (timer.value) window.clearTimeout(timer.value)
  updatePosition()
  timer.value = window.setTimeout(() => {
    visible.value = true
  }, props.delay)
}

const hide = () => {
  if (timer.value) window.clearTimeout(timer.value)
  timer.value = null
  visible.value = false
}
</script>

<template>
  <div ref="triggerRef" class="inline-flex" @mouseenter="show" @mouseleave="hide" @focusin="show" @focusout="hide">
    <slot />
  </div>
  <Teleport to="body">
    <Transition name="tooltip-fade">
      <div
        v-if="visible && text"
        class="pointer-events-none fixed z-[9999] whitespace-nowrap"
        :style="{ top: y + 'px', left: x + 'px', transform: transformVal }"
      >
        <div
          class="px-2.5 py-1 rounded-xl text-[11px] font-medium shadow-lg shadow-black/20 bg-slate-900/90 text-white border border-white/10 backdrop-blur-sm"
        >
          {{ text }}
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.tooltip-fade-enter-active,
.tooltip-fade-leave-active {
  transition: opacity 0.12s ease;
}

.tooltip-fade-enter-from,
.tooltip-fade-leave-to {
  opacity: 0;
}
</style>

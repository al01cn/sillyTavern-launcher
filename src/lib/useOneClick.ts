import { reactive } from 'vue'
import i18n from '../lang'

export interface OneClickState {
  isActive: boolean
  step: 'node' | 'tavern' | 'start' | 'none'
  message: string
}

export const oneClickState = reactive<OneClickState>({
  isActive: false,
  step: 'none',
  message: '',
})

export const startOneClickSetup = (initialMessage?: string) => {
  oneClickState.isActive = true
  oneClickState.step = 'node'
  oneClickState.message = initialMessage || i18n.global.t('oneClick.gitDetecting')
}

export const updateOneClickMessage = (message: string) => {
  if (oneClickState.isActive) {
    oneClickState.message = message
  }
}

export const finishOneClickSetup = () => {
  oneClickState.isActive = false
  oneClickState.step = 'none'
  oneClickState.message = ''
}

export const simulateClickEffect = (elementId: string) => {
  const el = document.getElementById(elementId)
  if (!el) return

  el.style.transition = 'all 0.15s ease-out'
  el.classList.add('scale-90', 'opacity-70', 'ring-4', 'ring-blue-400/50')

  setTimeout(() => {
    el.classList.remove('scale-90', 'opacity-70', 'ring-4', 'ring-blue-400/50')
  }, 250)
}

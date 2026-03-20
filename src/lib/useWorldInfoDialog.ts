import { reactive } from 'vue'

interface WorldInfoDialogState {
  visible: boolean
  fileName: string
}

export const worldInfoDialogState = reactive<WorldInfoDialogState>({
  visible: false,
  fileName: ''
})

export const openWorldInfoDialog = (fileName: string) => {
  worldInfoDialogState.fileName = fileName
  worldInfoDialogState.visible = true
}

export const closeWorldInfoDialog = () => {
  worldInfoDialogState.visible = false
  worldInfoDialogState.fileName = ''
}

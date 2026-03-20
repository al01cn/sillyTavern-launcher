import { reactive } from 'vue'

export const characterCardDialogState = reactive({
  show: false,
  fileName: '',
})

export const openCharacterCardDialog = (fileName: string) => {
  characterCardDialogState.fileName = fileName
  characterCardDialogState.show = true
}

export const closeCharacterCardDialog = () => {
  characterCardDialogState.show = false
}


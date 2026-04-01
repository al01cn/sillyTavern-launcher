import { reactive } from 'vue'

export const characterCardDialogState = reactive({
  show: false,
  fileName: '',
  isImportMode: false,
  importSourcePath: '',
})

export const openCharacterCardDialog = (fileName: string) => {
  characterCardDialogState.fileName = fileName
  characterCardDialogState.isImportMode = false
  characterCardDialogState.importSourcePath = ''
  characterCardDialogState.show = true
}

export const openImportCharacterCardDialog = (sourcePath: string, fileName: string) => {
  characterCardDialogState.fileName = fileName
  characterCardDialogState.isImportMode = true
  characterCardDialogState.importSourcePath = sourcePath
  characterCardDialogState.show = true
}

export const closeCharacterCardDialog = () => {
  characterCardDialogState.show = false
}

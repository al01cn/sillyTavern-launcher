import { reactive } from 'vue'

export const chatDialogState = reactive({
  show: false,
  charName: '',
  charFolder: '',
  fileName: '',
})

export function openChatDialog(opts: { charName: string; charFolder: string; fileName: string }) {
  chatDialogState.charName = opts.charName
  chatDialogState.charFolder = opts.charFolder
  chatDialogState.fileName = opts.fileName
  chatDialogState.show = true
}

export function closeChatDialog() {
  chatDialogState.show = false
}

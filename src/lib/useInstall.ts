import { reactive } from 'vue'

export interface InstallState {
  show: boolean
  version: string
  status: 'idle' | 'downloading' | 'extracting' | 'installing' | 'done' | 'error' | 'deleting'
  progress: number
  logs: string[]
  operation: 'install' | 'delete' | 'unbind'
  isCanceling?: boolean
}

export const installState = reactive<InstallState>({
  show: false,
  version: '',
  status: 'idle',
  progress: 0,
  logs: [],
  operation: 'install',
  isCanceling: false,
})

export const resetInstallState = () => {
  installState.show = false
  installState.version = ''
  installState.status = 'idle'
  installState.progress = 0
  installState.logs = []
  installState.operation = 'install'
  installState.isCanceling = false
}

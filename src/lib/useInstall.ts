import { reactive } from 'vue'

export interface InstallState {
    show: boolean
    version: string
    status: 'downloading' | 'extracting' | 'installing' | 'done' | 'error'
    progress: number
    logs: string[]
}

export const installState = reactive<InstallState>({
    show: false,
    version: '',
    status: 'downloading',
    progress: 0,
    logs: []
})

export const resetInstallState = () => {
    installState.show = false
    installState.version = ''
    installState.status = 'downloading'
    installState.progress = 0
    installState.logs = []
}

import { reactive } from 'vue'

export const typeConfig = {
    success: { bg: 'bg-emerald-500', lightBg: 'bg-emerald-50', text: 'text-emerald-500' },
    warning: { bg: 'bg-amber-500', lightBg: 'bg-amber-50', text: 'text-amber-500' },
    error: { bg: 'bg-red-500', lightBg: 'bg-red-50', text: 'text-red-500' },
    info: { bg: 'bg-blue-500', lightBg: 'bg-blue-50', text: 'text-blue-500' },
    loading: { bg: 'bg-slate-800', lightBg: 'bg-slate-50', text: 'text-slate-500' }
}

interface DialogOptions {
    title?: string
    msg?: string
    context?: string
    message?: string
    closeOnMask?: boolean
    confirmText?: string
    cancelText?: string
    onConfirm?: () => void
    onCancel?: () => void
}

export const dialogState = reactive({
    show: false,
    type: 'info' as keyof typeof typeConfig,
    title: '',
    msg: '',
    closeOnMask: true,
    confirmText: '确定',
    cancelText: '取消',
    // 存储回调函数
    onConfirm: undefined as (() => void) | undefined,
    onCancel: undefined as (() => void) | undefined
})

const openDialog = (type: keyof typeof typeConfig, options: DialogOptions) => {
    dialogState.type = type
    dialogState.title = options.title || '提示'
    dialogState.msg = options.msg || '' 
    dialogState.msg = options.context || ''
    dialogState.msg = options.message || ''
    dialogState.closeOnMask = options.closeOnMask ?? true
    dialogState.confirmText = options.confirmText || '确定'
    dialogState.cancelText = options.cancelText || '取消'

    // 绑定传入的函数
    dialogState.onConfirm = options.onConfirm
    dialogState.onCancel = options.onCancel

    dialogState.show = true
}

export const Dialog = {
    success: (opts: DialogOptions) => openDialog('success', opts),
    warning: (opts: DialogOptions) => openDialog('warning', opts),
    error: (opts: DialogOptions) => openDialog('error', opts),
    info: (opts: DialogOptions) => openDialog('info', opts),
    loading: (opts: DialogOptions) => openDialog('loading', opts),
    close: () => { dialogState.show = false }
}

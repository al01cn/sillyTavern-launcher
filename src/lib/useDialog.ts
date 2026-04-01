import { reactive } from 'vue'

export const typeConfig = {
  success: { bg: 'bg-emerald-500', lightBg: 'bg-emerald-50', text: 'text-emerald-500' },
  warning: { bg: 'bg-amber-500', lightBg: 'bg-amber-50', text: 'text-amber-500' },
  error: { bg: 'bg-red-500', lightBg: 'bg-red-50', text: 'text-red-500' },
  info: { bg: 'bg-blue-500', lightBg: 'bg-blue-50', text: 'text-blue-500' },
  loading: { bg: 'bg-slate-800', lightBg: 'bg-slate-50', text: 'text-slate-500' },
}

interface DialogOptions {
  title?: string
  msg?: string
  context?: string
  message?: string
  closeOnMask?: boolean
  confirmText?: string
  cancelText?: string
  showCancel?: boolean
  thirdBtnText?: string
  onConfirm?: () => void
  onCancel?: () => void
  onThirdBtn?: () => void
  onClose?: () => void
}

export const dialogState = reactive({
  show: false,
  type: 'info' as keyof typeof typeConfig,
  title: '',
  msg: '',
  closeOnMask: true,
  confirmText: '确定',
  cancelText: '取消',
  showCancel: true,
  thirdBtnText: '',
  // 存储回调函数
  onConfirm: undefined as (() => void) | undefined,
  onCancel: undefined as (() => void) | undefined,
  onThirdBtn: undefined as (() => void) | undefined,
  onClose: undefined as (() => void) | undefined,
})

/**
 * 打开指定类型的对话框
 * @param type - 对话框类型，决定样式
 * @param options - 对话框配置选项
 */
const openDialog = (type: keyof typeof typeConfig, options: DialogOptions) => {
  dialogState.type = type
  dialogState.title = options.title || '提示'
  dialogState.msg = options.msg || options.context || options.message || ''
  dialogState.closeOnMask = options.closeOnMask ?? true
  dialogState.confirmText = options.confirmText || '确定'
  dialogState.cancelText = options.cancelText || '取消'
  dialogState.showCancel = options.showCancel ?? true
  dialogState.thirdBtnText = options.thirdBtnText || ''

  // 绑定传入的函数
  dialogState.onConfirm = options.onConfirm
  dialogState.onCancel = options.onCancel
  dialogState.onThirdBtn = options.onThirdBtn
  dialogState.onClose = options.onClose

  dialogState.show = true
}

/**
 * 对话框工具对象，提供多种类型的对话框调用方法
 */
export const Dialog = {
  /**
   * 打开成功类型的对话框
   * @param opts - 对话框配置选项
   */
  success: (opts: DialogOptions) => openDialog('success', opts),
  /**
   * 打开警告类型的对话框
   * @param opts - 对话框配置选项
   */
  warning: (opts: DialogOptions) => openDialog('warning', opts),
  /**
   * 打开错误类型的对话框
   * @param opts - 对话框配置选项
   */
  error: (opts: DialogOptions) => openDialog('error', opts),
  /**
   * 打开信息类型的对话框
   * @param opts - 对话框配置选项
   */
  info: (opts: DialogOptions) => openDialog('info', opts),
  /**
   * 打开加载中的对话框
   * @param opts - 对话框配置选项
   */
  loading: (opts: DialogOptions) => openDialog('loading', opts),
  /**
   * 关闭当前显示的对话框
   */
  close: () => {
    dialogState.show = false
  },
}

<script lang="ts" setup>
import { dialogState, typeConfig } from '../lib/useDialog'
import { CheckCircle2, AlertTriangle, Info, Loader2, XCircle } from 'lucide-vue-next'

const handleAction = (isConfirm: boolean) => {
    // 1. 执行回调
    if (isConfirm) {
        dialogState.onConfirm?.()
    } else {
        dialogState.onCancel?.()
    }

    // 2. 关闭弹窗动画
    dialogState.show = false
}

const handleMaskClick = () => {
    if (!dialogState.closeOnMask) return
    handleAction(false)
}
</script>

<template>
    <div :class="[
        'absolute inset-0 z-200 flex items-center justify-center px-4 transition-all duration-300',
        dialogState.show ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'
    ]">
        <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-md" @click="handleMaskClick"></div>

        <div :class="[
            'modal-content relative bg-white w-full max-w-85 rounded-4xl shadow-modal border border-slate-100 overflow-hidden transition-all duration-300 transform',
            dialogState.show ? 'scale-100 translate-y-0' : 'scale-95 translate-y-8'
        ]">
            <div class="p-8 text-center">
                <div :class="[
                    'w-16 h-16 rounded-2xl flex items-center justify-center mx-auto mb-5',
                    typeConfig[dialogState.type].lightBg,
                    typeConfig[dialogState.type].text
                ]">
                    <CheckCircle2 v-if="dialogState.type === 'success'" class="w-7 h-7" />
                    <AlertTriangle v-else-if="dialogState.type === 'warning'" class="w-7 h-7" />
                    <XCircle v-else-if="dialogState.type === 'error'" class="w-7 h-7" />
                    <Info v-else-if="dialogState.type === 'info'" class="w-7 h-7" />
                    <Loader2 v-else-if="dialogState.type === 'loading'" class="w-7 h-7 animate-spin" />
                </div>

                <h3 class="text-lg font-black text-slate-800 mb-1.5">{{ dialogState.title }}</h3>
                <p class="text-slate-500 text-[13px] font-medium leading-relaxed">
                    {{ dialogState.msg }}
                </p>
            </div>

            <div v-if="dialogState.type !== 'loading'" class="flex gap-2.5 p-5 pt-0">
                <button @click="handleAction(false)"
                    class="flex-1 py-3 rounded-xl font-bold text-slate-500 bg-slate-100 hover:bg-slate-200 transition-colors text-[13px]">
                    {{ dialogState.cancelText }}
                </button>
                <button @click="handleAction(true)" :class="[
                    'flex-1 py-3 rounded-xl font-bold text-white transition-all active:scale-95 text-[13px]',
                    typeConfig[dialogState.type].bg
                ]">
                    {{ dialogState.confirmText }}
                </button>
            </div>
        </div>
    </div>
</template>

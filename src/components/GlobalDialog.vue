<script lang="ts" setup>
import { dialogState, typeConfig } from '../lib/useDialog'
import { CheckCircle2, AlertTriangle, Info, Loader2, XCircle, X } from 'lucide-vue-next'

const handleAction = (action: 'confirm' | 'cancel' | 'third' | 'close') => {
    // 1. 执行回调
    if (action === 'confirm') {
        dialogState.onConfirm?.()
    } else if (action === 'cancel') {
        dialogState.onCancel?.()
    } else if (action === 'third') {
        dialogState.onThirdBtn?.()
    } else if (action === 'close') {
        dialogState.onClose?.()
    }

    // 2. 关闭弹窗动画
    dialogState.show = false
}

const handleMaskClick = () => {
    if (!dialogState.closeOnMask) return
    handleAction('close')
}
</script>

<template>
    <div 
        v-if="dialogState.show"
        class="absolute inset-0 z-[300] flex items-center justify-center px-4 animate-in fade-in duration-200"
    >
        <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-md pointer-events-auto" @click="handleMaskClick"></div>

        <div 
            class="modal-content relative bg-white w-full max-w-[21.25rem] rounded-4xl shadow-modal border border-slate-100 overflow-hidden pointer-events-auto animate-in zoom-in-95 duration-200"
            @click.stop
        >
            <!-- Close Button -->
            <button 
                @click="handleAction('close')"
                class="absolute top-4 right-4 p-1.5 text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-xl transition-colors z-10"
            >
                <X class="w-5 h-5" />
            </button>

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
                <button v-if="dialogState.showCancel" @click="handleAction('cancel')"
                    class="flex-1 py-3 rounded-xl font-bold text-slate-500 bg-slate-100 hover:bg-slate-200 transition-colors text-[13px]">
                    {{ dialogState.cancelText }}
                </button>
                <button v-if="dialogState.thirdBtnText" @click="handleAction('third')"
                    class="flex-1 py-3 rounded-xl font-bold text-blue-500 bg-blue-50 hover:bg-blue-100 transition-colors text-[13px]">
                    {{ dialogState.thirdBtnText }}
                </button>
                <button @click="handleAction('confirm')" :class="[
                    'flex-1 py-3 rounded-xl font-bold text-white transition-all active:scale-95 text-[13px]',
                    typeConfig[dialogState.type].bg
                ]">
                    {{ dialogState.confirmText }}
                </button>
            </div>
        </div>
    </div>
</template>

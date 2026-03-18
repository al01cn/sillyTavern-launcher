<script lang="ts" setup>
import { X } from 'lucide-vue-next' // 确保你安装并导入了图标库

// 1. 定义 Props
const props = defineProps({
    open: {
        type: Boolean,
        default: false
    },
    title: {
        type: String,
        default: ''
    },
    // 增加一个配置：是否允许点击遮罩层关闭
    closeOnClickOverlay: {
        type: Boolean,
        default: true
    }
})

// 2. 定义 Emits (Vue 3 约定使用 update:modelValue 或自定义事件)
const emit = defineEmits(['update:open', 'close'])

const closeModal = () => {
    // 触发双向绑定更新
    emit('update:open', false)
    // 同时触发一个普通的 close 事件方便父组件监听
    emit('close')
}

const handleOverlayClick = () => {
    if (props.closeOnClickOverlay) {
        closeModal()
    }
}
</script>

<template>
    <Teleport to="body">
        <div v-if="open"
            :class="`${open ? 'opacity-100' : 'opacity-0 pointer-events-none'} fixed inset-0 z-200 flex items-center justify-center px-4 transition-all duration-300`">
            <div class="absolute inset-0 bg-slate-900/40 backdrop-blur-md" @click="handleOverlayClick"></div>

            <div @click.stop
                class="modal-content relative mb-12 bg-white w-full max-w-240 min-h-150 rounded-4xl shadow-modal border border-slate-100 overflow-hidden transition-all duration-300">
                <header class="app-titlebar h-14 shrink-0 flex items-center justify-between px-6 z-60">
                    <div v-if="title" class="flex items-center gap-2.5 w-40">
                        <span class="font-black text-xl tracking-tight text-slate-800 text-nowrap">
                            {{ title }}
                        </span>
                    </div>

                    <div class="flex items-center gap-1">
                        <slot name="header"></slot>
                        <div class="flex items-center w-40 justify-end h-full gap-1">
                            <button @click="closeModal"
                                class="h-8 w-8 rounded-lg flex items-center justify-center text-slate-400 hover:bg-red-500 hover:text-white transition-colors">
                                <X class="w-4 h-4" />
                            </button>
                        </div>
                    </div>
                </header>

                <div class="p-6">
                    <slot></slot>
                </div>
            </div>
        </div>
    </Teleport>
</template>

<style scoped>
/* 如果需要更精细的动画，可以在这里写 transition */
.modal-content {
    transform: scale(0.95);
    animation: modal-in 0.3s forwards cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes modal-in {
    to {
        transform: scale(1);
    }
}
</style>
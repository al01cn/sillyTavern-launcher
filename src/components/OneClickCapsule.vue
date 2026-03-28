<script setup lang="ts">
import { computed } from 'vue';
import { PhRobot, PhWarningCircle } from '@phosphor-icons/vue';
import { oneClickState } from '../lib/useOneClick';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const isVisible = computed(() => oneClickState.isActive);
const currentMessage = computed(() => oneClickState.message);
</script>

<template>
  <div
    v-if="isVisible"
    class="fixed bottom-6 right-6 z-[9999] flex flex-col gap-2 items-end pointer-events-none animate-in slide-in-from-bottom-4 fade-in duration-300"
  >
    <!-- 主胶囊 -->
    <div class="bg-white dark:bg-slate-800 shadow-xl rounded-full pl-2 pr-5 py-2 flex items-center gap-3 border border-blue-100 dark:border-blue-900/30 backdrop-blur-md bg-white/90 dark:bg-slate-800/90 pointer-events-auto">
      <div class="w-10 h-10 rounded-full bg-blue-500 flex items-center justify-center text-white shadow-inner animate-pulse">
        <PhRobot :size="24" weight="duotone" />
      </div>
      <div class="flex flex-col">
        <span class="text-xs font-bold text-blue-600 dark:text-blue-400 uppercase tracking-wider">{{ t('oneClick.status') }}</span>
        <span class="text-sm font-medium text-slate-800 dark:text-slate-200">{{ currentMessage }}</span>
      </div>
    </div>
    
    <!-- 警告提示 -->
    <div class="bg-amber-50 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400 text-xs px-3 py-1.5 rounded-full flex items-center gap-1.5 shadow-sm border border-amber-200/50 dark:border-amber-800/50 pointer-events-auto">
      <PhWarningCircle :size="14" weight="bold" />
      <span>{{ t('oneClick.warning') }}</span>
    </div>
  </div>
</template>

<style scoped>
</style>

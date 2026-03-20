<template>
  <div class="space-y-8 pb-10">
    <div v-for="(toolList, category) in tools" :key="category" class="space-y-2">
      <h2 class="text-lg font-bold text-slate-700 border-b border-slate-100 pb-2">{{ category }}</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <button 
          v-for="(tool, index) in toolList" 
          :key="index"
          @click="openLink(tool.url)"
          class="group bg-white p-5 rounded-2xl border border-slate-200 hover:border-blue-500/30 hover:shadow-md transition-all duration-300 flex items-start gap-4 text-left"
        >
          <div class="w-12 h-12 rounded-xl bg-blue-50 text-blue-500 flex items-center justify-center shrink-0 group-hover:scale-110 transition-transform duration-300 overflow-hidden">
            <img 
              v-if="tool.icon && !imageErrors[`${category}-${index}`]" 
              :src="tool.icon" 
              :alt="tool.name" 
              class="w-full h-full object-cover"
              @error="handleImageError(category, index)"
            />
            <component v-else-if="tool.defaultIcon" :is="tool.defaultIcon" class="w-6 h-6" />
            <Wrench v-else class="w-6 h-6" />
          </div>
          <div class="flex-1 min-w-0 pt-1">
            <h3 class="text-base font-bold text-slate-800 mb-1 group-hover:text-blue-600 transition-colors">{{ tool.name }}</h3>
            <p class="text-xs text-slate-500 truncate">{{ tool.url }}</p>
          </div>
          <div class="text-slate-300 group-hover:text-blue-500 transition-colors pt-2">
            <ExternalLink class="w-4 h-4" />
          </div>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { Wrench, ExternalLink } from 'lucide-vue-next';
import { openUrl as open } from '@tauri-apps/plugin-opener';
import config from '../lib/config';

interface ToolItem {
  name: string;
  url: string;
  icon?: string;
  defaultIcon?: any;
}

const tools = ref<Record<string, ToolItem[]>>(config.tools as Record<string, ToolItem[]> || {});
const imageErrors = ref<Record<string, boolean>>({});

const handleImageError = (category: string | number, index: number) => {
  imageErrors.value[`${category}-${index}`] = true;
};

const openLink = async (url: string) => {
  try {
    await open(url);
  } catch (error) {
    console.error('Failed to open link:', error);
    // Fallback to window.open if plugin-shell fails
    window.open(url, '_blank');
  }
};
</script>

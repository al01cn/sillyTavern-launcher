<script setup lang="ts">
import { onMounted } from 'vue';
import Oheader from './layouts/Oheader.vue';
import { Toaster } from 'vue-sonner'
import { invoke } from '@tauri-apps/api/core';
// import { Dialog } from './lib/useDialog';
import GlobalDialog from './components/GlobalDialog.vue';
import InstallDialog from './components/InstallDialog.vue';
import InstallExtensionDialog from './components/InstallExtensionDialog.vue';
import CharacterCardDialog from './components/CharacterCardDialog.vue';
import UploadCharacterCardDialog from './components/UploadCharacterCardDialog.vue';
import WorldInfoDialog from './components/WorldInfoDialog.vue';
import UploadWorldInfoDialog from './components/UploadWorldInfoDialog.vue';
import { initConsoleState, consoleStatus } from './lib/consoleState';

onMounted(async () => {
  await initConsoleState();
  // 检查是否已经在运行
  try {
    const isRunning = await invoke<boolean>('check_sillytavern_status');
    if (isRunning && consoleStatus.value === 0) {
      consoleStatus.value = 2; // 假设运行中
    }
  } catch (e) {
    console.error(e);
  }
});
</script>

<template>
  <Oheader>
    <!-- 页面区域 -->
    <router-view />

    <!-- 弹窗/全局消息区域 -->
    <template #Modal>
      <!-- 全局确认框 -->
      <GlobalDialog />
      <!-- 安装进度框 -->
      <InstallDialog />
      <!-- 扩展安装框 -->
      <InstallExtensionDialog />
      <CharacterCardDialog />
      <!-- 角色卡导入框 -->
      <UploadCharacterCardDialog />
      <!-- 世界书详情弹窗 -->
      <WorldInfoDialog />
      <!-- 世界书导入框 -->
      <UploadWorldInfoDialog />
    </template>
    <Toaster />
  </Oheader>
</template>

<style scoped></style>

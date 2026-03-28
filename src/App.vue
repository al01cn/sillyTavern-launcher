<script setup lang="ts">
import { onMounted } from 'vue';
import Oheader from './layouts/Oheader.vue';
import { Toaster } from 'vue-sonner'
import { invoke } from '@tauri-apps/api/core';
import { confirm } from '@tauri-apps/plugin-dialog';

import { getCurrentWindow } from '@tauri-apps/api/window';
import { useI18n } from 'vue-i18n';
import GlobalDialog from './components/GlobalDialog.vue';
import InstallDialog from './components/InstallDialog.vue';
import InstallExtensionDialog from './components/InstallExtensionDialog.vue';
import CharacterCardDialog from './components/CharacterCardDialog.vue';
import UploadCharacterCardDialog from './components/UploadCharacterCardDialog.vue';
import WorldInfoDialog from './components/WorldInfoDialog.vue';
import UploadWorldInfoDialog from './components/UploadWorldInfoDialog.vue';
import OneClickCapsule from './components/OneClickCapsule.vue';
import { initConsoleState, consoleStatus, stopProcess } from './lib/consoleState';
import { checkUpdate } from './lib/updater';

const { t } = useI18n();

onMounted(async () => {
  await initConsoleState();

  setTimeout(() => {
    // 检查自动更新
    checkUpdate().catch(console.error);
  }, 1500);

  // 拦截关闭窗口事件
  const appWindow = getCurrentWindow();
  appWindow.onCloseRequested(async (event) => {
    if (consoleStatus.value === 1 || consoleStatus.value === 2) {
      event.preventDefault();

      const yes = await confirm(
        t('home.tavernIsRunningCloseApp'),
        {
          title: t('home.confirmClose'),
          kind: 'warning',
          okLabel: t('common.confirm'),
          cancelLabel: t('common.cancel')
        }
      );

      if (yes) {
        try {
          await stopProcess();
        } catch (e) { }
        await appWindow.destroy();
      }
    }
  });

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

      <!-- 一键安装悬浮球 -->
      <OneClickCapsule />
    </template>
    <Toaster />
  </Oheader>
</template>

<style scoped></style>

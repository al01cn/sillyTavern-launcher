<script setup lang="ts">
import Oheader from './layouts/Oheader.vue';
import { Toaster } from 'vue-sonner'
// import { Dialog } from './lib/useDialog';
import GlobalDialog from './components/GlobalDialog.vue';
import InstallDialog from './components/InstallDialog.vue';
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
    </template>
    <Toaster />
  </Oheader>
</template>

<style scoped></style>

<script setup lang="ts">
import { onMounted } from 'vue'
import Oheader from './layouts/Oheader.vue'
import { Toaster } from 'vue-sonner'
import { invoke } from '@tauri-apps/api/core'
import { confirm } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'

import { getCurrentWindow } from '@tauri-apps/api/window'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import GlobalDialog from './components/GlobalDialog.vue'
import InstallDialog from './components/InstallDialog.vue'
import InstallExtensionDialog from './components/InstallExtensionDialog.vue'
import RepairGitDialog from './components/RepairGitDialog.vue'
import CharacterCardDialog from './components/CharacterCardDialog.vue'
import UploadCharacterCardDialog from './components/UploadCharacterCardDialog.vue'
import WorldInfoDialog from './components/WorldInfoDialog.vue'
import UploadWorldInfoDialog from './components/UploadWorldInfoDialog.vue'
import ChatDialog from './components/ChatDialog.vue'
import OneClickCapsule from './components/OneClickCapsule.vue'
import { initConsoleState, consoleStatus, stopProcess, setConsoleRouter } from './lib/consoleState'
import { checkUpdate } from './lib/updater'
import { initDownloadProgressListener } from './lib/useNodeGitInstall'

const { t } = useI18n()
const router = useRouter()

onMounted(async () => {
  // 全局注册 Git/Node 下载进度监听（只注册一次，页面切换不会丢失进度）
  initDownloadProgressListener().catch(console.error)

  // 注入 router 到 consoleState，供版本检查跳转使用
  setConsoleRouter(router)

  await initConsoleState()

  setTimeout(() => {
    // 检查自动更新
    checkUpdate().catch(console.error)
  }, 1500)

  // 拦截关闭窗口事件
  const appWindow = getCurrentWindow()
  appWindow.onCloseRequested(async event => {
    if (consoleStatus.value === 1 || consoleStatus.value === 2) {
      event.preventDefault()

      const yes = await confirm(t('home.tavernIsRunningCloseApp'), {
        title: t('home.confirmClose'),
        kind: 'warning',
        okLabel: t('common.confirm'),
        cancelLabel: t('common.cancel'),
      })

      if (yes) {
        try {
          await stopProcess()
        } catch (_e) {}
        await appWindow.destroy()
      }
    }
  })

  // 检查是否已经在运行
  try {
    const isRunning = await invoke<boolean>('check_sillytavern_status')
    if (isRunning && consoleStatus.value === 0) {
      consoleStatus.value = 2 // 假设运行中
    }
  } catch (e) {
    console.error(e)
  }

  // 桌面程序模式：监听酒馆启动成功，打开子窗口
  listen<string>('tavern-desktop-ready', event => {
    const tavernUrl = event.payload
    invoke('open_tavern_desktop_window', { url: tavernUrl }).catch(e => {
      console.error('打开桌面程序窗口失败:', e)
    })
  })
})
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
      <!-- Git 修复日志框 -->
      <RepairGitDialog />
      <CharacterCardDialog />
      <!-- 角色卡导入框 -->
      <UploadCharacterCardDialog />
      <!-- 世界书详情弹窗 -->
      <WorldInfoDialog />
      <!-- 世界书导入框 -->
      <UploadWorldInfoDialog />
      <!-- 对话历史预览弹窗 -->
      <ChatDialog />

      <!-- 一键安装悬浮球 -->
      <OneClickCapsule />
    </template>
    <Toaster />
  </Oheader>
</template>

<style scoped></style>

import { check } from '@tauri-apps/plugin-updater';
import { ask, message } from '@tauri-apps/plugin-dialog';
import { relaunch } from '@tauri-apps/plugin-process';

const IGNORE_UPDATE_KEY = 'app_ignore_update_version';

export async function checkUpdate(manual = false) {
  try {
    const update = await check();
    if (update) {
      if (!manual) {
        const ignoredVersion = localStorage.getItem(IGNORE_UPDATE_KEY);
        if (ignoredVersion === update.version) {
          // 当前版本已经被标记为“不再提醒”，不弹窗
          console.log(`已忽略版本 ${update.version} 的自动更新提醒`);
          return;
        }
      }

      const body = update.body ? `\n\n更新说明:\n${update.body}` : '';
      const promptTitle = manual ? '检查更新' : '有新版本可用';
      const promptBody = `发现新版本：${update.version}${body}\n\n是否立即更新？`;

      // 自动检查的话提供“不再提醒”和“稍后”的语义，手动检查则是“取消”
      const yes = await ask(promptBody, {
        title: promptTitle,
        kind: 'info',
        okLabel: '更新并重启',
        cancelLabel: manual ? '取消' : '本版本不再提醒'
      });

      if (yes) {
        let downloaded = 0;
        let contentLength = 0;
        await update.downloadAndInstall((event) => {
          switch (event.event) {
            case 'Started':
              contentLength = event.data.contentLength || 0;
              console.log(`开始下载新版本，文件大小：${contentLength}`);
              break;
            case 'Progress':
              downloaded += event.data.chunkLength;
              console.log(`下载进度：${downloaded}/${contentLength}`);
              break;
            case 'Finished':
              console.log('更新包下载并安装完成');
              break;
          }
        });

        await message('更新完成，应用即将重启', { title: '成功', kind: 'info' });
        await relaunch();
      } else {
        if (!manual) {
          localStorage.setItem(IGNORE_UPDATE_KEY, update.version);
        }
      }
    } else {
      if (manual) {
        await message('当前已经是最新版本，无需更新。', { title: '检查更新', kind: 'info' });
      }
      console.log('未发现新版本');
    }
  } catch (error) {
    console.error('更新检查失败:', error);
    if (manual) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      await message(`检查更新失败：${errorMsg}`, { title: '错误', kind: 'error' });
    }
  }
}

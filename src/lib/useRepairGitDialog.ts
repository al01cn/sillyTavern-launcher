import { reactive } from 'vue';

export const repairGitDialogState = reactive({
    /** 是否显示弹窗 */
    show: false,
    /** 是否正在修复中（锁定关闭） */
    isRepairing: false,
    /** 修复结果：
     *  null=进行中 / 'success'=完整修复成功 / 'offline'=离线保底 / 'error'=失败 */
    result: null as null | 'success' | 'offline' | 'error',
    /** 日志列表 */
    logs: [] as string[],
    /** 扩展名称，用于标题展示 */
    extensionName: '',
});

export const openRepairGitDialog = (extensionName = '') => {
    repairGitDialogState.show = true;
    repairGitDialogState.isRepairing = true;
    repairGitDialogState.result = null;
    repairGitDialogState.logs = [];
    repairGitDialogState.extensionName = extensionName;
};

export const closeRepairGitDialog = () => {
    repairGitDialogState.show = false;
    setTimeout(() => {
        repairGitDialogState.isRepairing = false;
        repairGitDialogState.result = null;
        repairGitDialogState.logs = [];
        repairGitDialogState.extensionName = '';
    }, 300);
};

import { reactive } from 'vue';

export const installExtensionState = reactive({
    show: false,
    version: '',
    onSuccess: undefined as (() => void) | undefined,
});

export const openInstallExtensionDialog = (version: string, onSuccess?: () => void) => {
    installExtensionState.version = version;
    installExtensionState.onSuccess = onSuccess;
    installExtensionState.show = true;
};

export const closeInstallExtensionDialog = () => {
    installExtensionState.show = false;
    setTimeout(() => {
        installExtensionState.version = '';
        installExtensionState.onSuccess = undefined;
    }, 300);
};

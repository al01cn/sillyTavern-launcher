import { reactive } from 'vue';

export const installExtensionState = reactive({
    show: false,
    version: null as {version: string, path: string} | null,
    onSuccess: undefined as (() => void) | undefined,
});

export const openInstallExtensionDialog = (version: {version: string, path: string}, onSuccess?: () => void) => {
    installExtensionState.version = version;
    installExtensionState.onSuccess = onSuccess;
    installExtensionState.show = true;
};

export const closeInstallExtensionDialog = () => {
    installExtensionState.show = false;
    setTimeout(() => {
        installExtensionState.version = null;
        installExtensionState.onSuccess = undefined;
    }, 300);
};

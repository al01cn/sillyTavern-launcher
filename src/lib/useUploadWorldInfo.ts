import { reactive } from 'vue';

interface UploadWorldInfoState {
    show: boolean;
    onSuccess?: () => void;
}

export const uploadWorldInfoState = reactive<UploadWorldInfoState>({
    show: false,
});

export const openUploadWorldInfoDialog = (onSuccess?: () => void) => {
    uploadWorldInfoState.onSuccess = onSuccess;
    uploadWorldInfoState.show = true;
};

export const closeUploadWorldInfoDialog = () => {
    uploadWorldInfoState.show = false;
    uploadWorldInfoState.onSuccess = undefined;
};

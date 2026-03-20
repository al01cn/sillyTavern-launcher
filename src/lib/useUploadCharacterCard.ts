import { reactive } from 'vue'

export const uploadCharacterCardState = reactive({
    show: false,
    onSuccess: undefined as (() => void) | undefined
})

export const openUploadCharacterCardDialog = (onSuccess?: () => void) => {
    uploadCharacterCardState.onSuccess = onSuccess
    uploadCharacterCardState.show = true
}

export const closeUploadCharacterCardDialog = () => {
    uploadCharacterCardState.show = false
}

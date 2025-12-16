import { ref } from 'vue';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

/**
 * Composable for avatar picking with preview
 */
export function useAvatarPicker() {
  const selectedFile = ref<string | null>(null);
  const avatarPreview = ref<string>('');
  const showPreviewDialog = ref<boolean>(false);
  const isUploading = ref<boolean>(false);
  const error = ref<string>('');

  /**
   * Open file picker dialog
   */
  async function selectFile(): Promise<void> {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'Images',
            extensions: ['jpg', 'jpeg', 'png', 'gif']
          }
        ]
      });

      if (selected) {
        selectedFile.value = selected as string;
        avatarPreview.value = convertFileSrc(selected as string);
        error.value = '';
        showPreviewDialog.value = true;
      }
    } catch (err) {
      console.error('Error selecting file:', err);
      error.value = 'Failed to select file';
    }
  }

  /**
   * Upload image with crop ratio and return URL
   * For user avatar (no URL returned, just upload)
   */
  async function uploadUserAvatar(cropRatio: number): Promise<boolean> {
    if (!selectedFile.value || isUploading.value) return false;

    isUploading.value = true;
    error.value = '';

    try {
      await invoke('update_user_avatar', {
        uploadFilepath: selectedFile.value,
        cropRatio,
      });
      return true;
    } catch (err) {
      console.error('Error uploading avatar:', err);
      error.value = err instanceof Error ? err.message : 'Failed to upload avatar';
      return false;
    } finally {
      isUploading.value = false;
    }
  }

  /**
   * Upload image with crop ratio and return URL
   * For group avatar or other uses
   */
  async function uploadImage(cropRatio: number): Promise<string | null> {
    if (!selectedFile.value || isUploading.value) return null;

    isUploading.value = true;
    error.value = '';

    try {
      const url = await invoke<string>('upload_image', {
        uploadFilepath: selectedFile.value,
        cropRatio,
      });
      return url;
    } catch (err) {
      console.error('Error uploading image:', err);
      error.value = err instanceof Error ? err.message : 'Failed to upload image';
      return null;
    } finally {
      isUploading.value = false;
    }
  }

  /**
   * Close preview dialog and reset state
   */
  function closePreview(): void {
    showPreviewDialog.value = false;
    selectedFile.value = null;
    avatarPreview.value = '';
    error.value = '';
  }

  /**
   * Reset all state
   */
  function reset(): void {
    closePreview();
    isUploading.value = false;
  }

  return {
    // State
    selectedFile,
    avatarPreview,
    showPreviewDialog,
    isUploading,
    error,
    // Methods
    selectFile,
    uploadUserAvatar,
    uploadImage,
    closePreview,
    reset,
  };
}

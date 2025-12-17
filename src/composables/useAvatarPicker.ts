import { ref } from 'vue';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

/**
 * Helper to convert Blob to base64 string
 */
async function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => {
      const result = reader.result as string;
      // Remove data URL prefix (e.g., "data:image/png;base64,")
      const base64 = result.split(',')[1];
      resolve(base64);
    };
    reader.onerror = reject;
    reader.readAsDataURL(blob);
  });
}

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
   * Upload user avatar from pre-cropped blob
   */
  async function uploadUserAvatarBlob(blob: Blob): Promise<boolean> {
    if (isUploading.value) return false;

    isUploading.value = true;
    error.value = '';

    try {
      const imageData = await blobToBase64(blob);
      await invoke('upload_user_avatar_blob', { imageData });
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
   * Upload image from pre-cropped blob and return URL
   */
  async function uploadImageBlob(blob: Blob): Promise<string | null> {
    if (isUploading.value) return null;

    isUploading.value = true;
    error.value = '';

    try {
      const imageData = await blobToBase64(blob);
      const url = await invoke<string>('upload_image_blob', { imageData });
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
   * Upload group avatar from pre-cropped blob
   */
  async function uploadGroupAvatarBlob(groupId: string, blob: Blob): Promise<boolean> {
    if (isUploading.value) return false;

    isUploading.value = true;
    error.value = '';

    try {
      const imageData = await blobToBase64(blob);
      await invoke('upload_group_avatar_blob', { groupId, imageData });
      return true;
    } catch (err) {
      console.error('Error uploading group avatar:', err);
      error.value = err instanceof Error ? err.message : 'Failed to upload group avatar';
      return false;
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
    uploadUserAvatarBlob,
    uploadImageBlob,
    uploadGroupAvatarBlob,
    closePreview,
    reset,
  };
}

import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface UploadProgress {
  uploadMode: number; // 0=exists, 1=single, 2=chunk
  currentChunk: number;
  totalChunks: number;
  fileName: string;
}

export interface UploadAttachmentResponse {
  file_url: string;
}

/**
 * Composable for file upload functionality
 *
 * Provides methods to:
 * - Upload files to the server
 * - Track upload progress
 * - Cancel ongoing uploads
 *
 * @returns Upload state and methods
 */
export function useFileUpload() {
  const uploading = ref(false);
  const uploadProgress = ref<UploadProgress | null>(null);
  const uploadError = ref<string | null>(null);

  // Progress percentage (0-100)
  const progressPercent = computed(() => {
    if (!uploadProgress.value) return 0;
    const { uploadMode, currentChunk, totalChunks } = uploadProgress.value;

    // Mode 0: File already exists - instant complete
    if (uploadMode === 0) return 100;
    // Mode 1: Single upload - indeterminate (show as spinner in UI)
    if (uploadMode === 1) return -1;
    // Mode 2: Chunked upload - calculate percentage
    if (uploadMode === 2 && totalChunks > 0) {
      return Math.round((currentChunk / totalChunks) * 100);
    }
    return 0;
  });

  /**
   * Upload a file to the server
   *
   * @param filePath - Full path to the file to upload
   * @returns Promise resolving to the file URL
   */
  async function uploadFile(filePath: string): Promise<string> {
    try {
      uploading.value = true;
      uploadError.value = null;

      // Extract filename from path
      const fileName = filePath.split('/').pop() || filePath.split('\\').pop() || 'file';

      // Set initial progress state
      uploadProgress.value = {
        uploadMode: 1, // Assume single upload initially
        currentChunk: 0,
        totalChunks: 0,
        fileName,
      };

      console.log('[useFileUpload] Starting upload:', filePath);

      const result = await invoke<UploadAttachmentResponse>('upload_attachment', {
        filePath,
      });

      console.log('[useFileUpload] Upload complete:', result.file_url);
      return result.file_url;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      uploadError.value = errorMessage;
      console.error('[useFileUpload] Upload failed:', error);
      throw error;
    } finally {
      uploading.value = false;
      uploadProgress.value = null;
    }
  }

  /**
   * Cancel ongoing upload (only works for chunked uploads)
   * Note: Currently the backend doesn't support progress events,
   * so this is a placeholder for future implementation
   */
  function cancelUpload(): void {
    console.log('[useFileUpload] Cancel upload requested');
    // TODO: Implement abort_attachment_upload when progress events are added
    uploading.value = false;
    uploadProgress.value = null;
  }

  /**
   * Reset upload state
   */
  function resetUpload(): void {
    uploading.value = false;
    uploadProgress.value = null;
    uploadError.value = null;
  }

  return {
    // State
    uploading,
    uploadProgress,
    uploadError,
    progressPercent,

    // Methods
    uploadFile,
    cancelUpload,
    resetUpload,
  };
}

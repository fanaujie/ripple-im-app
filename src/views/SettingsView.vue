<template>
  <div class="h-full bg-background">
    <!-- Main Content -->
    <div class="max-w-4xl mx-auto p-8">
      <!-- Page Header -->
      <div class="mb-8">
        <h1 class="text-2xl font-semibold text-text mb-2">Settings</h1>
        <hr class="mt-4 border-border">
      </div>

      <!-- Basic Settings Section -->
      <div class="bg-surface rounded-xl border border-border overflow-hidden">
        <div class="p-6 space-y-8">

          <!-- Profile Image -->
          <div class="space-y-4">
            <h3 class="text-sm font-medium text-text">Profile Image</h3>

            <div class="flex items-start space-x-6">
              <div class="relative group">
                <div
                    class="w-20 h-20 rounded-full overflow-hidden bg-muted border-2 border-border flex items-center justify-center transition-all duration-200 group-hover:border-primary/50">
                  <img :src="avatarPreview || getAvatarUrl(userProfile?.avatar)"
                       alt="Profile Image"
                       class="w-full h-full object-cover"
                       @error="onImageError">
                </div>


              </div>

              <div class="flex-1 space-y-3">
                <p class="text-sm text-muted-foreground">JPG, PNG or GIF (max. 5MB, min. 460x460)</p>

                <!-- Error Message -->
                <div v-if="imageError" class="p-3 bg-red-50 border border-red-200 rounded-lg">
                  <p class="text-sm text-red-600">{{ imageError }}</p>
                </div>

                <!-- Avatar Actions -->
                <div class="flex space-x-3">
                  <button @click="selectFile"
                          :disabled="isUpdating"
                          :title="userProfile?.avatar ? 'Change Profile Image' : 'Upload Profile Image'"
                          class="p-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"/>
                    </svg>
                  </button>
                  <button v-if="userProfile?.avatar"
                          @click="removeAvatar"
                          :disabled="isUpdating"
                          title="Remove Profile Image"
                          class="p-2 text-destructive border border-destructive/20 rounded-lg hover:bg-destructive/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                    </svg>
                  </button>
                </div>

              </div>
            </div>
          </div>

          <!-- Nickname -->
          <div class="flex items-center space-x-6">
            <label class="text-sm font-medium text-text w-24">Nickname</label>
            <div class="flex-1 flex items-center space-x-3">
              <!-- Display Mode -->
              <div v-if="!isEditingNickname" class="flex-1 flex items-center space-x-3">
                <span class="px-4 py-2.5 text-text">{{ userProfile?.nickName || 'Not set' }}</span>
                <button @click="startEditingNickname"
                        class="p-1.5 text-muted-foreground hover:text-text transition-colors">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
                  </svg>
                </button>
              </div>

              <!-- Edit Mode -->
              <div v-else class="flex-1 flex space-x-3">
                <input v-model="nicknameInput"
                       type="text"
                       class="flex-1 px-4 py-2.5 border border-border rounded-lg bg-background text-text focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary transition-colors"
                       placeholder="Enter your nickname"
                       :disabled="isUpdating"
                       @keydown.enter="saveNickname"
                       @keydown.escape="cancelEditingNickname">
                <button @click="saveNickname"
                        :disabled="isUpdating || !nicknameInput.trim()"
                        class="p-1.5 text-green-600 hover:text-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
                  </svg>
                </button>
                <button @click="cancelEditingNickname"
                        :disabled="isUpdating"
                        class="p-1.5 text-red-500 hover:text-red-600 hover:bg-red-50 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                  <svg class="w-5 h-5" fill="currentColor" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                  </svg>
                </button>
              </div>
            </div>
          </div>

          <!-- User ID (Read-only) -->
          <div class="flex items-center space-x-6">
            <label class="text-sm font-medium text-text w-24">User ID</label>
            <div
                class="flex-1 px-4 py-2.5 bg-muted/30 border border-border rounded-lg text-muted-foreground font-mono text-sm">
              {{ userId || 'Loading...' }}
            </div>
          </div>

        </div>
      </div>
    </div>

    <!-- Avatar Preview Dialog -->
    <div v-if="showPreviewDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
         @click="closePreviewDialog">
      <div class="bg-surface rounded-2xl border border-border max-w-md w-full mx-4 overflow-hidden" @click.stop>
        <!-- Dialog Header -->
        <div class="px-6 py-4 border-b border-border">
          <h3 class="text-lg font-semibold text-text">Preview Profile Image</h3>
        </div>

        <!-- Dialog Content -->
        <div class="p-6">
          <div class="w-full rounded-xl overflow-hidden bg-muted border border-border mb-6 relative"
               :style="{ minHeight: '320px', maxHeight: '400px' }">
            <div v-if="avatarPreview" class="relative w-full h-full flex items-center justify-center">
              <img ref="previewImage"
                   :src="avatarPreview"
                   alt="Avatar Preview"
                   class="max-w-full h-auto"
                   @load="onImageLoad">

              <!-- Cropping overlay -->
              <div v-if="showCropOverlay"
                   class="absolute inset-0 pointer-events-none">
                <!-- Dark overlay -->
                <div class="absolute inset-0 bg-black/40"></div>

                <!-- Crop selection area -->
                <div
                    :style="{
                    position: 'absolute',
                    left: cropArea.x + 'px',
                    top: cropArea.y + 'px',
                    width: cropArea.size + 'px',
                    height: cropArea.size + 'px',
                    border: '2px solid white',
                    backgroundColor: 'transparent',
                    boxShadow: '0 0 0 9999px rgba(0, 0, 0, 0.5)'
                  }"
                    class="pointer-events-auto cursor-move"
                    @mousedown="startDrag"
                    @touchstart="startDrag">
                </div>
              </div>
            </div>
            <div v-else class="flex items-center justify-center h-full text-muted-foreground">
              No image selected
            </div>
          </div>


          <!-- Dialog Actions -->
          <div class="flex justify-end space-x-3">
            <button @click="closePreviewDialog"
                    :disabled="isUpdating"
                    class="px-4 py-2 text-sm text-muted-foreground border border-border rounded-lg hover:bg-muted/50 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
              Cancel
            </button>
            <button @click="saveAvatar"
                    :disabled="isUpdating"
                    class="px-4 py-2 text-sm bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
              {{ isUpdating ? 'Saving...' : 'Save' }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Error Dialog -->
    <div v-if="showErrorDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
         @click="closeErrorDialog">
      <div class="bg-surface rounded-2xl border border-border max-w-md w-full mx-4 overflow-hidden" @click.stop>
        <!-- Dialog Header -->
        <div class="px-6 py-4 border-b border-border">
          <h3 class="text-lg font-semibold text-text">Error</h3>
        </div>

        <!-- Dialog Content -->
        <div class="p-6">
          <p class="text-sm text-muted-foreground mb-6">{{ errorMessage }}</p>

          <!-- Dialog Actions -->
          <div class="flex justify-end">
            <button @click="closeErrorDialog"
                    class="px-4 py-2 text-sm bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors">
              OK
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {ref, computed, nextTick} from 'vue';
import {invoke, convertFileSrc} from '@tauri-apps/api/core';
import {open} from '@tauri-apps/plugin-dialog';
import { useUserProfileDisplay } from '../composables/useUserProfileDisplay';
import defaultAvatarUrl from '../assets/default-avatar.svg';

// Define component name for KeepAlive
defineOptions({
  name: 'SettingsView'
});

// Use composables
const { userProfile, loading: profileLoading, error: profileError } = useUserProfileDisplay();

// Local UI state
const nicknameInput = ref<string>('');
const isUpdating = ref<boolean>(false);
const isEditingNickname = ref<boolean>(false);

// Computed properties
const userId = computed(() => userProfile.value?.userId || 'Unknown');
const selectedFile = ref<string | null>(null);
const avatarPreview = ref<string>('');
const showPreviewDialog = ref<boolean>(false);
const previewImage = ref<HTMLImageElement>();
const showCropOverlay = ref<boolean>(false);
const cropArea = ref({x: 50, y: 50, size: 200});
const isDragging = ref<boolean>(false);
const dragStart = ref({x: 0, y: 0});
const imageRect = ref({width: 0, height: 0, left: 0, top: 0});
const containerRect = ref({width: 0, height: 0, left: 0, top: 0});
const imageError = ref<string>('');
const showErrorDialog = ref<boolean>(false);
const errorMessage = ref<string>('');

// Computed (for future use)
// const hasUnsavedChanges = computed(() => {
//   return nicknameInput.value !== userProfile.value?.nickname || selectedFile.value !== null;
// });

// Methods - Simplified without manual state management


const startEditingNickname = () => {
  isEditingNickname.value = true;
  nicknameInput.value = userProfile.value?.nickName || '';
};

const cancelEditingNickname = () => {
  isEditingNickname.value = false;
  nicknameInput.value = userProfile.value?.nickName || '';
};

const saveNickname = async () => {
  if (!nicknameInput.value.trim() || isUpdating.value) return;

  isUpdating.value = true;
  try {
    await invoke('update_user_nickname', {
      nickname: nicknameInput.value.trim()
    });

    isEditingNickname.value = false;
    // UI will update automatically via event
  } catch (error) {
    console.error('Error updating nickname:', error);
    showErrorDialog.value = true;
    errorMessage.value = error instanceof Error ? error.message : 'Failed to update nickname';
  } finally {
    isUpdating.value = false;
  }
};

const selectFile = async () => {
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
      await handleFileSelected(selected as string);
    }
  } catch (error) {
    console.error('Error selecting file:', error);
    imageError.value = 'Failed to select file';
  }
};

const onImageLoad = () => {
  if (!previewImage.value) return;

  // Use nextTick to ensure DOM is fully rendered
  nextTick(() => {
    if (!previewImage.value) return;

    // Get the container and image dimensions
    const container = previewImage.value.parentElement;
    const containerBounds = container?.getBoundingClientRect();
    const imgRect = previewImage.value.getBoundingClientRect();

    if (!containerBounds) return;

    imageRect.value = {
      width: imgRect.width,
      height: imgRect.height,
      left: imgRect.left,
      top: imgRect.top
    };

    containerRect.value = {
      width: containerBounds.width,
      height: containerBounds.height,
      left: containerBounds.left,
      top: containerBounds.top
    };

    const img = previewImage.value;
    const originalWidth = img.naturalWidth;
    const originalHeight = img.naturalHeight;

    // Calculate image position within container
    const imgOffsetX = imgRect.left - containerBounds.left;
    const imgOffsetY = imgRect.top - containerBounds.top;

    // For images >= 460x460, calculate crop area
    const displayMinSize = Math.min(imgRect.width, imgRect.height);
    const scaleX = imgRect.width / originalWidth;
    const scaleY = imgRect.height / originalHeight;
    const averageScale = Math.min(scaleX, scaleY);

    const targetDisplaySize = 460 * averageScale;
    const maxDisplaySize = displayMinSize * 0.95;
    const finalSize = Math.max(Math.min(targetDisplaySize, maxDisplaySize), displayMinSize * 0.6);
    const cropSize = Math.max(50, finalSize);

    cropArea.value = {
      x: imgOffsetX + (imgRect.width - cropSize) / 2,
      y: imgOffsetY + (imgRect.height - cropSize) / 2,
      size: cropSize
    };

    // Validate and fix cropArea values
    cropArea.value = {
      x: Math.max(0, cropArea.value.x || 0),
      y: Math.max(0, cropArea.value.y || 0),
      size: Math.max(50, cropArea.value.size || 50)  // Minimum 50px
    };

  });
};


const startDrag = (e: MouseEvent | TouchEvent) => {
  e.preventDefault();
  isDragging.value = true;

  const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX;
  const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;

  dragStart.value = {
    x: (clientX - containerRect.value.left) - cropArea.value.x,
    y: (clientY - containerRect.value.top) - cropArea.value.y
  };

  document.addEventListener('mousemove', handleDrag);
  document.addEventListener('mouseup', stopDrag);
  document.addEventListener('touchmove', handleDrag);
  document.addEventListener('touchend', stopDrag);
};

const handleDrag = (e: MouseEvent | TouchEvent) => {
  if (!isDragging.value) return;

  const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX;
  const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;

  // Calculate new position relative to container
  const containerX = clientX - containerRect.value.left;
  const containerY = clientY - containerRect.value.top;

  const newX = Math.max(0, Math.min(containerRect.value.width - cropArea.value.size, containerX - dragStart.value.x));
  const newY = Math.max(0, Math.min(containerRect.value.height - cropArea.value.size, containerY - dragStart.value.y));

  cropArea.value.x = newX;
  cropArea.value.y = newY;
};

const stopDrag = () => {
  isDragging.value = false;

  document.removeEventListener('mousemove', handleDrag);
  document.removeEventListener('mouseup', stopDrag);
  document.removeEventListener('touchmove', handleDrag);
  document.removeEventListener('touchend', stopDrag);
};


const handleFileSelected = async (filePath: string) => {
  try {
    selectedFile.value = filePath;
    console.log('File selected:', filePath);

    // Convert file path to Tauri asset URL for preview
    const assetUrl = convertFileSrc(filePath);
    avatarPreview.value = assetUrl;

    // Check image dimensions
    const img = new Image();
    img.onload = () => {
      // Check if image is too small
      if (img.width < 460 || img.height < 460) {
        imageError.value = `Image too small. Please select an image that is at least 460x460 pixels. Current size: ${img.width}x${img.height}`;
        avatarPreview.value = '';
        selectedFile.value = null;
        return;
      }

      // Clear any previous errors
      imageError.value = '';
      showCropOverlay.value = true;
      showPreviewDialog.value = true;
    };
    img.onerror = () => {
      console.error('Failed to load image from:', assetUrl);
      imageError.value = 'Failed to load selected image';
      avatarPreview.value = '';
      selectedFile.value = null;
    };
    img.src = assetUrl;
  } catch (error) {
    console.error('Error handling file selection:', error);
    imageError.value = 'Failed to process selected file';
  }
};

const saveAvatar = async () => {
  if (!selectedFile.value || isUpdating.value) return;

  isUpdating.value = true;
  try {
    let x = 0, y = 0;
    const width = 460; // Always crop to 460x460
    const height = 460;
    const filename = selectedFile.value; // Now this is the complete file path

    if (previewImage.value) {
      const img = previewImage.value;
      const originalWidth = img.naturalWidth;
      const originalHeight = img.naturalHeight;

      // For large images >= 460x460, use crop area
      const displayScaleX = img.width / originalWidth;
      const displayScaleY = img.height / originalHeight;
      const displayScale = Math.min(displayScaleX, displayScaleY);

      // Convert container-relative crop area to image-relative coordinates
      const imgOffsetX = (imageRect.value.left - containerRect.value.left);
      const imgOffsetY = (imageRect.value.top - containerRect.value.top);

      const cropRelativeToImage = {
        x: cropArea.value.x - imgOffsetX,
        y: cropArea.value.y - imgOffsetY,
        size: cropArea.value.size
      };

      // Convert to original image coordinates
      x = cropRelativeToImage.x / displayScale;
      y = cropRelativeToImage.y / displayScale;
    }

    // Call the API with complete file path
    await invoke('update_user_avatar', {
      uploadFilepath: filename,
      x,
      y,
      width,
      height
    });

    // UI will update automatically via event
    closePreviewDialog();
  } catch (error) {
    console.error('Error updating avatar:', error);
    showErrorDialog.value = true;
    errorMessage.value = error instanceof Error ? error.message : 'Failed to update avatar';
  } finally {
    isUpdating.value = false;
  }
};

const removeAvatar = async () => {
  if (isUpdating.value) return;

  isUpdating.value = true;
  try {
    await invoke('remove_user_avatar');

    closePreviewDialog();
    // UI will update automatically via event
  } catch (error) {
    console.error('Error removing avatar:', error);
    showErrorDialog.value = true;
    errorMessage.value = error instanceof Error ? error.message : 'Failed to remove avatar';
  } finally {
    isUpdating.value = false;
  }
};

const closePreviewDialog = () => {
  showPreviewDialog.value = false;
  showCropOverlay.value = false;
  selectedFile.value = null;
  avatarPreview.value = '';
  imageError.value = '';

  // Reset crop area
  cropArea.value = {x: 50, y: 50, size: 200};
};

const closeErrorDialog = () => {
  showErrorDialog.value = false;
  errorMessage.value = '';
};

const getAvatarUrl = (avatarPath?: string) => {
  // If no avatar path, return default avatar
  if (!avatarPath) return defaultAvatarUrl;

  // Check if it's already an HTTP URL, if so return as-is
  if (avatarPath.startsWith('http://') || avatarPath.startsWith('https://')) {
    return avatarPath;
  }
  // Otherwise treat as local asset path
  return `asset://localhost/${avatarPath}`;
};

const onImageError = (event: Event) => {
  // Handle image loading errors by falling back to default avatar
  const img = event.target as HTMLImageElement;
  if (img.src !== defaultAvatarUrl) {
    console.warn('Failed to load avatar image, falling back to default');
    img.src = defaultAvatarUrl;
  }
};

// No lifecycle needed - useUserProfileDisplay handles everything automatically
</script>
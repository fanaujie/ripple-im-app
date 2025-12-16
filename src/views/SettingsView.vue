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
                  <img :src="getAvatarUrl(userProfile?.avatar)"
                       alt="Profile Image"
                       class="w-full h-full object-cover"
                       @error="onImageError">
                </div>
              </div>

              <div class="flex-1 space-y-3">
                <p class="text-sm text-muted-foreground">JPG, PNG or GIF (max. 5MB)</p>

                <!-- Error Message -->
                <div v-if="avatarPicker.error.value" class="p-3 bg-red-50 border border-red-200 rounded-lg">
                  <p class="text-sm text-red-600">{{ avatarPicker.error.value }}</p>
                </div>

                <!-- Avatar Actions -->
                <div class="flex space-x-3">
                  <button @click="avatarPicker.selectFile()"
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
    <AvatarPreviewDialog
      :is-open="avatarPicker.showPreviewDialog.value"
      :preview-url="avatarPicker.avatarPreview.value"
      :is-uploading="avatarPicker.isUploading.value"
      title="Preview Profile Image"
      @close="avatarPicker.closePreview()"
      @confirm="handleAvatarConfirm"
    />

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
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useUserProfileDisplay } from '../composables/useUserProfileDisplay';
import { useAvatarPicker } from '../composables/useAvatarPicker';
import AvatarPreviewDialog from '../components/common/AvatarPreviewDialog.vue';
import defaultAvatarUrl from '../assets/default-avatar.svg';

// Define component name for KeepAlive
defineOptions({
  name: 'SettingsView'
});

// Use composables
const { userProfile } = useUserProfileDisplay();
const avatarPicker = useAvatarPicker();

// Local UI state
const nicknameInput = ref<string>('');
const isUpdating = ref<boolean>(false);
const isEditingNickname = ref<boolean>(false);
const showErrorDialog = ref<boolean>(false);
const errorMessage = ref<string>('');

// Computed properties
const userId = computed(() => userProfile.value?.userId || 'Unknown');

// Nickname methods
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
  } catch (error) {
    console.error('Error updating nickname:', error);
    showErrorDialog.value = true;
    errorMessage.value = error instanceof Error ? error.message : 'Failed to update nickname';
  } finally {
    isUpdating.value = false;
  }
};

// Avatar methods
async function handleAvatarConfirm(cropRatio: number) {
  const success = await avatarPicker.uploadUserAvatar(cropRatio);
  if (success) {
    avatarPicker.closePreview();
  } else {
    showErrorDialog.value = true;
    errorMessage.value = avatarPicker.error.value || 'Failed to update avatar';
  }
}

const removeAvatar = async () => {
  if (isUpdating.value) return;

  isUpdating.value = true;
  try {
    await invoke('remove_user_avatar');
  } catch (error) {
    console.error('Error removing avatar:', error);
    showErrorDialog.value = true;
    errorMessage.value = error instanceof Error ? error.message : 'Failed to remove avatar';
  } finally {
    isUpdating.value = false;
  }
};

const closeErrorDialog = () => {
  showErrorDialog.value = false;
  errorMessage.value = '';
};

const getAvatarUrl = (avatarPath?: string) => {
  if (!avatarPath) return defaultAvatarUrl;
  if (avatarPath.startsWith('http://') || avatarPath.startsWith('https://')) {
    return avatarPath;
  }
  return `asset://localhost/${avatarPath}`;
};

const onImageError = (event: Event) => {
  const img = event.target as HTMLImageElement;
  if (img.src !== defaultAvatarUrl) {
    console.warn('Failed to load avatar image, falling back to default');
    img.src = defaultAvatarUrl;
  }
};
</script>

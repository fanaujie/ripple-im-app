<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center">
    <!-- Backdrop -->
    <div class="absolute inset-0 bg-black/50" @click="handleClose"></div>

    <!-- Dialog -->
    <div class="relative bg-white rounded-lg shadow-xl w-full max-w-md mx-4">
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200">
        <h2 class="text-xl font-semibold">Edit Group Info</h2>
        <button
          @click="handleClose"
          :disabled="isUploadingAvatar"
          class="text-gray-400 hover:text-gray-600 disabled:opacity-50"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="px-6 py-4">
        <!-- Group Avatar -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Group Avatar
          </label>
          <div class="flex items-start space-x-4">
            <div class="relative">
              <div class="w-16 h-16 rounded-full overflow-hidden bg-gray-200 border-2 border-gray-300 flex items-center justify-center">
                <img
                  v-if="currentAvatarDisplay"
                  :src="currentAvatarDisplay"
                  alt="Group Avatar"
                  class="w-full h-full object-cover"
                  @error="onAvatarError"
                />
                <svg v-else class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                </svg>
              </div>
              <!-- Loading overlay for avatar -->
              <div
                v-if="isUploadingAvatar"
                class="absolute inset-0 bg-black/40 rounded-full flex items-center justify-center"
              >
                <svg class="w-6 h-6 text-white animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
              </div>
            </div>
            <div class="flex-1 space-y-2">
              <p class="text-xs text-gray-500">JPG, PNG or GIF (max. 5MB)</p>
              <!-- Avatar Actions -->
              <div class="flex space-x-2">
                <button
                  @click="avatarPicker.selectFile()"
                  :disabled="isUploadingAvatar"
                  :title="currentAvatarDisplay ? 'Change Group Avatar' : 'Upload Group Avatar'"
                  type="button"
                  class="p-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"/>
                  </svg>
                </button>
                <button
                  v-if="currentAvatarDisplay"
                  @click="handleDeleteAvatar"
                  :disabled="isUploadingAvatar || isDeletingAvatar"
                  title="Remove Group Avatar"
                  type="button"
                  class="p-2 text-red-500 border border-red-200 rounded-lg hover:bg-red-50 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <svg v-if="!isDeletingAvatar" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                  </svg>
                  <svg v-else class="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                </button>
              </div>
              <!-- Avatar success message -->
              <p v-if="avatarSuccessMessage" class="text-xs text-green-600">
                {{ avatarSuccessMessage }}
              </p>
              <!-- Avatar error message -->
              <p v-if="avatarErrorMessage" class="text-xs text-red-600">
                {{ avatarErrorMessage }}
              </p>
            </div>
          </div>
        </div>

        <!-- Group Name Input -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Group Name <span class="text-red-500">*</span>
          </label>
          <div class="flex items-center gap-2">
            <input
              v-model="localGroupName"
              type="text"
              :maxlength="100"
              :disabled="isUpdatingName"
              placeholder="Enter group name"
              class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100 disabled:cursor-not-allowed"
              @keyup.enter="handleSaveName"
            />
            <!-- Checkmark button - only show when name changed and valid -->
            <button
              v-if="canSaveName"
              @click="handleSaveName"
              :disabled="isUpdatingName"
              type="button"
              class="w-10 h-10 flex items-center justify-center text-white bg-blue-500 rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed flex-shrink-0"
            >
              <svg v-if="!isUpdatingName" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              <svg v-else class="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            </button>
          </div>
          <p class="text-xs text-gray-500 mt-1">{{ localGroupName.length }}/100 characters</p>
          <p v-if="localGroupName.trim() === ''" class="text-xs text-red-500 mt-1">
            Group name is required
          </p>
          <!-- Name success message -->
          <p v-if="nameSuccessMessage" class="text-xs text-green-600 mt-1">
            {{ nameSuccessMessage }}
          </p>
          <!-- Name error message -->
          <p v-if="nameErrorMessage" class="text-xs text-red-600 mt-1">
            {{ nameErrorMessage }}
          </p>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end px-6 py-4 border-t border-gray-200">
        <button
          @click="handleClose"
          :disabled="isUploadingAvatar"
          class="px-4 py-2 text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Close
        </button>
      </div>
    </div>

    <!-- Avatar Preview Dialog -->
    <AvatarPreviewDialog
      :is-open="avatarPicker.showPreviewDialog.value"
      :preview-url="avatarPicker.avatarPreview.value"
      :is-uploading="isUploadingAvatar"
      title="Preview Group Avatar"
      @close="avatarPicker.closePreview()"
      @confirm="confirmAndUploadAvatar"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useGroupActions } from '../../composables/chat/useGroupActions';
import { useAvatarPicker } from '../../composables/useAvatarPicker';
import AvatarPreviewDialog from '../common/AvatarPreviewDialog.vue';

defineOptions({
  name: 'EditGroupDialog',
});

interface Props {
  isOpen: boolean;
  groupId: string;
  groupName: string;
  groupAvatar?: string;
  currentUserId: string;
}

const props = defineProps<Props>();

interface Emits {
  (e: 'close'): void;
  (e: 'success'): void;
}

const emit = defineEmits<Emits>();

const { updateGroupName, updateGroupAvatar } = useGroupActions();
const avatarPicker = useAvatarPicker();

// State
const localGroupName = ref('');
const savedGroupName = ref(''); // Track the last saved name
const isUploadingAvatar = ref(false);
const isDeletingAvatar = ref(false);
const isUpdatingName = ref(false);
const avatarSuccessMessage = ref('');
const avatarErrorMessage = ref('');
const nameSuccessMessage = ref('');
const nameErrorMessage = ref('');
const uploadedAvatarPreview = ref<string>('');

// Computed
const canSaveName = computed(() => {
  const trimmedName = localGroupName.value.trim();
  return trimmedName.length > 0 &&
         trimmedName.length <= 100 &&
         trimmedName !== savedGroupName.value;
});

const currentAvatarDisplay = computed(() => {
  if (uploadedAvatarPreview.value) return uploadedAvatarPreview.value;
  if (!props.groupAvatar) return '';
  if (props.groupAvatar.startsWith('http://') || props.groupAvatar.startsWith('https://')) {
    return props.groupAvatar;
  }
  return `asset://localhost/${props.groupAvatar}`;
});

// Methods
function onAvatarError(event: Event) {
  const img = event.target as HTMLImageElement;
  img.style.display = 'none';
}

function resetState() {
  localGroupName.value = props.groupName || '';
  savedGroupName.value = props.groupName || '';
  isUploadingAvatar.value = false;
  isDeletingAvatar.value = false;
  isUpdatingName.value = false;
  avatarSuccessMessage.value = '';
  avatarErrorMessage.value = '';
  nameSuccessMessage.value = '';
  nameErrorMessage.value = '';
  uploadedAvatarPreview.value = '';
  avatarPicker.reset();
}

function handleClose() {
  if (isUploadingAvatar.value) return;
  resetState();
  emit('close');
}

function clearAvatarMessages() {
  avatarSuccessMessage.value = '';
  avatarErrorMessage.value = '';
}

function clearNameMessages() {
  nameSuccessMessage.value = '';
  nameErrorMessage.value = '';
}

async function handleDeleteAvatar() {
  if (isDeletingAvatar.value || isUploadingAvatar.value) return;

  clearAvatarMessages();
  // Note: Delete group avatar API is not available yet
  avatarErrorMessage.value = 'Delete group avatar is not supported yet';
}

async function confirmAndUploadAvatar(cropRatio: number) {
  if (!avatarPicker.selectedFile.value || isUploadingAvatar.value) return;

  isUploadingAvatar.value = true;
  clearAvatarMessages();

  // Store preview before closing dialog
  const previewUrl = avatarPicker.avatarPreview.value;
  const filePath = avatarPicker.selectedFile.value;

  // Close preview dialog immediately
  avatarPicker.closePreview();

  try {
    await updateGroupAvatar(props.groupId, filePath, cropRatio);
    uploadedAvatarPreview.value = previewUrl;
    avatarSuccessMessage.value = 'Avatar updated';
    emit('success');

    // Clear success message after delay
    setTimeout(() => {
      avatarSuccessMessage.value = '';
    }, 3000);

    console.log('[EditGroupDialog] Group avatar uploaded successfully');
  } catch (error) {
    console.error('[EditGroupDialog] Failed to upload avatar:', error);
    avatarErrorMessage.value = error instanceof Error ? error.message : 'Failed to update avatar';
  } finally {
    isUploadingAvatar.value = false;
  }
}

async function handleSaveName() {
  if (!canSaveName.value || isUpdatingName.value) return;

  isUpdatingName.value = true;
  clearNameMessages();

  const newName = localGroupName.value.trim();

  try {
    await updateGroupName(props.groupId, props.currentUserId, newName);
    savedGroupName.value = newName;
    nameSuccessMessage.value = 'Name updated';
    emit('success');

    // Clear success message after delay
    setTimeout(() => {
      nameSuccessMessage.value = '';
    }, 3000);

    console.log('[EditGroupDialog] Group name updated successfully');
  } catch (error) {
    console.error('[EditGroupDialog] Failed to update name:', error);
    nameErrorMessage.value = error instanceof Error ? error.message : 'Failed to update name';
  } finally {
    isUpdatingName.value = false;
  }
}

// Initialize form when dialog opens
watch(
  () => props.isOpen,
  (newValue) => {
    if (newValue) {
      resetState();
    }
  }
);

// Update local state when props change (e.g., from WebSocket updates)
watch(
  () => props.groupName,
  (newValue) => {
    if (props.isOpen && !isUpdatingName.value) {
      localGroupName.value = newValue || '';
      savedGroupName.value = newValue || '';
    }
  }
);
</script>

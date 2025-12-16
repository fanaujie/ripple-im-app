<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center">
    <!-- Backdrop -->
    <div class="absolute inset-0 bg-black/50" @click="handleClose"></div>

    <div class="relative bg-white rounded-lg shadow-xl w-full max-w-md mx-4">
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200">
        <h2 class="text-xl font-semibold">Create Group</h2>
        <button
          @click="handleClose"
          :disabled="isCreating"
          class="text-gray-400 hover:text-gray-600 disabled:opacity-50"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="px-6 py-4 max-h-96 overflow-y-auto">
        <!-- Group Avatar -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Group Avatar
          </label>
          <div class="flex items-center space-x-4">
            <div class="relative">
              <div class="w-16 h-16 rounded-full overflow-hidden bg-gray-200 border-2 border-gray-300 flex items-center justify-center">
                <img
                  v-if="groupAvatarPreview"
                  :src="groupAvatarPreview"
                  alt="Group Avatar"
                  class="w-full h-full object-cover"
                />
                <svg v-else class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                </svg>
              </div>
            </div>
            <button
              @click="avatarPicker.selectFile()"
              :disabled="isCreating"
              type="button"
              class="px-3 py-1.5 text-sm text-blue-600 border border-blue-300 rounded-lg hover:bg-blue-50 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {{ groupAvatarPreview ? 'Change' : 'Add Photo' }}
            </button>
            <button
              v-if="groupAvatarPreview"
              @click="removeAvatar"
              :disabled="isCreating"
              type="button"
              class="px-3 py-1.5 text-sm text-red-600 border border-red-300 rounded-lg hover:bg-red-50 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Remove
            </button>
          </div>
        </div>

        <!-- Group Name Input -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Group Name <span class="text-red-500">*</span>
          </label>
          <input
            v-model="groupName"
            type="text"
            :maxlength="100"
            :disabled="isCreating"
            placeholder="Enter group name"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100 disabled:cursor-not-allowed"
            @keydown.escape="handleClose"
          />
          <p class="text-xs text-gray-500 mt-1">{{ groupName.length }}/100 characters</p>
          <p v-if="groupName.trim() === ''" class="text-xs text-red-500 mt-1">Group name is required</p>
        </div>

        <!-- Member Selection -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Select Members <span class="text-red-500">*</span> ({{ selectedMemberIds.size }} selected)
          </label>
          <div v-if="friends.length === 0" class="text-sm text-gray-500 py-4 text-center">
            No friends available
          </div>
          <div v-else class="border border-gray-300 rounded-lg max-h-48 overflow-y-auto">
            <label
              v-for="friend in friends"
              :key="friend.userId"
              class="flex items-center px-4 py-3 hover:bg-gray-50 cursor-pointer border-b border-gray-100 last:border-b-0"
            >
              <input
                type="checkbox"
                :checked="selectedMemberIds.has(friend.userId)"
                @change="toggleMember(friend.userId)"
                :disabled="isCreating"
                class="w-4 h-4 text-blue-500 border-gray-300 rounded focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
              />
              <img
                :src="getFriendAvatar(friend)"
                @error="onImageError"
                class="w-8 h-8 rounded-full ml-3 object-cover"
              />
              <span class="ml-3 text-sm">{{ getFriendDisplayName(friend) }}</span>
            </label>
          </div>
          <p v-if="selectedMemberIds.size === 0" class="text-xs text-red-500 mt-1">
            Please select at least 1 member
          </p>
        </div>

        <!-- Loading/Error State -->
        <div v-if="isCreating" class="mt-4 p-3 bg-blue-50 rounded-lg">
          <p class="text-sm text-blue-800">
            Creating group... Waiting for confirmation.
          </p>
        </div>
        <div v-if="errorMessage" class="mt-4 p-3 bg-red-50 rounded-lg">
          <p class="text-sm text-red-800">{{ errorMessage }}</p>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200">
        <button
          @click="handleClose"
          :disabled="isCreating"
          class="px-4 py-2 text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Cancel
        </button>
        <button
          @click="handleCreate"
          :disabled="!isFormValid || isCreating"
          class="px-4 py-2 text-white bg-blue-500 rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ isCreating ? 'Creating...' : 'Create' }}
        </button>
      </div>
    </div>

    <!-- Avatar Preview Dialog -->
    <AvatarPreviewDialog
      :is-open="avatarPicker.showPreviewDialog.value"
      :preview-url="avatarPicker.avatarPreview.value"
      :is-uploading="avatarPicker.isUploading.value"
      title="Preview Group Avatar"
      @close="avatarPicker.closePreview()"
      @confirm="confirmAvatar"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { RelationUser } from '../../types/relations';
import { useGroupActions } from '../../composables/chat/useGroupActions';
import { useAvatarPicker } from '../../composables/useAvatarPicker';
import AvatarPreviewDialog from '../common/AvatarPreviewDialog.vue';
import defaultAvatarUrl from '../../assets/default-avatar.svg';

defineOptions({
  name: 'CreateGroupDialog',
});

interface Props {
  isOpen: boolean;
  friends: RelationUser[];
  currentUserId: string;
}

const props = defineProps<Props>();

interface Emits {
  (e: 'close'): void;
  (e: 'success', groupId: string): void;
}

const emit = defineEmits<Emits>();

const { createGroup, updateGroupAvatar } = useGroupActions();
const avatarPicker = useAvatarPicker();

// Form state
const groupName = ref('');
const selectedMemberIds = ref(new Set<string>());
const isCreating = ref(false);
const errorMessage = ref('');
const groupAvatarPreview = ref<string>('');
const pendingAvatarFile = ref<string | null>(null);
const pendingAvatarCropRatio = ref<number>(0.5);

// Computed
const isFormValid = computed(() => {
  return groupName.value.trim().length > 0 &&
         groupName.value.trim().length <= 100 &&
         selectedMemberIds.value.size > 0;
});

// Methods
function toggleMember(userId: string) {
  if (selectedMemberIds.value.has(userId)) {
    selectedMemberIds.value.delete(userId);
  } else {
    selectedMemberIds.value.add(userId);
  }
  // Trigger reactivity
  selectedMemberIds.value = new Set(selectedMemberIds.value);
}

function getFriendDisplayName(friend: RelationUser): string {
  return friend.remarkName || friend.nickName || 'Unknown';
}

function getFriendAvatar(friend: RelationUser): string {
  if (!friend.avatar) return defaultAvatarUrl;
  if (friend.avatar.startsWith('http://') || friend.avatar.startsWith('https://')) {
    return friend.avatar;
  }
  return `asset://localhost/${friend.avatar}`;
}

function onImageError(event: Event) {
  const img = event.target as HTMLImageElement;
  img.src = defaultAvatarUrl;
}

function resetForm() {
  groupName.value = '';
  selectedMemberIds.value.clear();
  selectedMemberIds.value = new Set();
  errorMessage.value = '';
  isCreating.value = false;
  groupAvatarPreview.value = '';
  pendingAvatarFile.value = null;
  pendingAvatarCropRatio.value = 0.5;
  avatarPicker.reset();
}

function confirmAvatar(cropRatio: number) {
  // Store file path and crop ratio for later upload after group creation
  if (avatarPicker.selectedFile.value) {
    pendingAvatarFile.value = avatarPicker.selectedFile.value;
    pendingAvatarCropRatio.value = cropRatio;
    groupAvatarPreview.value = avatarPicker.avatarPreview.value;
    avatarPicker.closePreview();
  }
}

function removeAvatar() {
  groupAvatarPreview.value = '';
  pendingAvatarFile.value = null;
  pendingAvatarCropRatio.value = 0.5;
}

function handleClose() {
  if (isCreating.value) return;
  resetForm();
  emit('close');
}

async function handleCreate() {
  if (!isFormValid.value || isCreating.value) return;

  isCreating.value = true;
  errorMessage.value = '';

  try {
    // Include creator's userId in memberIds (ensure no duplicates)
    const memberIds = Array.from(selectedMemberIds.value);
    if (!memberIds.includes(props.currentUserId)) {
      memberIds.push(props.currentUserId);
    }

    const groupId = await createGroup({
      senderId: props.currentUserId,
      groupName: groupName.value.trim(),
      memberIds,
    });

    console.log('[CreateGroupDialog] Group created:', groupId);

    // If avatar was selected, upload and update the group avatar
    if (pendingAvatarFile.value) {
      try {
        await updateGroupAvatar(groupId, pendingAvatarFile.value, pendingAvatarCropRatio.value);
        console.log('[CreateGroupDialog] Group avatar updated');
      } catch (avatarError) {
        console.error('[CreateGroupDialog] Failed to set group avatar:', avatarError);
        // Don't fail the whole operation, group was created successfully
      }
    }

    // Keep dialog open with loading state
    // It will close automatically when WebSocket push event arrives
    // and parent component calls handleClose
    emit('success', groupId);
  } catch (error) {
    console.error('[CreateGroupDialog] Failed to create group:', error);
    errorMessage.value = error instanceof Error ? error.message : 'Failed to create group. Please try again.';
    isCreating.value = false;
  }
}

// Watch isOpen to reset form when dialog opens
watch(() => props.isOpen, (newValue) => {
  if (newValue) {
    resetForm();
  }
});

// Handle escape key
function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape' && props.isOpen && !isCreating.value) {
    handleClose();
  }
}

// Add event listener for escape key
if (typeof window !== 'undefined') {
  window.addEventListener('keydown', handleKeyDown);
}
</script>

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
          :disabled="isSaving"
          class="text-gray-400 hover:text-gray-600 disabled:opacity-50"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="px-6 py-4">
        <!-- Group Name Input -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Group Name <span class="text-red-500">*</span>
          </label>
          <input
            v-model="groupName"
            type="text"
            :maxlength="100"
            :disabled="isSaving"
            placeholder="Enter group name"
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100 disabled:cursor-not-allowed"
          />
          <p class="text-xs text-gray-500 mt-1">{{ groupName.length }}/100 characters</p>
          <p v-if="groupName.trim() === ''" class="text-xs text-red-500 mt-1">
            Group name is required
          </p>
        </div>

        <!-- Error State -->
        <div v-if="errorMessage" class="mt-4 p-3 bg-red-50 rounded-lg">
          <p class="text-sm text-red-800">{{ errorMessage }}</p>
        </div>

        <!-- Success State -->
        <div v-if="successMessage" class="mt-4 p-3 bg-green-50 rounded-lg">
          <p class="text-sm text-green-800">{{ successMessage }}</p>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200">
        <button
          @click="handleClose"
          :disabled="isSaving"
          class="px-4 py-2 text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Cancel
        </button>
        <button
          @click="handleSave"
          :disabled="!isFormValid || isSaving"
          class="px-4 py-2 text-white bg-blue-500 rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ isSaving ? 'Saving...' : 'Save' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useGroupActions } from '../../composables/chat/useGroupActions';

defineOptions({
  name: 'EditGroupDialog',
});

interface Props {
  isOpen: boolean;
  groupId: string;
  groupName: string;
  currentUserId: string;
}

const props = defineProps<Props>();

interface Emits {
  (e: 'close'): void;
  (e: 'success'): void;
}

const emit = defineEmits<Emits>();

const { updateGroupName } = useGroupActions();

// State
const groupName = ref('');
const isSaving = ref(false);
const errorMessage = ref('');
const successMessage = ref('');

// Computed
const isFormValid = computed(() => {
  return groupName.value.trim().length > 0 &&
         groupName.value.trim().length <= 100 &&
         groupName.value.trim() !== props.groupName;
});

// Methods
function resetState() {
  groupName.value = props.groupName || '';
  errorMessage.value = '';
  successMessage.value = '';
  isSaving.value = false;
}

function handleClose() {
  if (isSaving.value) return;
  resetState();
  emit('close');
}

async function handleSave() {
  if (!isFormValid.value || isSaving.value) return;

  isSaving.value = true;
  errorMessage.value = '';
  successMessage.value = '';

  try {
    await updateGroupName(props.groupId, props.currentUserId, groupName.value.trim());
    successMessage.value = 'Group name updated successfully';
    emit('success');

    // Close dialog after a short delay
    setTimeout(() => {
      handleClose();
    }, 1000);
  } catch (error) {
    console.error('[EditGroupDialog] Failed to update group name:', error);
    errorMessage.value = error instanceof Error ? error.message : 'Failed to update group name';
    isSaving.value = false;
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

// Update local state when props change
watch(
  () => props.groupName,
  (newValue) => {
    if (props.isOpen && !isSaving.value) {
      groupName.value = newValue || '';
    }
  }
);
</script>

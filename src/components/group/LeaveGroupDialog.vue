<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center">
    <!-- Backdrop -->
    <div class="absolute inset-0 bg-black/50" @click="handleClose"></div>

    <!-- Dialog -->
    <div class="relative bg-white rounded-lg shadow-xl w-full max-w-sm mx-4">
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200">
        <h2 class="text-xl font-semibold text-red-600">Leave Group</h2>
        <button
          @click="handleClose"
          :disabled="isLeaving"
          class="text-gray-400 hover:text-gray-600 disabled:opacity-50"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="px-6 py-4">
        <p class="text-gray-600">
          Are you sure you want to leave
          <span class="font-semibold">{{ groupName }}</span>?
        </p>
        <p class="text-sm text-gray-500 mt-2">
          You will no longer receive messages from this group and will need to be invited again to rejoin.
        </p>

        <!-- Error State -->
        <div v-if="errorMessage" class="mt-4 p-3 bg-red-50 rounded-lg">
          <p class="text-sm text-red-800">{{ errorMessage }}</p>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200">
        <button
          @click="handleClose"
          :disabled="isLeaving"
          class="px-4 py-2 text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Cancel
        </button>
        <button
          @click="handleLeave"
          :disabled="isLeaving"
          class="px-4 py-2 text-white bg-red-600 rounded-lg hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ isLeaving ? 'Leaving...' : 'Leave Group' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useGroupActions } from '../../composables/chat/useGroupActions';

defineOptions({
  name: 'LeaveGroupDialog',
});

interface Props {
  isOpen: boolean;
  groupId: string;
  groupName: string;
}

const props = defineProps<Props>();

interface Emits {
  (e: 'close'): void;
  (e: 'success'): void;
}

const emit = defineEmits<Emits>();

const { leaveGroup } = useGroupActions();

// State
const isLeaving = ref(false);
const errorMessage = ref('');

// Methods
function resetState() {
  errorMessage.value = '';
  isLeaving.value = false;
}

function handleClose() {
  if (isLeaving.value) return;
  resetState();
  emit('close');
}

async function handleLeave() {
  if (isLeaving.value) return;

  isLeaving.value = true;
  errorMessage.value = '';

  try {
    await leaveGroup(props.groupId);
    // Reset state before emitting events so handleClose() won't early return
    resetState();
    emit('success');
    emit('close');
  } catch (error) {
    console.error('[LeaveGroupDialog] Failed to leave group:', error);
    errorMessage.value = error instanceof Error ? error.message : 'Failed to leave group';
    isLeaving.value = false;
  }
}

// Reset state when dialog opens
watch(
  () => props.isOpen,
  (newValue) => {
    if (newValue) {
      resetState();
    }
  }
);
</script>

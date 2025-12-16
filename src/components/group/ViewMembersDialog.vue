<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center">
    <!-- Backdrop -->
    <div class="absolute inset-0 bg-black/50" @click="handleClose"></div>

    <!-- Dialog -->
    <div class="relative bg-white rounded-lg shadow-xl w-full max-w-md mx-4">
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200">
        <h2 class="text-xl font-semibold">
          Group Members
          <span v-if="!isLoading" class="text-base font-normal text-gray-500">
            ({{ members.length }})
          </span>
        </h2>
        <button
          @click="handleClose"
          class="text-gray-400 hover:text-gray-600"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="px-6 py-4 max-h-96 overflow-y-auto">
        <!-- Loading State -->
        <div v-if="isLoading" class="text-center py-8 text-gray-500">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto mb-2"></div>
          Loading members...
        </div>

        <!-- Error State -->
        <div v-else-if="errorMessage" class="text-center py-8">
          <p class="text-red-600">{{ errorMessage }}</p>
          <button
            @click="loadMembers"
            class="mt-4 px-4 py-2 text-blue-500 hover:text-blue-600"
          >
            Retry
          </button>
        </div>

        <!-- Empty State -->
        <div v-else-if="members.length === 0" class="text-center py-8 text-gray-500">
          No members found
        </div>

        <!-- Members List -->
        <div v-else class="space-y-2">
          <div
            v-for="member in members"
            :key="member.userId"
            class="flex items-center px-4 py-3 bg-gray-50 rounded-lg"
          >
            <img
              :src="getMemberAvatar(member)"
              @error="onImageError"
              class="w-10 h-10 rounded-full object-cover"
            />
            <div class="ml-3">
              <div class="font-medium text-gray-900">{{ member.name }}</div>
              <div class="text-xs text-gray-500">{{ member.userId }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200">
        <button
          @click="handleClose"
          class="px-4 py-2 text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50"
        >
          Close
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { GroupMemberData } from '../../types/group';
import { useGroupActions } from '../../composables/chat/useGroupActions';
import defaultAvatarUrl from '../../assets/default-avatar.svg';

defineOptions({
  name: 'ViewMembersDialog',
});

interface Props {
  isOpen: boolean;
  groupId: string;
}

const props = defineProps<Props>();

interface Emits {
  (e: 'close'): void;
}

const emit = defineEmits<Emits>();

const { getGroupMembers } = useGroupActions();

// State
const members = ref<GroupMemberData[]>([]);
const isLoading = ref(false);
const errorMessage = ref('');

// Methods
function getMemberAvatar(member: GroupMemberData): string {
  if (!member.avatar) return defaultAvatarUrl;
  if (member.avatar.startsWith('http://') || member.avatar.startsWith('https://')) {
    return member.avatar;
  }
  return `asset://localhost/${member.avatar}`;
}

function onImageError(event: Event) {
  const img = event.target as HTMLImageElement;
  img.src = defaultAvatarUrl;
}

function resetState() {
  members.value = [];
  errorMessage.value = '';
  isLoading.value = false;
}

function handleClose() {
  resetState();
  emit('close');
}

async function loadMembers() {
  if (!props.groupId) return;

  isLoading.value = true;
  errorMessage.value = '';

  try {
    members.value = await getGroupMembers(props.groupId);
  } catch (error) {
    console.error('[ViewMembersDialog] Failed to load members:', error);
    errorMessage.value = error instanceof Error ? error.message : 'Failed to load members';
  } finally {
    isLoading.value = false;
  }
}

// Load members when dialog opens
watch(
  () => props.isOpen,
  (newValue) => {
    if (newValue) {
      resetState();
      loadMembers();
    }
  }
);
</script>

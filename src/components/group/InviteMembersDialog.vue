<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center">
    <!-- Backdrop -->
    <div class="absolute inset-0 bg-black/50" @click="handleClose"></div>

    <!-- Dialog -->
    <div class="relative bg-white rounded-lg shadow-xl w-full max-w-md mx-4">
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200">
        <h2 class="text-xl font-semibold">Invite Members</h2>
        <button
          @click="handleClose"
          :disabled="isInviting"
          class="text-gray-400 hover:text-gray-600 disabled:opacity-50"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="px-6 py-4 max-h-96 overflow-y-auto">
        <!-- Loading current members -->
        <div v-if="isLoadingMembers" class="text-center py-8 text-gray-500">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto mb-2"></div>
          Loading members...
        </div>

        <!-- Member Selection -->
        <div v-else>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Select Friends to Invite ({{ selectedMemberIds.size }} selected)
          </label>

          <div v-if="availableFriends.length === 0" class="text-sm text-gray-500 py-4 text-center">
            No friends available to invite
          </div>
          <div v-else class="border border-gray-300 rounded-lg max-h-64 overflow-y-auto">
            <label
              v-for="friend in availableFriends"
              :key="friend.userId"
              class="flex items-center px-4 py-3 hover:bg-gray-50 cursor-pointer border-b border-gray-100 last:border-b-0"
            >
              <input
                type="checkbox"
                :checked="selectedMemberIds.has(friend.userId)"
                @change="toggleMember(friend.userId)"
                :disabled="isInviting"
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
        </div>

        <!-- Error State -->
        <div v-if="errorMessage" class="mt-4 p-3 bg-red-50 rounded-lg">
          <p class="text-sm text-red-800">{{ errorMessage }}</p>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200">
        <button
          @click="handleClose"
          :disabled="isInviting"
          class="px-4 py-2 text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Cancel
        </button>
        <button
          @click="handleInvite"
          :disabled="selectedMemberIds.size === 0 || isInviting"
          class="px-4 py-2 text-white bg-blue-500 rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ isInviting ? 'Inviting...' : 'Invite' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { RelationUser } from '../../types/relations';
import type { GroupMemberData } from '../../types/group';
import { useGroupActions } from '../../composables/chat/useGroupActions';
import defaultAvatarUrl from '../../assets/default-avatar.svg';

defineOptions({
  name: 'InviteMembersDialog',
});

interface Props {
  isOpen: boolean;
  groupId: string;
  groupName: string;
  groupAvatar: string | null;
  friends: RelationUser[];
  currentUserId: string;
}

const props = defineProps<Props>();

interface Emits {
  (e: 'close'): void;
  (e: 'success'): void;
}

const emit = defineEmits<Emits>();

const { inviteMembers, getGroupMembers } = useGroupActions();

// State
const selectedMemberIds = ref(new Set<string>());
const currentMembers = ref<GroupMemberData[]>([]);
const isLoadingMembers = ref(false);
const isInviting = ref(false);
const errorMessage = ref('');

// Computed: Friends not already in the group
const availableFriends = computed(() => {
  const memberIds = new Set(currentMembers.value.map((m) => m.userId));
  return props.friends.filter((f) => !memberIds.has(f.userId));
});

// Methods
function toggleMember(userId: string) {
  if (selectedMemberIds.value.has(userId)) {
    selectedMemberIds.value.delete(userId);
  } else {
    selectedMemberIds.value.add(userId);
  }
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

function resetState() {
  selectedMemberIds.value.clear();
  selectedMemberIds.value = new Set();
  currentMembers.value = [];
  errorMessage.value = '';
  isInviting.value = false;
  isLoadingMembers.value = false;
}

function handleClose() {
  if (isInviting.value) return;
  resetState();
  emit('close');
}

async function loadCurrentMembers() {
  if (!props.groupId) return;

  isLoadingMembers.value = true;
  errorMessage.value = '';

  try {
    currentMembers.value = await getGroupMembers(props.groupId);
  } catch (error) {
    console.error('[InviteMembersDialog] Failed to load members:', error);
    errorMessage.value = 'Failed to load current members';
  } finally {
    isLoadingMembers.value = false;
  }
}

async function handleInvite() {
  if (selectedMemberIds.value.size === 0 || isInviting.value) return;

  isInviting.value = true;
  errorMessage.value = '';

  try {
    await inviteMembers(
      props.groupId,
      props.currentUserId,
      Array.from(selectedMemberIds.value),
      props.groupName,
      props.groupAvatar
    );
    emit('success');
    handleClose();
  } catch (error) {
    console.error('[InviteMembersDialog] Failed to invite members:', error);
    errorMessage.value = error instanceof Error ? error.message : 'Failed to invite members';
    isInviting.value = false;
  }
}

// Load members when dialog opens
watch(
  () => props.isOpen,
  (newValue) => {
    if (newValue) {
      resetState();
      loadCurrentMembers();
    }
  }
);
</script>

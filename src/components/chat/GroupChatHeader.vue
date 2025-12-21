<template>
  <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center justify-between">
    <div class="flex items-center gap-3">
      <img
        :src="groupAvatarUrl"
        @error="onImageError"
        class="w-10 h-10 rounded-full object-cover"
      />
      <div>
        <h2 class="font-medium">{{ groupDisplayName }}</h2>
      </div>
    </div>

    <!-- Menu Button -->
    <button
      @click="toggleMenu"
      class="px-3 py-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
      title="Group menu"
    >
      <span class="text-xl">⋮</span>
    </button>

    <!-- Menu Dropdown -->
    <Teleport to="body">
      <div
        v-if="isMenuOpen"
        @click.self="closeMenu"
        class="fixed inset-0 z-40"
      />
    </Teleport>

    <div
      v-if="isMenuOpen"
      class="absolute top-16 right-6 bg-white border border-gray-200 rounded-lg shadow-lg z-50 min-w-48"
    >
      <button
        @click="emitInviteMembers"
        class="w-full text-left px-4 py-2 hover:bg-gray-50 text-sm font-medium text-gray-700 first:rounded-t-lg transition-colors"
      >
        Invite Members
      </button>
      <button
        @click="emitViewMembers"
        class="w-full text-left px-4 py-2 hover:bg-gray-50 text-sm font-medium text-gray-700 transition-colors"
      >
        View Members
      </button>
      <button
        @click="emitEditGroup"
        class="w-full text-left px-4 py-2 hover:bg-gray-50 text-sm font-medium text-gray-700 transition-colors"
      >
        Edit Group Info
      </button>
      <button
        @click="emitLeaveGroup"
        class="w-full text-left px-4 py-2 hover:bg-red-50 text-sm font-medium text-red-600 last:rounded-b-lg transition-colors"
      >
        Leave Group
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import defaultAvatarUrl from '../../assets/default-avatar.svg';

interface Props {
  groupName: string;
  groupAvatar?: string;
  memberCount?: number;
}

const props = withDefaults(defineProps<Props>(), {
  memberCount: 0,
});

const emit = defineEmits<{
  (e: 'invite-members'): void;
  (e: 'view-members'): void;
  (e: 'edit-group'): void;
  (e: 'leave-group'): void;
}>();

const isMenuOpen = ref(false);

const groupAvatarUrl = computed(() => {
  if (!props.groupAvatar) return defaultAvatarUrl;
  if (props.groupAvatar.startsWith('http://') || props.groupAvatar.startsWith('https://')) {
    return props.groupAvatar;
  }
  return `asset://localhost/${props.groupAvatar}`;
});

const groupDisplayName = computed(() => {
  return `${props.groupName}(${props.memberCount})`;
});

function toggleMenu() {
  isMenuOpen.value = !isMenuOpen.value;
}

function closeMenu() {
  isMenuOpen.value = false;
}

function emitInviteMembers() {
  closeMenu();
  emit('invite-members');
}

function emitViewMembers() {
  closeMenu();
  emit('view-members');
}

function emitEditGroup() {
  closeMenu();
  emit('edit-group');
}

function emitLeaveGroup() {
  closeMenu();
  emit('leave-group');
}

function onImageError() {
  // Fallback to default avatar is handled by src binding
}
</script>

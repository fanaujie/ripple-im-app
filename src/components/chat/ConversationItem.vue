<template>
  <div
    class="flex items-center gap-3 px-6 py-4 hover:bg-gray-50 cursor-pointer transition-colors"
    @click="handleClick"
  >
    <!-- Avatar with optional Bot badge -->
    <div class="relative flex-shrink-0">
      <img
        :src="avatarUrl"
        @error="onImageError"
        alt="Avatar"
        class="w-12 h-12 rounded-full object-cover"
      />
      <!-- Bot badge -->
      <div
        v-if="isBot"
        class="absolute -bottom-0.5 -right-0.5 w-5 h-5 bg-purple-500 rounded-full flex items-center justify-center"
      >
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-3 h-3 text-white">
          <path d="M14 6H6v8h8V6Z" />
          <path fill-rule="evenodd" d="M9.25 3V1.75a.75.75 0 0 1 1.5 0V3h1.5V1.75a.75.75 0 0 1 1.5 0V3h.5A2.75 2.75 0 0 1 17 5.75v.5h1.25a.75.75 0 0 1 0 1.5H17v1.5h1.25a.75.75 0 0 1 0 1.5H17v1.5h1.25a.75.75 0 0 1 0 1.5H17v.5A2.75 2.75 0 0 1 14.25 17h-.5v1.25a.75.75 0 0 1-1.5 0V17h-1.5v1.25a.75.75 0 0 1-1.5 0V17h-1.5v1.25a.75.75 0 0 1-1.5 0V17h-.5A2.75 2.75 0 0 1 3 14.25v-.5H1.75a.75.75 0 0 1 0-1.5H3v-1.5H1.75a.75.75 0 0 1 0-1.5H3v-1.5H1.75a.75.75 0 0 1 0-1.5H3v-.5A2.75 2.75 0 0 1 5.75 3h.5V1.75a.75.75 0 0 1 1.5 0V3h1.5ZM4.5 5.75c0-.69.56-1.25 1.25-1.25h8.5c.69 0 1.25.56 1.25 1.25v8.5c0 .69-.56 1.25-1.25 1.25h-8.5c-.69 0-1.25-.56-1.25-1.25v-8.5Z" clip-rule="evenodd" />
        </svg>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 min-w-0">
      <div class="flex items-center justify-between mb-1">
        <!-- Name -->
        <div class="font-medium text-gray-900 truncate">
          {{ displayName }}
        </div>

        <!-- Time -->
        <div class="text-xs text-gray-500 flex-shrink-0 ml-2">
          {{ formattedTime }}
        </div>
      </div>

      <div class="flex items-center justify-between">
        <!-- Last Message Preview -->
        <div class="text-sm text-gray-500 truncate flex-1">
          {{ conversation.lastMessage || '暂无消息' }}
        </div>

        <!-- Unread Badge -->
        <div
          v-if="conversation.unreadCount > 0"
          class="ml-2 flex-shrink-0 bg-blue-500 text-white text-xs font-medium rounded-full w-5 h-5 flex items-center justify-center"
        >
          {{ conversation.unreadCount > 99 ? '99+' : conversation.unreadCount }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { ConversationDisplay } from '../../types/chat';
import { getConversationDisplayName, getConversationAvatar, isGroupChat } from '../../types/chat';
import { formatConversationTime } from '../../utils/dateFormat';
import { useGroupMembersCache } from '../../composables/chat/useGroupMembersCache';
import defaultAvatarUrl from '../../assets/default-avatar.svg';
import defaultBotAvatarUrl from '../../assets/default-bot-avatar.svg';

const props = defineProps<{
  conversation: ConversationDisplay;
  isBot?: boolean;
}>();

const emit = defineEmits<{
  click: [conversation: ConversationDisplay];
}>();

const { getGroupMemberCount } = useGroupMembersCache();

const displayName = computed(() => {
  const baseName = getConversationDisplayName(props.conversation);

  // For group chats, append member count if available
  if (isGroupChat(props.conversation) && props.conversation.groupId) {
    const memberCount = getGroupMemberCount(props.conversation.groupId);
    if (memberCount !== undefined) {
      return `${baseName}(${memberCount})`;
    }
  }

  return baseName;
});

const fallbackAvatarUrl = computed(() => props.isBot ? defaultBotAvatarUrl : defaultAvatarUrl);

const avatarUrl = computed(() => {
  const avatar = getConversationAvatar(props.conversation);
  if (!avatar) return fallbackAvatarUrl.value;
  if (avatar.startsWith('http://') || avatar.startsWith('https://')) {
    return avatar;
  }
  return `asset://localhost/${avatar}`;
});

const formattedTime = computed(() => {
  if (!props.conversation.lastMessageTimestamp) {
    return '';
  }
  return formatConversationTime(props.conversation.lastMessageTimestamp);
});

function onImageError(event: Event) {
  const img = event.target as HTMLImageElement;
  img.src = fallbackAvatarUrl.value;
}

function handleClick() {
  emit('click', props.conversation);
}
</script>

<template>
  <div class="h-full flex">
    <!-- Left: Conversation List -->
    <div class="w-80 border-r border-gray-200 flex flex-col bg-white">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-200">
        <h1 class="text-2xl font-semibold">Chat</h1>
      </div>

      <!-- Search -->
      <div class="px-4 py-3">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search conversations..."
          class="w-full px-4 py-2 bg-gray-50 border border-gray-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>

      <!-- Conversation List -->
      <div class="flex-1 overflow-y-auto">
        <div v-if="loading" class="p-8 text-center text-gray-500">Loading...</div>
        <div v-else-if="filteredConversations.length === 0" class="p-8 text-center text-gray-500">
          No conversations
        </div>
        <ConversationItem
          v-for="conv in filteredConversations"
          :key="conv.conversationId"
          :conversation="conv"
          @click="selectConversation"
        />
      </div>
    </div>

    <!-- Right: Chat Window or Empty State -->
    <div class="flex-1 flex flex-col bg-gray-50">
      <template v-if="selectedConversation || targetUserId">
        <!-- Chat Header -->
        <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center justify-between">
          <div class="flex items-center gap-3">
            <img
              :src="selectedAvatarUrl"
              @error="onImageError"
              class="w-10 h-10 rounded-full object-cover"
            />
            <div class="font-medium">{{ selectedDisplayName }}</div>
          </div>
        </div>

        <!-- Messages Area -->
        <div ref="messagesContainer" @scroll="handleScroll" class="flex-1 overflow-y-auto px-6 py-4 space-y-4">
          <!-- Loading indicator at top -->
          <div v-if="isLoadingOlder" class="text-center py-2">
            <span class="text-gray-500 text-sm">Loading...</span>
          </div>

          <!-- No more messages indicator -->
          <div v-else-if="!hasMoreMessages && currentMessages.length > 0" class="text-center py-2">
            <span class="text-gray-400 text-sm">No more messages</span>
          </div>

          <!-- Messages -->
          <div
            v-for="message in currentMessages"
            :key="message.messageId"
            :class="[
              'flex items-end gap-2',
              message.senderId === currentUserId ? 'justify-end flex-row-reverse' : 'justify-start',
            ]"
          >
            <div
              :class="[
                'max-w-md px-4 py-2 rounded-2xl',
                message.senderId === currentUserId
                  ? 'bg-blue-500 text-white'
                  : 'bg-white text-gray-900',
              ]"
            >
              <div>{{ message.textContent }}</div>
            </div>
            <div class="text-xs text-gray-400 pb-1 whitespace-nowrap">
              {{ formatMessageTime(message.sendTimestamp) }}
            </div>
          </div>
        </div>

        <!-- Input Area -->
        <div class="bg-white border-t border-gray-200 px-6 py-4">
          <div class="flex items-end gap-3">
            <textarea
              v-model="messageInput"
              @keydown.enter.exact.prevent="handleSend"
              placeholder="Type a message..."
              rows="1"
              class="flex-1 px-4 py-2 bg-gray-50 border border-gray-200 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
              style="max-height: 120px"
            ></textarea>
            <button
              @click="handleSend"
              :disabled="!messageInput.trim()"
              class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              Send
            </button>
          </div>
        </div>
      </template>

      <template v-else>
        <div class="flex-1 flex items-center justify-center">
          <EmptyView
            title="Select a conversation"
            description="Choose a conversation from the left to start chatting"
            icon="chat-bubble-left-right"
          />
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { useChatDisplay } from '../composables/chat/useChatDisplay';
import { useRelationsDisplay } from '../composables/useRelationsDisplay';
import { useUserProfileDisplay } from '../composables/useUserProfileDisplay';
import { getConversationDisplayName, getConversationAvatar } from '../types/chat';
import type { ConversationDisplay } from '../types/chat';
import { formatMessageTime } from '../utils/dateFormat';
import ConversationItem from '../components/chat/ConversationItem.vue';
import EmptyView from '../components/views/EmptyView.vue';
import defaultAvatarUrl from '../assets/default-avatar.svg';

defineOptions({
  name: 'ChatView',
});

// Router
const route = useRoute();

// User profile
const { userProfile } = useUserProfileDisplay();
const currentUserId = computed(() => userProfile.value?.userId || '');

// Relations (for peer profiles)
const { relationsMap } = useRelationsDisplay();

// Chat display (auto-initializes conversations via onMounted)
const {
  conversations,
  loading,
  loadLatestMessages,
  loadOlderMessages,
  sendMessage,
  markConversationRead,
  getConversationMessages,
  setActiveConversation,
} = useChatDisplay(relationsMap);

// Selected conversation
const selectedConversation = ref<ConversationDisplay | null>(null);
const searchQuery = ref('');
const messageInput = ref('');
const messagesContainer = ref<HTMLElement | null>(null);

// Pagination state
const isLoadingOlder = ref(false);
const hasMoreMessages = ref(true);

// Target user for starting new conversation
const targetUserId = ref<string | null>(null);

// Target user info from relations
const targetUserInfo = computed(() => {
  if (!targetUserId.value) return null;
  return relationsMap.value.get(targetUserId.value) || null;
});

// Filtered conversations
const filteredConversations = computed(() => {
  if (!searchQuery.value.trim()) {
    return conversations.value;
  }
  const query = searchQuery.value.toLowerCase();
  return conversations.value.filter((conv) => {
    const name = getConversationDisplayName(conv).toLowerCase();
    return name.includes(query);
  });
});

// Current messages for selected conversation
const currentMessages = computed(() => {
  if (!selectedConversation.value) {
    return [];
  }
  return getConversationMessages(selectedConversation.value.conversationId);
});

// Selected conversation display
const selectedDisplayName = computed(() => {
  if (selectedConversation.value) {
    return getConversationDisplayName(selectedConversation.value);
  }
  // Use target user info if starting new conversation
  if (targetUserInfo.value) {
    return targetUserInfo.value.remarkName || targetUserInfo.value.nickName || 'Unknown';
  }
  return '';
});

const selectedAvatarUrl = computed(() => {
  // Use conversation avatar if available
  if (selectedConversation.value) {
    const avatar = getConversationAvatar(selectedConversation.value);
    if (!avatar) return defaultAvatarUrl;
    if (avatar.startsWith('http://') || avatar.startsWith('https://')) {
      return avatar;
    }
    return `asset://localhost/${avatar}`;
  }

  // Use target user avatar if starting new conversation
  if (targetUserInfo.value?.avatar) {
    const avatar = targetUserInfo.value.avatar;
    if (avatar.startsWith('http://') || avatar.startsWith('https://')) {
      return avatar;
    }
    return `asset://localhost/${avatar}`;
  }

  return defaultAvatarUrl;
});

// Select conversation
async function selectConversation(conversation: ConversationDisplay) {
  selectedConversation.value = conversation;

  // Update active conversation for message filtering
  setActiveConversation(conversation.conversationId);

  // Reset pagination state
  hasMoreMessages.value = true;

  // Load latest 50 messages
  try {
    await loadLatestMessages(conversation.conversationId, 50);

    // Scroll to bottom
    await nextTick();
    scrollToBottom();

    // Mark as read
    const messages = currentMessages.value;
    if (messages.length > 0) {
      const lastMessage = messages[messages.length - 1];
      await markConversationRead(conversation.conversationId, lastMessage.messageId);
    }
  } catch (error) {
    console.error('Failed to select conversation:', error);
  }
}

// Send message
async function handleSend() {
  if (!messageInput.value.trim() || !currentUserId.value) {
    return;
  }

  // Must have either a selected conversation or a target user
  if (!selectedConversation.value && !targetUserId.value) {
    return;
  }

  const content = messageInput.value.trim();
  // Use empty string for conversationId when starting new conversation
  const conversationId = selectedConversation.value?.conversationId || '';
  const receiverId = selectedConversation.value?.peerId || targetUserId.value || '';

  try {
    await sendMessage(currentUserId.value, conversationId, receiverId, content);
    messageInput.value = '';

    // Clear targetUserId after sending first message (conversation will be created server-side)
    if (targetUserId.value) {
      console.log('First message sent to user:', targetUserId.value, '- conversation will be created');
      targetUserId.value = null;
    }

    // Auto scroll on send
    await nextTick();
    scrollToBottom();
  } catch (error) {
    console.error('Failed to send message:', error);
  }
}

// Scroll to bottom
function scrollToBottom() {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
  }
}

// Handle scroll event - load older messages when scrolling to top
function handleScroll(event: Event) {
  if (isLoadingOlder.value || !hasMoreMessages.value || !selectedConversation.value) {
    return;
  }

  const container = event.target as HTMLElement;
  const scrollTop = container.scrollTop;

  // When scrolled near top (within 50px), load older messages
  if (scrollTop < 50) {
    loadOlderMessagesWithScroll();
  }
}

// Load older messages and preserve scroll position
async function loadOlderMessagesWithScroll() {
  if (!selectedConversation.value || isLoadingOlder.value || !hasMoreMessages.value) {
    return;
  }

  isLoadingOlder.value = true;

  try {
    const container = messagesContainer.value;
    if (!container) return;

    // Record current scroll position
    const oldScrollHeight = container.scrollHeight;
    const oldScrollTop = container.scrollTop;

    // Load older messages
    const hasMore = await loadOlderMessages(selectedConversation.value.conversationId, 50);
    hasMoreMessages.value = hasMore;

    // Wait for DOM to update
    await nextTick();

    // Restore scroll position (keep user viewing the same message)
    const newScrollHeight = container.scrollHeight;
    const scrollDiff = newScrollHeight - oldScrollHeight;
    container.scrollTop = oldScrollTop + scrollDiff;
  } catch (error) {
    console.error('Failed to load older messages:', error);
  } finally {
    isLoadingOlder.value = false;
  }
}

// Smart auto-scroll: only scroll to bottom when user is already near the bottom
watch(currentMessages, async (newMessages, oldMessages) => {
  // Only auto-scroll if messages were added (not removed or cleared)
  if (!oldMessages || newMessages.length <= oldMessages.length) {
    return;
  }

  await nextTick();

  const container = messagesContainer.value;
  if (!container) return;

  // Check if user is near bottom (within 100px of the bottom)
  const scrollTop = container.scrollTop;
  const scrollHeight = container.scrollHeight;
  const clientHeight = container.clientHeight;
  const distanceFromBottom = scrollHeight - (scrollTop + clientHeight);

  // Auto-scroll only if user is already near the bottom
  if (distanceFromBottom < 100) {
    scrollToBottom();
  }
});

function onImageError(event: Event) {
  const img = event.target as HTMLImageElement;
  img.src = defaultAvatarUrl;
}

// Conversations are automatically initialized by useChatDisplay on mount

// Handle userId from route query parameter
function handleUserIdNavigation(userId: string) {
  if (!userId) return;

  // If still loading, set targetUserId and let watcher handle it when ready
  if (loading.value) {
    targetUserId.value = userId;
    return;
  }

  // Find conversation with matching peerId
  const targetConversation = conversations.value.find(conv => conv.peerId === userId);

  if (targetConversation) {
    // Auto-select the conversation
    selectConversation(targetConversation);
  } else {
    // No existing conversation found - set targetUserId to allow starting new conversation
    console.log('No existing conversation with user:', userId, '- allowing user to start new chat');
    targetUserId.value = userId;
  }
}

// Watch conversations array - handle pending userId when conversations load
watch(conversations, () => {
  const userId = route.query.userId as string | undefined;
  if (userId && targetUserId.value === userId && !selectedConversation.value) {
    // Retry finding conversation now that conversations are loaded
    const targetConversation = conversations.value.find(conv => conv.peerId === userId);
    if (targetConversation) {
      selectConversation(targetConversation);
    }
  }
}, { deep: true });

// Watch route changes for navigation when component is cached by KeepAlive
watch(() => route.query.userId, (newUserId) => {
  if (newUserId) {
    handleUserIdNavigation(newUserId as string);
  } else {
    // Clear selection when leaving chat
    selectedConversation.value = null;
    targetUserId.value = null;
    setActiveConversation(null);
  }
});

// Handle initial load
onMounted(() => {
  const userId = route.query.userId as string | undefined;
  if (userId) {
    handleUserIdNavigation(userId);
  }
});
</script>

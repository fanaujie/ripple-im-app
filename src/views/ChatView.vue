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
        <!-- Group Chat Header -->
        <GroupChatHeader
          v-if="isGroupChat && selectedConversation"
          :group-name="selectedDisplayName"
          :group-avatar="getConversationAvatar(selectedConversation) || undefined"
          :member-count="groupMemberCount"
          @invite-members="handleGroupAction('invite-members')"
          @view-members="handleGroupAction('view-members')"
          @edit-group="handleGroupAction('edit-group')"
          @leave-group="handleGroupAction('leave-group')"
        />

        <!-- 1v1 Chat Header -->
        <div v-else class="bg-white border-b border-gray-200 px-6 py-4 flex items-center justify-between">
          <div class="flex items-center gap-3">
            <img
              :src="selectedAvatarUrl"
              @error="onImageError"
              class="w-10 h-10 rounded-full object-cover"
            />
            <div class="font-medium">{{ selectedDisplayName }}</div>
          </div>
        </div>

        <!-- Stranger Message Banner -->
        <div
          v-if="isStrangerConversation"
          class="bg-amber-50 border-b border-amber-200 px-6 py-3"
        >
          <div class="flex items-center justify-between">
            <!-- Left: Message (text only, no icon) -->
            <p class="text-sm text-amber-800">
              This user is not in your friends list
            </p>

            <!-- Right: Action Buttons -->
            <div class="flex items-center gap-3">
              <button
                @click="handleAddFriendFromBanner"
                :disabled="!!bannerActionLoading"
                class="px-4 py-1.5 bg-blue-500 text-white text-sm font-medium rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {{ bannerActionLoading === 'add' ? 'Adding...' : 'Add Friend' }}
              </button>
              <button
                @click="handleBlockUserFromBanner"
                :disabled="!!bannerActionLoading"
                class="px-4 py-1.5 bg-white text-gray-700 text-sm font-medium border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {{ bannerActionLoading === 'block' ? 'Blocking...' : 'Block' }}
              </button>
            </div>
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
          <template v-for="(message, index) in currentMessages" :key="message.messageId">
            <!-- Date Separator -->
            <div v-if="shouldShowDateSeparator(index)" class="flex justify-center py-2">
              <span class="text-xs text-gray-400 bg-gray-100 px-3 py-1 rounded-full">
                {{ formatMessageDate(message.sendTimestamp) }}
              </span>
            </div>

            <!-- Command Message (group notifications) - use Number() to ensure type consistency -->
            <div v-if="Number(message.messageType) === MessageType.GROUP_COMMAND">
              <div class="text-center text-xs text-gray-400 mb-1">
                {{ formatMessageTime(message.sendTimestamp) }}
              </div>
              <CommandMessage :message="message" />
            </div>

            <!-- Regular Message -->
            <div
              v-else
              :class="[
                'flex items-end gap-2',
                message.senderId === currentUserId ? 'justify-end' : 'justify-start',
              ]"
            >
              <!-- Sent messages: timestamp on left, bubble on right -->
              <template v-if="message.senderId === currentUserId">
                <div class="text-xs text-gray-400 pb-1 whitespace-nowrap">
                  {{ formatMessageTime(message.sendTimestamp) }}
                </div>
                <div class="max-w-md px-4 py-2 rounded-2xl bg-blue-500 text-white">
                  <div>{{ message.text }}</div>
                </div>
              </template>

              <!-- Received messages: bubble on left, timestamp on right -->
              <template v-else>
                <div class="max-w-md px-4 py-2 rounded-2xl bg-white text-gray-900">
                  <div>{{ message.text }}</div>
                </div>
                <div class="text-xs text-gray-400 pb-1 whitespace-nowrap">
                  {{ formatMessageTime(message.sendTimestamp) }}
                </div>
              </template>
            </div>
          </template>
        </div>

        <!-- Blocked User Notification (replaces input area) -->
        <div
          v-if="isBlockedConversation"
          class="bg-red-50 border-t border-red-200 px-6 py-4"
        >
          <p class="text-sm text-red-800 text-center">
            This user has been blocked
          </p>
        </div>

        <!-- Input Area (only shown when user is not blocked) -->
        <div v-else class="bg-white border-t border-gray-200 px-6 py-4">
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

    <!-- Group Dialogs -->
    <InviteMembersDialog
      :is-open="isInviteMembersDialogOpen"
      :group-id="selectedConversation?.groupId || ''"
      :group-name="selectedConversation?.name || ''"
      :group-avatar="selectedConversation?.avatar || null"
      :friends="friends"
      :current-user-id="currentUserId"
      @close="closeInviteMembersDialog"
      @success="closeInviteMembersDialog"
    />

    <ViewMembersDialog
      :is-open="isViewMembersDialogOpen"
      :group-id="selectedConversation?.groupId || ''"
      @close="closeViewMembersDialog"
    />

    <EditGroupDialog
      :is-open="isEditGroupDialogOpen"
      :group-id="selectedConversation?.groupId || ''"
      :group-name="selectedDisplayName"
      :group-avatar="selectedConversation?.avatar"
      :current-user-id="currentUserId"
      @close="closeEditGroupDialog"
      @success="closeEditGroupDialog"
    />

    <LeaveGroupDialog
      :is-open="isLeaveGroupDialogOpen"
      :group-id="selectedConversation?.groupId || ''"
      :group-name="selectedDisplayName"
      @close="closeLeaveGroupDialog"
      @success="handleLeaveGroupSuccess"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { useChatDisplay } from '../composables/chat/useChatDisplay';
import { useRelationsDisplay } from '../composables/useRelationsDisplay';
import { useRelationActions } from '../composables/useRelationActions';
import { useUserProfileDisplay } from '../composables/useUserProfileDisplay';
import { getConversationDisplayName, getConversationAvatar } from '../types/chat';
import type { ConversationDisplay } from '../types/chat';
import { MessageType } from '../types/chat';
import { formatMessageTime, formatMessageDate } from '../utils/dateFormat';
import ConversationItem from '../components/chat/ConversationItem.vue';
import EmptyView from '../components/views/EmptyView.vue';
import CommandMessage from '../components/chat/CommandMessage.vue';
import GroupChatHeader from '../components/chat/GroupChatHeader.vue';
import InviteMembersDialog from '../components/group/InviteMembersDialog.vue';
import ViewMembersDialog from '../components/group/ViewMembersDialog.vue';
import EditGroupDialog from '../components/group/EditGroupDialog.vue';
import LeaveGroupDialog from '../components/group/LeaveGroupDialog.vue';
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
const { relationsMap, friends } = useRelationsDisplay();

// Relation actions (for banner actions)
const { addFriend, blockUser } = useRelationActions();

// Group dialog states
const isInviteMembersDialogOpen = ref(false);
const isViewMembersDialogOpen = ref(false);
const isEditGroupDialogOpen = ref(false);
const isLeaveGroupDialogOpen = ref(false);

// Chat display (auto-initializes conversations via onMounted)
const {
  conversations,
  loading,
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

// Stranger banner state
const bannerActionLoading = ref<'add' | 'block' | null>(null);
const bannerDismissedForUser = ref<string | null>(null); // Track which user's banner was dismissed

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

// Check if we should show a date separator before this message
function shouldShowDateSeparator(index: number): boolean {
  if (index === 0) return true; // Always show for first message

  const currentMsg = currentMessages.value[index];
  const prevMsg = currentMessages.value[index - 1];

  // Compare dates using formatted date string
  return formatMessageDate(currentMsg.sendTimestamp) !== formatMessageDate(prevMsg.sendTimestamp);
}

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

// Determine if current conversation is with a stranger
const isStrangerConversation = computed(() => {
  // Get peer ID from selected conversation or target user
  const peerId = selectedConversation.value?.peerId || targetUserId.value;

  // Don't show banner if no peer or self-messaging
  if (!peerId || peerId === currentUserId.value) {
    return false;
  }

  // Hide banner if user has already taken action on this user's banner
  if (bannerDismissedForUser.value === peerId) {
    return false;
  }

  // Hide banner immediately when action is in progress (prevents flicker)
  if (bannerActionLoading.value) {
    return false;
  }

  // Look up peer in relations map
  const peerRelation = relationsMap.value.get(peerId);

  // If not in relations at all, it's a stranger
  if (!peerRelation) {
    return true;
  }

  // Check relation flags
  const FRIEND_FLAG = 0b0001;
  const BLOCKED_FLAG = 0b0010;

  const isFriend = (peerRelation.relationFlags & FRIEND_FLAG) !== 0;
  const isBlocked = (peerRelation.relationFlags & BLOCKED_FLAG) !== 0;

  // Show banner only if not a friend and not blocked
  return !isFriend && !isBlocked;
});

// Determine if current conversation is with a blocked user
const isBlockedConversation = computed(() => {
  // Get peer ID from selected conversation or target user
  const peerId = selectedConversation.value?.peerId || targetUserId.value;

  // Don't show banner if no peer or self-messaging
  if (!peerId || peerId === currentUserId.value) {
    return false;
  }

  // Look up peer in relations map
  const peerRelation = relationsMap.value.get(peerId);

  // If not in relations at all, not blocked
  if (!peerRelation) {
    return false;
  }

  // Check if user is blocked
  const BLOCKED_FLAG = 0b0010;
  const isBlocked = (peerRelation.relationFlags & BLOCKED_FLAG) !== 0;

  return isBlocked;
});

// Determine if current conversation is a group chat
const isGroupChat = computed(() => {
  return !!selectedConversation.value?.groupId;
});

// Get group member count
const groupMemberCount = computed(() => {
  // TODO: This will be populated from group members list
  // For now, return 0 as placeholder
  return 0;
});

// Select conversation
async function selectConversation(conversation: ConversationDisplay) {
  selectedConversation.value = conversation;

  // Update active conversation (clears old messages, loads new ones with optimization)
  try {
    // setActiveConversation returns whether there might be more older messages
    const mightHaveMore = await setActiveConversation(conversation.conversationId);
    hasMoreMessages.value = mightHaveMore;

    // Scroll to bottom
    await nextTick();
    scrollToBottom();

    // Mark as read (with optimization to skip API if no new messages)
    const messages = currentMessages.value;
    if (messages.length > 0) {
      const lastMessage = messages[messages.length - 1];
      // Pass current last_read_message_id for optimization
      await markConversationRead(
        conversation.conversationId,
        lastMessage.messageId,
        conversation.lastReadMessageId
      );
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
  // For group chats: receiverId is null, groupId is set
  // For direct chats: receiverId is set, groupId is null
  const groupId = selectedConversation.value?.groupId || null;
  const receiverId = groupId ? null : (selectedConversation.value?.peerId || targetUserId.value || '');

  try {
    await sendMessage(currentUserId.value, conversationId, receiverId, content, groupId);
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

// Handle add friend from banner
async function handleAddFriendFromBanner() {
  const peerId = selectedConversation.value?.peerId || targetUserId.value;

  if (!peerId || bannerActionLoading.value) {
    return;
  }

  // Dismiss banner immediately for this user (prevents flicker)
  bannerDismissedForUser.value = peerId;
  bannerActionLoading.value = 'add';

  try {
    await addFriend(peerId);
    console.log('Successfully added friend from chat banner:', peerId);
    // Banner will stay hidden due to bannerDismissedForUser
  } catch (error) {
    console.error('Failed to add friend from banner:', error);
    // On error, allow banner to show again
    bannerDismissedForUser.value = null;
    // TODO: Show error notification to user
  } finally {
    bannerActionLoading.value = null;
  }
}

// Handle block user from banner
async function handleBlockUserFromBanner() {
  const peerId = selectedConversation.value?.peerId || targetUserId.value;
  const displayName = selectedDisplayName.value;

  if (!peerId || bannerActionLoading.value) {
    return;
  }

  // Dismiss banner immediately for this user (prevents flicker)
  bannerDismissedForUser.value = peerId;
  bannerActionLoading.value = 'block';

  try {
    await blockUser(peerId, displayName);
    console.log('Successfully blocked user from chat banner:', peerId);
    // Banner will stay hidden due to bannerDismissedForUser
  } catch (error) {
    console.error('Failed to block user from banner:', error);
    // On error, allow banner to show again
    bannerDismissedForUser.value = null;
    // TODO: Show error notification to user
  } finally {
    bannerActionLoading.value = null;
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

// Auto-scroll to bottom when new messages arrive
watch(
  currentMessages,
  async (newMessages, oldMessages) => {
    console.log('[ChatView] Messages changed:', {
      oldLength: oldMessages?.length,
      newLength: newMessages.length,
    });

    // Skip if no messages
    if (!newMessages || newMessages.length === 0) {
      console.log('[ChatView] Skip auto-scroll: no messages');
      return;
    }

    await nextTick();

    const container = messagesContainer.value;
    if (!container) {
      console.log('[ChatView] Skip auto-scroll: container not found');
      return;
    }

    console.log('[ChatView] Auto-scrolling to bottom...');
    // Use requestAnimationFrame to ensure DOM is fully rendered before scrolling
    requestAnimationFrame(() => {
      scrollToBottom();
      console.log('[ChatView] Scrolled to bottom');
    });
  },
  { deep: true }
);

function onImageError(event: Event) {
  const img = event.target as HTMLImageElement;
  img.src = defaultAvatarUrl;
}

// Group dialog handlers
function closeInviteMembersDialog() {
  isInviteMembersDialogOpen.value = false;
}

function closeViewMembersDialog() {
  isViewMembersDialogOpen.value = false;
}

function closeEditGroupDialog() {
  isEditGroupDialogOpen.value = false;
}

function closeLeaveGroupDialog() {
  isLeaveGroupDialogOpen.value = false;
}

function handleLeaveGroupSuccess() {
  console.log('[ChatView] Left group successfully');
  // Clear selection since group no longer exists for this user
  selectedConversation.value = null;
  setActiveConversation(null);
}

// Handle group menu actions
function handleGroupAction(action: string) {
  if (!selectedConversation.value) return;

  switch (action) {
    case 'invite-members':
      isInviteMembersDialogOpen.value = true;
      break;
    case 'view-members':
      isViewMembersDialogOpen.value = true;
      break;
    case 'edit-group':
      isEditGroupDialogOpen.value = true;
      break;
    case 'leave-group':
      isLeaveGroupDialogOpen.value = true;
      break;
  }
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
// Also sync selectedConversation when underlying data changes (e.g., name/avatar update)
watch(conversations, () => {
  const userId = route.query.userId as string | undefined;
  if (userId && targetUserId.value === userId && !selectedConversation.value) {
    // Retry finding conversation now that conversations are loaded
    const targetConversation = conversations.value.find(conv => conv.peerId === userId);
    if (targetConversation) {
      selectConversation(targetConversation);
    }
  }

  // Sync selectedConversation with updated data from conversations array
  if (selectedConversation.value) {
    const updatedConversation = conversations.value.find(
      conv => conv.conversationId === selectedConversation.value?.conversationId
    );
    if (updatedConversation) {
      // Update the ref with the latest data (preserves reactivity for header/dialogs)
      selectedConversation.value = updatedConversation;
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
  // Reset banner dismissed state when switching conversations
  bannerDismissedForUser.value = null;
});

// Handle initial load
onMounted(() => {
  const userId = route.query.userId as string | undefined;
  if (userId) {
    handleUserIdNavigation(userId);
  }
});
</script>

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
              <!-- Sent messages: timestamp on left, content on right -->
              <template v-if="message.senderId === currentUserId">
                <div class="text-xs text-gray-400 pb-1 whitespace-nowrap">
                  {{ formatMessageTime(message.sendTimestamp) }}
                </div>
                <!-- Image message -->
                <ImageMessageBubble
                  v-if="message.fileUrl && isImageFile(message.fileName)"
                  :file-url="message.fileUrl"
                  :file-name="message.fileName || ''"
                  :is-self="true"
                  @preview="openImagePreview"
                />
                <!-- File message -->
                <FileMessageBubble
                  v-else-if="message.fileUrl"
                  :file-url="message.fileUrl"
                  :file-name="message.fileName || ''"
                  :is-self="true"
                  @preview="openFileInfo"
                />
                <!-- Text message -->
                <div v-else class="max-w-md px-4 py-2 rounded-2xl bg-blue-500 text-white">
                  <div>{{ message.text }}</div>
                </div>
              </template>

              <!-- Received messages: avatar, content, timestamp -->
              <template v-else>
                <!-- Sender Avatar (aligned to top with sender name) -->
                <img
                  :src="getMessageSenderAvatarUrl(message.senderId)"
                  @error="onImageError"
                  class="w-8 h-8 rounded-full object-cover flex-shrink-0 self-start"
                />
                <!-- Message content with optional sender name -->
                <div class="flex flex-col">
                  <!-- Sender name (only for group chats) -->
                  <div v-if="isGroupChat" class="text-xs text-gray-500 mb-1">
                    {{ getMessageSenderName(message.senderId) }}
                  </div>
                  <!-- Image message -->
                  <ImageMessageBubble
                    v-if="message.fileUrl && isImageFile(message.fileName)"
                    :file-url="message.fileUrl"
                    :file-name="message.fileName || ''"
                    :is-self="false"
                    @preview="openImagePreview"
                  />
                  <!-- File message -->
                  <FileMessageBubble
                    v-else-if="message.fileUrl"
                    :file-url="message.fileUrl"
                    :file-name="message.fileName || ''"
                    :is-self="false"
                    @preview="openFileInfo"
                  />
                  <!-- Text message -->
                  <div v-else class="max-w-md px-4 py-2 rounded-2xl bg-white text-gray-900">
                    <div>{{ message.text }}</div>
                  </div>
                </div>
                <div class="text-xs text-gray-400 pb-1 whitespace-nowrap self-end">
                  {{ formatMessageTime(message.sendTimestamp) }}
                </div>
              </template>
            </div>
          </template>

          <!-- Upload Progress Placeholder -->
          <div v-if="uploading" class="flex justify-end px-4 py-2">
            <div class="max-w-xs bg-blue-100 border border-blue-200 rounded-2xl rounded-br-sm px-4 py-3">
              <div class="flex items-center gap-3">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-8 h-8 text-blue-500 flex-shrink-0">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m6.75 12-3-3m0 0-3 3m3-3v6m-1.5-15H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
                </svg>
                <div class="flex-1 min-w-0">
                  <div class="text-sm text-blue-800 truncate mb-1">{{ uploadProgress?.fileName || 'Uploading...' }}</div>
                  <div v-if="progressPercent >= 0" class="flex items-center gap-2">
                    <div class="flex-1 bg-blue-200 rounded-full h-1.5">
                      <div class="bg-blue-500 h-1.5 rounded-full transition-all duration-300" :style="{ width: `${progressPercent}%` }"></div>
                    </div>
                    <span class="text-xs text-blue-600 w-8 text-right">{{ progressPercent }}%</span>
                  </div>
                  <div v-else class="flex items-center gap-2">
                    <svg class="animate-spin h-3 w-3 text-blue-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    <span class="text-xs text-blue-600">Uploading...</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
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
            <!-- File Attachment Button -->
            <button
              @click="handleAttachFile"
              :disabled="uploading"
              class="p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              title="Attach file"
            >
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                <path stroke-linecap="round" stroke-linejoin="round" d="m18.375 12.739-7.693 7.693a4.5 4.5 0 0 1-6.364-6.364l10.94-10.94A3 3 0 1 1 19.5 7.372L8.552 18.32m.009-.01-.01.01m5.699-9.941-7.81 7.81a1.5 1.5 0 0 0 2.112 2.13" />
              </svg>
            </button>

            <textarea
              v-model="messageInput"
              @keydown.enter.exact="handleKeydownEnter"
              @compositionstart="isComposing = true"
              @compositionend="onCompositionEnd"
              placeholder="Type a message..."
              rows="1"
              :disabled="uploading"
              class="flex-1 px-4 py-2 bg-gray-50 border border-gray-200 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
              style="max-height: 120px"
            ></textarea>
            <button
              @click="handleSend"
              :disabled="!messageInput.trim() || uploading"
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

    <!-- File Preview Modals -->
    <ImagePreviewModal
      :is-open="isImagePreviewOpen"
      :image-url="previewFileUrl"
      :file-name="previewFileName"
      :downloading="isDownloading"
      @close="closeImagePreview"
      @download="handleDownload"
    />

    <FileInfoModal
      :is-open="isFileInfoOpen"
      :file-url="previewFileUrl"
      :file-name="previewFileName"
      :downloading="isDownloading"
      @close="closeFileInfo"
      @download="handleDownload"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { open } from '@tauri-apps/plugin-dialog';
import { useChatDisplay } from '../composables/chat/useChatDisplay';
import { useRelationsDisplay } from '../composables/useRelationsDisplay';
import { useRelationActions } from '../composables/useRelationActions';
import { useUserProfileDisplay } from '../composables/useUserProfileDisplay';
import { useFileUpload } from '../composables/chat/useFileUpload';
import { useGroupMembersCache, type SenderInfo } from '../composables/chat/useGroupMembersCache';
import { getConversationDisplayName, getConversationAvatar } from '../types/chat';
import type { ConversationDisplay } from '../types/chat';
import { MessageType } from '../types/chat';
import { formatMessageTime, formatMessageDate } from '../utils/dateFormat';
import { isImageFile, downloadFile, extractFileName } from '../utils/fileUtils';
import ConversationItem from '../components/chat/ConversationItem.vue';
import EmptyView from '../components/views/EmptyView.vue';
import CommandMessage from '../components/chat/CommandMessage.vue';
import GroupChatHeader from '../components/chat/GroupChatHeader.vue';
import InviteMembersDialog from '../components/group/InviteMembersDialog.vue';
import ViewMembersDialog from '../components/group/ViewMembersDialog.vue';
import EditGroupDialog from '../components/group/EditGroupDialog.vue';
import LeaveGroupDialog from '../components/group/LeaveGroupDialog.vue';
import ImageMessageBubble from '../components/chat/ImageMessageBubble.vue';
import FileMessageBubble from '../components/chat/FileMessageBubble.vue';
import ImagePreviewModal from '../components/chat/ImagePreviewModal.vue';
import FileInfoModal from '../components/chat/FileInfoModal.vue';
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

// File upload
const { uploading, uploadProgress, progressPercent, uploadFile } = useFileUpload();

// Group members cache for sender avatars
const { fetchGroupMembers, getSenderInfo } = useGroupMembersCache();

// File preview modal state
const isImagePreviewOpen = ref(false);
const isFileInfoOpen = ref(false);
const previewFileUrl = ref('');
const previewFileName = ref('');

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
const isComposing = ref(false);
const justFinishedComposing = ref(false);
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

/**
 * Get sender info for a received message
 * - For 1v1 chats: returns peer avatar (no name needed)
 * - For group chats: returns sender avatar and name from cache
 */
function getMessageSenderInfo(senderId: string): SenderInfo | undefined {
  if (!selectedConversation.value) return undefined;

  // For group chats, lookup from group members cache
  if (selectedConversation.value.groupId) {
    return getSenderInfo(selectedConversation.value.groupId, senderId);
  }

  // For 1v1 chats, use peer profile from conversation
  const peerProfile = selectedConversation.value.peerProfile;
  if (peerProfile) {
    return {
      name: peerProfile.remarkName || peerProfile.nickName,
      avatar: peerProfile.avatar,
    };
  }

  return undefined;
}

/**
 * Get avatar URL for a message sender
 */
function getMessageSenderAvatarUrl(senderId: string): string {
  const senderInfo = getMessageSenderInfo(senderId);
  if (!senderInfo?.avatar) return defaultAvatarUrl;

  const avatar = senderInfo.avatar;
  if (avatar.startsWith('http://') || avatar.startsWith('https://')) {
    return avatar;
  }
  return `asset://localhost/${avatar}`;
}

/**
 * Get sender name for group chat messages
 */
function getMessageSenderName(senderId: string): string {
  const senderInfo = getMessageSenderInfo(senderId);
  return senderInfo?.name || 'Unknown';
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
    // For group chats, prefetch group members for sender avatar display
    if (conversation.groupId) {
      // Fire and forget - don't block conversation loading
      fetchGroupMembers(conversation.groupId).catch((err) => {
        console.warn('[ChatView] Failed to prefetch group members:', err);
      });
    }

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

// Handle IME composition end
function onCompositionEnd() {
  isComposing.value = false;
  justFinishedComposing.value = true;
}

// Handle Enter key - ignore during IME composition
function handleKeydownEnter(event: KeyboardEvent) {
  // Ignore Enter that confirms IME selection
  if (justFinishedComposing.value) {
    justFinishedComposing.value = false;
    return;
  }
  if (isComposing.value || event.isComposing) return;
  event.preventDefault();
  handleSend();
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

// Handle file attachment button click
async function handleAttachFile() {
  if (uploading.value || !currentUserId.value) {
    return;
  }

  // Must have either a selected conversation or a target user
  if (!selectedConversation.value && !targetUserId.value) {
    return;
  }

  try {
    // Open native file picker
    const filePath = await open({
      multiple: false,
      directory: false,
    });

    if (!filePath) {
      console.log('[ChatView] File picker cancelled');
      return;
    }

    console.log('[ChatView] Selected file:', filePath);

    // Upload the file
    const fileUrl = await uploadFile(filePath);
    const fileName = extractFileName(filePath);

    // Send file message
    const conversationId = selectedConversation.value?.conversationId || '';
    const groupId = selectedConversation.value?.groupId || null;
    const receiverId = groupId ? null : (selectedConversation.value?.peerId || targetUserId.value || '');

    await sendFileMessage(
      currentUserId.value,
      conversationId,
      receiverId,
      groupId,
      fileUrl,
      fileName
    );

    // Clear targetUserId after sending first message
    if (targetUserId.value) {
      console.log('First file message sent to user:', targetUserId.value);
      targetUserId.value = null;
    }

    // Auto scroll on send
    await nextTick();
    scrollToBottom();
  } catch (error) {
    console.error('[ChatView] Failed to send file:', error);
  }
}

// Send file message (separate from text message)
async function sendFileMessage(
  senderId: string,
  conversationId: string,
  receiverId: string | null,
  groupId: string | null,
  fileUrl: string,
  fileName: string
) {
  const { invoke } = await import('@tauri-apps/api/core');
  await invoke('send_message', {
    senderId,
    conversationId,
    receiverId,
    groupId,
    text: null,
    fileUrl,
    fileName,
  });
}

// Open image preview modal
function openImagePreview(url: string, name: string) {
  previewFileUrl.value = url;
  previewFileName.value = name;
  isImagePreviewOpen.value = true;
}

// Open file info modal
function openFileInfo(url: string, name: string) {
  previewFileUrl.value = url;
  previewFileName.value = name;
  isFileInfoOpen.value = true;
}

// Close preview modals
function closeImagePreview() {
  isImagePreviewOpen.value = false;
}

function closeFileInfo() {
  isFileInfoOpen.value = false;
}

// Handle file download
const isDownloading = ref(false);

async function handleDownload(url: string, name: string) {
  if (isDownloading.value) return;

  isDownloading.value = true;
  try {
    await downloadFile(url, name);
    // Download completed (user saved the file)
  } catch (error) {
    console.error('[ChatView] Download failed:', error);
    alert('Download failed');
  } finally {
    isDownloading.value = false;
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

// Auto-scroll to bottom when upload placeholder appears
watch(uploading, async (isUploading) => {
  if (isUploading) {
    await nextTick();
    requestAnimationFrame(() => {
      scrollToBottom();
    });
  }
});

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

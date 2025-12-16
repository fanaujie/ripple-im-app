import { computed, ref, onMounted, type Ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { ConversationDisplay, ConversationReceivedMessageEvent } from '../../types/chat';
import { sortConversationsByTime, MessageAction } from '../../types/chat';
import { useConversationsState } from './useConversationsState';
import { useMessagesState } from './useMessagesState';
import { useConversationEvents } from './useConversationEvents';
import { useMessageEvents } from './useMessageEvents';
import { useChatActions } from './useChatActions';
import type { RelationUser } from '../../types/relations';

interface UIConversations {
  conversations: ConversationDisplay[];
}

/**
 * Main composable for chat display
 *
 * Combines all chat-related state and logic:
 * - Conversations list with peer profiles
 * - Messages grouped by conversation
 * - Event listeners
 * - Actions (send, mark read, load)
 *
 * @param relations - Map of userId to RelationUser for peer profile lookup
 * @returns Everything needed for chat UI
 */
export function useChatDisplay(relations: Ref<Map<string, RelationUser>>) {
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Track the currently active conversation (for filtering incoming messages)
  const activeConversationId = ref<string | null>(null);

  // Initialize state managers
  const conversationsState = useConversationsState(relations);
  const messagesState = useMessagesState();

  // Actions
  const actions = useChatActions();

  /**
   * Handle new message received event
   * If the message is for the active conversation, mark it as read immediately
   */
  function handleReceivedNewMessage(event: ConversationReceivedMessageEvent): void {
    const { conversationId, message, timestamp } = event;

    // Always update the preview (lastMessage, lastMessageTimestamp)
    // But handle unreadCount differently based on whether this is the active conversation
    if (conversationId === activeConversationId.value) {
      // Active conversation: update preview but set unreadCount to 0
      // (we'll call markConversationRead when the message-updated event arrives with messageId)
      const conversation = conversationsState.conversations.value.find(
        c => c.conversationId === conversationId
      );
      if (conversation) {
        const timestampMs = parseInt(timestamp, 10) * 1000;
        conversation.lastMessage = message;
        conversation.lastMessageTimestamp = timestampMs;
        // Set unreadCount to 0 since user is viewing this conversation
        conversation.unreadCount = 0;
        console.log('[useChatDisplay] Updated preview for active conversation (unreadCount=0):', conversationId);
      }
    } else {
      // Inactive conversation: use normal behavior (apply unreadCount from push)
      conversationsState.handleReceivedNewMessage(event);
    }
  }

  /**
   * Handle message update event
   * When a new message is added to the active conversation, mark it as read
   */
  function handleMessageEvent(event: import('../../types/chat').MessageUpdateEvent): void {
    // First, let messagesState handle the event (add/update/delete message)
    messagesState.handleEvent(event);

    // If this is a new message (ADD action) for the active conversation, mark as read
    if (event.action === MessageAction.ADD && event.message) {
      const messageId = event.message.messageId;
      const conversationId = event.message.conversationId;

      // This should always be true since useMessageEvents filters by active conversation,
      // but double-check to be safe
      if (conversationId === activeConversationId.value) {
        console.log('[useChatDisplay] New message in active conversation, marking as read:', messageId);
        // Call markConversationRead (fire-and-forget, don't await)
        actions.markConversationRead(conversationId, messageId).then(() => {
          // Update lastReadMessageId in conversation state
          conversationsState.updateUnreadCount(conversationId, 0, messageId);
        }).catch(err => {
          console.error('[useChatDisplay] Failed to auto-mark as read:', err);
        });
      }
    }
  }

  // Event listeners
  useConversationEvents({
    onInsert: conversationsState.handleInsert,
    onUpdate: conversationsState.handleUpdate,
    onDelete: conversationsState.handleDelete,
    onClearAll: conversationsState.handleClearAll,
    onReceivedNewMessage: handleReceivedNewMessage,
  });
  useMessageEvents(activeConversationId, handleMessageEvent);

  /**
   * Sorted conversations (by last message time, descending)
   */
  const sortedConversations = computed(() => {
    return sortConversationsByTime(conversationsState.conversations.value);
  });

  /**
   * Initialize conversations from get_conversations
   * Called automatically on mount, can also be called manually to refresh
   */
  async function initializeConversations(): Promise<void> {
    loading.value = true;
    error.value = null;

    try {
      console.log('[useChatDisplay] Fetching initial data...');
      const data = await invoke<UIConversations>('get_conversations');
      conversationsState.initialize(data.conversations);
      console.log('[useChatDisplay] Initialized conversations:', data.conversations.length);
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load conversations';
      console.error('[useChatDisplay] Failed to initialize conversations:', err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  // Initialize on mount (same pattern as useRelationsDisplay)
  onMounted(() => {
    initializeConversations();
  });

  /**
   * Load messages for a conversation
   */
  async function loadMessages(conversationId: string): Promise<void> {
    try {
      const messages = await actions.loadConversationMessages(conversationId);
      messagesState.loadMessages(conversationId, messages);
    } catch (err) {
      console.error('[useChatDisplay] Failed to load messages:', err);
      throw err;
    }
  }

  /**
   * Load latest messages for initial view
   * @param conversationId - Conversation ID
   * @param limit - Number of messages to load
   * @param lastReadMessageId - Last read message ID for cache optimization (empty string for first visit)
   * @returns Number of messages loaded
   */
  async function loadLatestMessages(
    conversationId: string,
    limit: number = 50,
    lastReadMessageId: string = ''
  ): Promise<number> {
    try {
      const messages = await actions.loadLatestMessages(conversationId, limit, lastReadMessageId);
      messagesState.loadMessages(conversationId, messages);
      return messages.length;
    } catch (err) {
      console.error('[useChatDisplay] Failed to load latest messages:', err);
      throw err;
    }
  }

  /**
   * Load older messages for pagination (向上滾動時使用)
   * @returns true if might have more messages, false if no more or error
   */
  async function loadOlderMessages(conversationId: string, limit: number = 50): Promise<boolean> {
    try {
      const oldestId = messagesState.getOldestMessageId(conversationId);
      if (!oldestId) {
        console.log('[useChatDisplay] No messages yet, cannot load older');
        return false;
      }

      const messages = await actions.loadOlderMessages(conversationId, oldestId, limit);
      if (messages.length === 0) {
        console.log('[useChatDisplay] No more older messages');
        return false;
      }

      messagesState.prependMessages(conversationId, messages);
      return messages.length === limit; // True if might have more
    } catch (err) {
      console.error('[useChatDisplay] Failed to load older messages:', err);
      throw err;
    }
  }

  /**
   * Send a message to a conversation
   */
  async function sendMessage(
    senderId: string,
    conversationId: string,
    receiverId: string | null,
    content: string,
    groupId: string | null = null
  ): Promise<string> {
    return actions.sendMessage(senderId, conversationId, receiverId, content, groupId);
  }

  /**
   * Mark conversation as read and update unread count to 0
   * Only calls API if there are actually new unread messages (optimization)
   * @param conversationId - Conversation ID
   * @param lastMessageId - Last message ID in the conversation
   * @param currentLastReadMessageId - Current last read message ID (optional)
   */
  async function markConversationRead(
    conversationId: string,
    lastMessageId: string,
    currentLastReadMessageId?: string
  ): Promise<void> {
    try {
      // Optimization: Skip API call if already read
      if (currentLastReadMessageId && currentLastReadMessageId === lastMessageId) {
        console.log(
          `[useChatDisplay] Skipping mark-as-read: already at ${lastMessageId}`
        );
        // Still update local unread count to 0 (for UI consistency)
        conversationsState.updateUnreadCount(conversationId, 0);
        return;
      }

      console.log(
        `[useChatDisplay] Marking as read: ${conversationId} → ${lastMessageId}`,
        currentLastReadMessageId ? `(was: ${currentLastReadMessageId})` : '(first read)'
      );

      await actions.markConversationRead(conversationId, lastMessageId);
      // Update local unread count to 0 AND update lastReadMessageId
      conversationsState.updateUnreadCount(conversationId, 0, lastMessageId);
    } catch (err) {
      console.error('[useChatDisplay] Failed to mark as read:', err);
      throw err;
    }
  }

  /**
   * Get messages for a specific conversation
   */
  function getConversationMessages(conversationId: string) {
    return messagesState.getMessages(conversationId);
  }

  /**
   * Find a conversation by ID
   */
  function getConversationById(conversationId: string): ConversationDisplay | undefined {
    return conversationsState.conversations.value.find(
      (c) => c.conversationId === conversationId
    );
  }

  /**
   * Set the active conversation (updates message filtering)
   * Clears previous conversation's messages and loads new conversation's messages
   * @returns Whether there might be more older messages to load
   */
  async function setActiveConversation(conversationId: string | null): Promise<boolean> {
    // Clear previous conversation's messages to free memory
    if (activeConversationId.value && activeConversationId.value !== conversationId) {
      console.log('[useChatDisplay] Clearing messages for:', activeConversationId.value);
      messagesState.clearConversationMessages(activeConversationId.value);
    }

    // Set new active conversation
    activeConversationId.value = conversationId;
    console.log('[useChatDisplay] Active conversation set to:', conversationId);

    // Load messages for the new active conversation
    if (conversationId) {
      try {
        // Get conversation to access last_read_message_id for cache optimization
        const conversation = conversationsState.conversations.value.find(
          (c) => c.conversationId === conversationId
        );
        const lastReadMessageId = conversation?.lastReadMessageId || '';

        // Pass last_read_message_id for cache optimization
        const limit = 50;
        const loadedCount = await loadLatestMessages(conversationId, limit, lastReadMessageId);
        // If we got fewer messages than requested, there are no more older messages
        return loadedCount >= limit;
      } catch (err) {
        console.error('[useChatDisplay] Failed to load messages for active conversation:', err);
        return false;
      }
    }
    return false;
  }

  return {
    // State
    conversations: sortedConversations,
    loading,
    error,
    activeConversationId,

    // Methods
    initializeConversations,
    loadMessages,
    loadLatestMessages,
    loadOlderMessages,
    sendMessage,
    markConversationRead,
    getConversationMessages,
    getConversationById,
    setActiveConversation,
  };
}

import { computed, ref, onMounted, type Ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { ConversationDisplay } from '../../types/chat';
import { sortConversationsByTime } from '../../types/chat';
import { useConversationsState } from './useConversationsState';
import { useMessagesState } from './useMessagesState';
import { useConversationEvents } from './useConversationEvents';
import { useMessageEvents } from './useMessageEvents';
import { useChatActions } from './useChatActions';
import type { RelationUser } from '../../types/relations';

interface UIConversations {
  conversation: ConversationDisplay[];
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

  // Event listeners
  useConversationEvents(conversationsState.handleEvent);
  useMessageEvents(activeConversationId, messagesState.handleEvent);

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
      conversationsState.initialize(data.conversation);
      console.log('[useChatDisplay] Initialized conversations:', data.conversation.length);
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
   */
  async function loadLatestMessages(conversationId: string, limit: number = 50): Promise<void> {
    try {
      const messages = await actions.loadLatestMessages(conversationId, limit);
      messagesState.loadMessages(conversationId, messages);
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
    receiverId: string,
    content: string
  ): Promise<string> {
    return actions.sendMessage(senderId, conversationId, receiverId, content);
  }

  /**
   * Mark conversation as read and update unread count to 0
   */
  async function markConversationRead(
    conversationId: string,
    lastMessageId: string
  ): Promise<void> {
    try {
      await actions.markConversationRead(conversationId, lastMessageId);
      // Update local unread count to 0
      conversationsState.updateUnreadCount(conversationId, 0);
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
   */
  function setActiveConversation(conversationId: string | null) {
    activeConversationId.value = conversationId;
    console.log('[useChatDisplay] Active conversation set to:', conversationId);
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

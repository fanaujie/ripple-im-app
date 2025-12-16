import { ref, type Ref } from 'vue';
import type {
  ConversationDisplay,
  ConversationUpdateEvent,
  ConversationItem,
  ConversationReceivedMessageEvent,
} from '../../types/chat';
import { ConversationAction } from '../../types/chat';
import type { RelationUser } from '../../types/relations';

/**
 * Composable for managing conversations state
 *
 * Handles all conversation update events and maintains reactive list of conversations
 * Each conversation includes peer profile information for display
 *
 * @param relations - Ref to relations map for looking up peer profiles
 * @returns State and update handler
 */
export function useConversationsState(relations: Ref<Map<string, RelationUser>>) {
  const conversations = ref<ConversationDisplay[]>([]);

  /**
   * Handle conversation insert event (conversation-inserted)
   * Creates a new conversation or updates if it already exists
   */
  function handleInsert(conversation: ConversationItem): void {
    updateConversation(conversation);
    console.log('[useConversationsState] Inserted conversation:', conversation.conversationId);
  }

  /**
   * Handle conversation update event (conversation-updated)
   * Updates an existing conversation, replacing it entirely
   */
  function handleUpdate(conversation: ConversationItem): void {
    updateConversation(conversation);
    console.log('[useConversationsState] Updated conversation:', conversation.conversationId);
  }

  /**
   * Handle conversation delete event (conversations-deleted)
   * Removes a conversation by ID
   */
  function handleDelete(conversationId: string): void {
    removeConversation(conversationId);
    console.log('[useConversationsState] Deleted conversation:', conversationId);
  }

  /**
   * Handle clear all event (conversations-cleared-all)
   * Removes all conversations
   */
  function handleClearAll(): void {
    conversations.value = [];
    console.log('[useConversationsState] Cleared all conversations');
  }

  /**
   * Handle new message preview event (conversation-received-new-message)
   * Updates conversation's lastMessage, lastMessageTimestamp, and unreadCount
   */
  function handleReceivedNewMessage(event: ConversationReceivedMessageEvent): void {
    const { conversationId, message, unreadCount, timestamp } = event;

    // Find the conversation
    const conversation = conversations.value.find(c => c.conversationId === conversationId);

    if (conversation) {
      // Convert UTC seconds (string) to milliseconds (number)
      const timestampMs = parseInt(timestamp, 10) * 1000;

      // Update preview and timestamp
      conversation.lastMessage = message;
      conversation.lastMessageTimestamp = timestampMs;
      conversation.unreadCount = unreadCount;

      console.log('[useConversationsState] Updated preview for:', conversationId, {
        preview: message.substring(0, 20) + (message.length > 20 ? '...' : ''),
        unreadCount,
        timestamp: new Date(timestampMs).toISOString(),
      });
    } else {
      console.warn('[useConversationsState] Conversation not found for message preview:', conversationId);
    }
  }

  /**
   * Handle a conversation update event from the backend
   * @deprecated Use separate handleInsert/handleUpdate/handleDelete/handleClearAll methods instead
   */
  function handleEvent(event: ConversationUpdateEvent): void {
    const { action, conversation } = event;

    // CLEAR action - reset everything
    if (action === ConversationAction.CLEAR) {
      handleClearAll();
      return;
    }

    // All other actions require conversation data
    if (!conversation) {
      console.warn('[useConversationsState] Received event without conversation data:', action);
      return;
    }

    switch (action) {
      case ConversationAction.CREATE:
      case ConversationAction.NEW_MESSAGE:
      case ConversationAction.READ_MESSAGE:
      case ConversationAction.UPDATE_NAME:
      case ConversationAction.UPDATE_AVATAR:
      case ConversationAction.UPDATE_NAME_AVATAR:
        // All update-like actions now just do a full replace
        handleUpdate(conversation);
        break;

      case ConversationAction.DELETE:
        handleDelete(conversation.conversationId);
        break;

      default:
        console.warn('[useConversationsState] Unknown action:', action);
    }
  }

  /**
   * Initialize state with conversation displays
   */
  function initialize(conversationDisplays: ConversationDisplay[]): void {
    conversations.value = conversationDisplays.map(enrichWithPeerProfile);
    console.log(`[useConversationsState] Initialized: ${conversationDisplays.length} conversations`);
  }

  /**
   * Add a new conversation
   */
  function addConversation(conversation: ConversationItem): void {
    // Create ConversationDisplay with unreadCount from conversation
    const display: ConversationDisplay = {
      ...conversation,
    };

    const enriched = enrichWithPeerProfile(display);
    conversations.value.push(enriched);
  }

  /**
   * Update an existing conversation
   */
  function updateConversation(conversation: ConversationItem): void {
    const list = conversations.value;
    const index = list.findIndex((c) => c.conversationId === conversation.conversationId);

    if (index >= 0) {
      // Use unreadCount from the updated conversation
      const display: ConversationDisplay = {
        ...conversation,
      };

      const enriched = enrichWithPeerProfile(display);
      list.splice(index, 1, enriched);
    } else {
      // Conversation doesn't exist, add it
      addConversation(conversation);
    }
  }

  /**
   * Remove a conversation by ID
   */
  function removeConversation(conversationId: string): void {
    const list = conversations.value;
    const index = list.findIndex((c) => c.conversationId === conversationId);

    if (index >= 0) {
      list.splice(index, 1);
    }
  }

  /**
   * Update specific fields of a conversation (for granular updates like name/avatar)
   * @deprecated No longer needed with new event structure - full replace is used instead
   */
  // function updateConversationField(
  //   conversationId: string,
  //   updates: Partial<ConversationItem>
  // ): void {
  //   const list = conversations.value;
  //   const index = list.findIndex((c) => c.conversationId === conversationId);
  //
  //   if (index >= 0) {
  //     const updated = {
  //       ...list[index],
  //       ...updates,
  //     };
  //     list.splice(index, 1, updated);
  //   } else {
  //     // This should not happen - conversation should exist after CREATE event
  //     console.error(
  //       '[useConversationsState] Cannot update conversation fields - conversation not found:',
  //       conversationId,
  //       'Updates:',
  //       updates
  //     );
  //   }
  // }

  /**
   * Enrich conversation with peer profile from relations
   */
  function enrichWithPeerProfile(conversation: ConversationDisplay): ConversationDisplay {
    if (conversation.peerId) {
      const peerProfile = relations.value.get(conversation.peerId);
      return {
        ...conversation,
        peerProfile,
      };
    }
    return conversation;
  }

  /**
   * Update unread count and optionally lastReadMessageId for a conversation
   * (called after marking as read)
   */
  function updateUnreadCount(
    conversationId: string,
    unreadCount: number,
    lastReadMessageId?: string
  ): void {
    const list = conversations.value;
    const index = list.findIndex((c) => c.conversationId === conversationId);

    if (index >= 0) {
      const updated = {
        ...list[index],
        unreadCount,
        ...(lastReadMessageId !== undefined && { lastReadMessageId }),
      };
      list.splice(index, 1, updated);
    }
  }

  return {
    conversations,
    handleEvent, // @deprecated
    handleInsert,
    handleUpdate,
    handleDelete,
    handleClearAll,
    handleReceivedNewMessage,
    initialize,
    updateUnreadCount,
  };
}

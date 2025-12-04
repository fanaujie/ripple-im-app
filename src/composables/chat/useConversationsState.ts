import { ref, type Ref } from 'vue';
import type {
  ConversationDisplay,
  ConversationUpdateEvent,
  ConversationItem,
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
   * Handle a conversation update event from the backend
   */
  function handleEvent(event: ConversationUpdateEvent): void {
    const { action, conversation } = event;

    // CLEAR action - reset everything
    if (action === ConversationAction.CLEAR) {
      conversations.value = [];
      console.log('[useConversationsState] Cleared all conversations');
      return;
    }

    // All other actions require conversation data
    if (!conversation) {
      console.warn('[useConversationsState] Received event without conversation data:', action);
      return;
    }

    switch (action) {
      case ConversationAction.CREATE:
        // CREATE includes full data - do complete upsert
        updateConversation(conversation);
        console.log('[useConversationsState] Created conversation:', conversation.conversationId);
        break;

      case ConversationAction.NEW_MESSAGE:
        // NEW_MESSAGE only includes message fields - update specific fields to preserve name/avatar
        updateConversationField(conversation.conversationId, {
          lastMessageId: conversation.lastMessageId,
          lastMessage: conversation.lastMessage,
          lastMessageTimestamp: conversation.lastMessageTimestamp,
          unreadCount: conversation.unreadCount,
        });
        console.log('[useConversationsState] Updated conversation with new message:', conversation.conversationId);
        break;

      case ConversationAction.READ_MESSAGE:
        // READ_MESSAGE only includes read-related fields
        updateConversationField(conversation.conversationId, {
          lastReadMessageId: conversation.lastReadMessageId,
          unreadCount: conversation.unreadCount,
        });
        console.log('[useConversationsState] Updated read status:', conversation.conversationId);
        break;

      case ConversationAction.UPDATE_NAME:
        // Update only the name field
        updateConversationField(conversation.conversationId, { name: conversation.name });
        console.log('[useConversationsState] Updated name:', conversation.conversationId);
        break;

      case ConversationAction.UPDATE_AVATAR:
        // Update only the avatar field
        updateConversationField(conversation.conversationId, { avatar: conversation.avatar });
        console.log('[useConversationsState] Updated avatar:', conversation.conversationId);
        break;

      case ConversationAction.UPDATE_NAME_AVATAR:
        // Update both name and avatar fields atomically
        updateConversationField(conversation.conversationId, {
          name: conversation.name,
          avatar: conversation.avatar
        });
        console.log('[useConversationsState] Updated name and avatar:', conversation.conversationId);
        break;

      case ConversationAction.DELETE:
        removeConversation(conversation.conversationId);
        console.log('[useConversationsState] Deleted conversation:', conversation.conversationId);
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
   */
  function updateConversationField(
    conversationId: string,
    updates: Partial<ConversationItem>
  ): void {
    const list = conversations.value;
    const index = list.findIndex((c) => c.conversationId === conversationId);

    if (index >= 0) {
      const updated = {
        ...list[index],
        ...updates,
      };
      list.splice(index, 1, updated);
    } else {
      // This should not happen - conversation should exist after CREATE event
      console.error(
        '[useConversationsState] Cannot update conversation fields - conversation not found:',
        conversationId,
        'Updates:',
        updates
      );
    }
  }

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
   * Update unread count for a conversation (called after marking as read)
   */
  function updateUnreadCount(conversationId: string, unreadCount: number): void {
    const list = conversations.value;
    const index = list.findIndex((c) => c.conversationId === conversationId);

    if (index >= 0) {
      const updated = { ...list[index], unreadCount };
      list.splice(index, 1, updated);
    }
  }

  return {
    conversations,
    handleEvent,
    initialize,
    updateUnreadCount,
  };
}

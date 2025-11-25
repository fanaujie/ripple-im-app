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
      case ConversationAction.NEW_MESSAGE:
        updateConversation(conversation);
        console.log('[useConversationsState] Upserted conversation:', conversation.conversationId);
        break;

      case ConversationAction.UPDATE:
        updateConversation(conversation);
        console.log('[useConversationsState] Updated conversation:', conversation.conversationId);
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

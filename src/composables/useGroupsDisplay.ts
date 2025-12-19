import { computed, type Ref } from 'vue';
import type { ConversationDisplay } from '../types/chat';

/**
 * Group display item for UI
 */
export interface GroupDisplayItem {
  /** Group ID */
  groupId: string;
  /** Conversation ID (for navigation) */
  conversationId: string;
  /** Group name */
  name: string;
  /** Group avatar URL */
  avatar?: string;
  /** Last message timestamp for sorting */
  lastMessageTimestamp?: number;
}

/**
 * Composable for displaying groups
 *
 * Filters group conversations from the full conversation list
 *
 * @param conversations - Reactive conversations list from useChatDisplay
 * @returns Groups list and lookup methods
 */
export function useGroupsDisplay(conversations: Ref<ConversationDisplay[]>) {
  /**
   * All groups (conversations with groupId)
   */
  const groups = computed<GroupDisplayItem[]>(() => {
    return conversations.value
      .filter((conv) => !!conv.groupId)
      .map((conv) => ({
        groupId: conv.groupId!,
        conversationId: conv.conversationId,
        name: conv.name || conv.conversationId,
        avatar: conv.avatar,
        lastMessageTimestamp: conv.lastMessageTimestamp,
      }))
      .sort((a, b) => {
        // Sort by last message timestamp (descending)
        const timeA = a.lastMessageTimestamp || 0;
        const timeB = b.lastMessageTimestamp || 0;
        return timeB - timeA;
      });
  });

  /**
   * Find a group by groupId
   */
  function getGroupById(groupId: string): GroupDisplayItem | undefined {
    return groups.value.find((g) => g.groupId === groupId);
  }

  /**
   * Find a group by conversationId
   */
  function getGroupByConversationId(conversationId: string): GroupDisplayItem | undefined {
    return groups.value.find((g) => g.conversationId === conversationId);
  }

  return {
    // State
    groups,

    // Methods
    getGroupById,
    getGroupByConversationId,
  };
}

import { computed, ref, type Ref } from 'vue';
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
 * Composable for displaying and filtering groups
 *
 * Filters group conversations from the full conversation list
 * and provides search functionality
 *
 * @param conversations - Reactive conversations list from useChatDisplay
 * @returns Groups list, search functionality, and loading state
 */
export function useGroupsDisplay(conversations: Ref<ConversationDisplay[]>) {
  const searchQuery = ref('');

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
   * Filtered groups based on search query
   */
  const filteredGroups = computed<GroupDisplayItem[]>(() => {
    if (!searchQuery.value.trim()) {
      return groups.value;
    }
    const query = searchQuery.value.toLowerCase().trim();
    return groups.value.filter((group) =>
      group.name.toLowerCase().includes(query)
    );
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
    filteredGroups,
    searchQuery,

    // Methods
    getGroupById,
    getGroupByConversationId,
  };
}

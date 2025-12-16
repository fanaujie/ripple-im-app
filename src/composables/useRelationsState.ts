import { ref, computed } from 'vue';
import type { RelationUser, RelationUpdateEvent } from '../types/relations';
import { RelationAction, shouldShowInFriendsList, shouldShowInBlockedList } from '../types/relations';

/**
 * Composable for managing relations state using a Map
 *
 * Architecture:
 * - Stores all relations in a single Map<userId, RelationUser>
 * - Provides computed properties for filtered views (friends, blockedUsers)
 * - Simplifies event handling to basic Map operations (set/delete)
 * - Achieves O(1) lookup performance
 *
 * @returns State and update handler
 */
export function useRelationsState() {
  const relationsMap = ref<Map<string, RelationUser>>(new Map());

  /**
   * Handle relation insert event (relation-inserted)
   * Adds a new relation to the map
   */
  function handleInsert(user: RelationUser): void {
    relationsMap.value.set(user.userId, user);
    console.log('[useRelationsState] Inserted user:', user.userId, 'flags:', user.relationFlags);
  }

  /**
   * Handle relation update event (relation-updated)
   * Updates an existing relation, replacing it entirely
   */
  function handleUpdate(user: RelationUser): void {
    relationsMap.value.set(user.userId, user);
    console.log('[useRelationsState] Updated user:', user.userId, 'flags:', user.relationFlags);
  }

  /**
   * Handle relation delete event (relation-deleted)
   * Removes a relation by userId
   */
  function handleDelete(userId: string): void {
    relationsMap.value.delete(userId);
    console.log('[useRelationsState] Deleted user:', userId);
  }

  /**
   * Handle clear all event (relations-cleared-all)
   * Removes all relations
   */
  function handleClearAll(): void {
    relationsMap.value.clear();
    console.log('[useRelationsState] Cleared all relations');
  }

  /**
   * Handle a relation update event from the backend
   * @deprecated Use separate handleInsert/handleUpdate/handleDelete/handleClearAll methods instead
   */
  function handleEvent(event: RelationUpdateEvent): void {
    const { action, userProfile: user } = event;

    // CLEAR action - reset everything
    if (action === RelationAction.CLEAR) {
      handleClearAll();
      return;
    }

    // All other actions require user data
    if (!user) {
      console.warn('[useRelationsState] Received event without user data:', action);
      return;
    }

    // Determine if this is a removal action
    const isRemoval = action === RelationAction.REMOVE_FRIEND ||
                      action === RelationAction.REMOVE_BLOCK;

    if (isRemoval) {
      handleDelete(user.userId);
    } else {
      // All add/update/block/unblock actions set the user in the map
      // The relationFlags will determine how it's displayed
      handleUpdate(user);
    }
  }

  /**
   * Initialize state with data (called on component mount)
   */
  function initialize(relations: RelationUser[]): void {
    relationsMap.value.clear();
    for (const user of relations) {
      relationsMap.value.set(user.userId, user);
    }
    console.log(`[useRelationsState] Initialized: ${relations.length} relations`);
  }

  /**
   * Computed: Friends list (filtered from relationsMap)
   * Shows users with FRIEND flag and not blocked
   */
  const friends = computed(() => {
    const result: RelationUser[] = [];
    for (const user of relationsMap.value.values()) {
      if (shouldShowInFriendsList(user)) {
        result.push(user);
      }
    }
    return result;
  });

  /**
   * Computed: Blocked users list (filtered from relationsMap)
   * Shows users with BLOCKED flag and not hidden
   */
  const blockedUsers = computed(() => {
    const result: RelationUser[] = [];
    for (const user of relationsMap.value.values()) {
      if (shouldShowInBlockedList(user)) {
        result.push(user);
      }
    }
    return result;
  });

  return {
    relationsMap,
    friends,
    blockedUsers,
    handleEvent, // @deprecated
    handleInsert,
    handleUpdate,
    handleDelete,
    handleClearAll,
    initialize,
  };
}

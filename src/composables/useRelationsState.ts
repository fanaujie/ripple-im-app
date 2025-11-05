import { ref, type Ref } from 'vue';
import type { RelationUser, RelationUpdateEvent } from '../types/relations';
import { RelationAction } from '../types/relations';

/**
 * Composable for managing relations state (friends and blocked users)
 *
 * Handles all relation update events and maintains reactive lists:
 * - friends: List of users with FRIEND flag (not blocked)
 * - blockedUsers: List of users with BLOCKED flag (not hidden)
 *
 * @returns State and update handler
 */
export function useRelationsState() {
  const friends = ref<RelationUser[]>([]);
  const blockedUsers = ref<RelationUser[]>([]);

  /**
   * Handle a relation update event from the backend
   * Updates friends and/or blockedUsers lists based on the action type
   */
  function handleEvent(event: RelationUpdateEvent): void {
    const { action, userProfile: user } = event;

    // CLEAR action - reset everything
    if (action === RelationAction.CLEAR) {
      friends.value = [];
      blockedUsers.value = [];
      console.log('[useRelationsState] Cleared all relations');
      return;
    }

    // All other actions require user data
    if (!user) {
      console.warn('[useRelationsState] Received event without user data:', action);
      return;
    }

    switch (action) {
      case RelationAction.ADD_FRIEND:
        // Add user to friends list, ensure not in blocked list
        upsertUser(friends, user);
        removeUser(blockedUsers, user.userId);
        console.log('[useRelationsState] Added friend:', user.userId);
        break;

      case RelationAction.REMOVE_FRIEND:
        // Remove user from friends list
        removeUser(friends, user.userId);
        console.log('[useRelationsState] Removed friend:', user.userId);
        break;

      case RelationAction.ADD_BLOCK:
        // Add user to blocked list (user was never a friend or already removed)
        upsertUser(blockedUsers, user);
        console.log('[useRelationsState] Added blocked user:', user.userId);
        break;

      case RelationAction.BLOCK_FRIEND:
        // User was a friend, now blocked: move from friends to blockedUsers
        removeUser(friends, user.userId);
        upsertUser(blockedUsers, user);
        console.log('[useRelationsState] Blocked friend:', user.userId);
        break;

      case RelationAction.UNBLOCK_TO_FRIEND:
        // ⭐ Critical: Unblock a friend - restore to friends list
        // User was a blocked friend, now unblocked with friendship restored
        removeUser(blockedUsers, user.userId);
        upsertUser(friends, user);
        console.log('[useRelationsState] Unblocked to friend:', user.userId);
        break;

      case RelationAction.REMOVE_BLOCK:
        // ⭐ Critical: Unblock a stranger - just remove from blocked list
        // User was a blocked stranger (no friendship), simply remove from UI
        removeUser(blockedUsers, user.userId);
        console.log('[useRelationsState] Removed blocked user:', user.userId);
        break;

      case RelationAction.UPDATE_FRIEND:
        // Update user info (remarkName, nickName, avatar) in both lists
        // User might be in either friends or blocked list
        updateUser(friends, user);
        updateUser(blockedUsers, user);
        console.log('[useRelationsState] Updated user:', user.userId);
        break;

      default:
        console.warn('[useRelationsState] Unknown action:', action);
    }
  }

  /**
   * Initialize state with data (called on component mount)
   */
  function initialize(friendsList: RelationUser[], blockedList: RelationUser[]): void {
    friends.value = friendsList;
    blockedUsers.value = blockedList;
    console.log(
      `[useRelationsState] Initialized: ${friendsList.length} friends, ${blockedList.length} blocked`
    );
  }

  return {
    friends,
    blockedUsers,
    handleEvent,
    initialize,
  };
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Upsert (insert or update) a user in a list
 * If user exists, update it; otherwise, add it
 *
 * Note: Uses splice() instead of direct assignment to ensure Vue reactivity
 */
function upsertUser(listRef: Ref<RelationUser[]>, user: RelationUser): void {
  const list = listRef.value;
  const index = list.findIndex((u) => u.userId === user.userId);

  if (index >= 0) {
    // Update existing user using splice for Vue reactivity
    // Direct assignment (list[index] = user) doesn't reliably trigger updates
    list.splice(index, 1, user);
  } else {
    // Add new user
    list.push(user);
  }
}

/**
 * Remove a user from a list by userId
 */
function removeUser(listRef: Ref<RelationUser[]>, userId: string): void {
  const list = listRef.value;
  const index = list.findIndex((u) => u.userId === userId);

  if (index >= 0) {
    list.splice(index, 1);
  }
}

/**
 * Update an existing user in a list
 * Only updates if user exists in the list
 *
 * Note: Uses splice() instead of direct assignment to ensure Vue reactivity
 */
function updateUser(listRef: Ref<RelationUser[]>, user: RelationUser): void {
  const list = listRef.value;
  const index = list.findIndex((u) => u.userId === user.userId);

  if (index >= 0) {
    // Update existing user using splice for Vue reactivity
    // Direct assignment (list[index] = user) doesn't reliably trigger updates
    list.splice(index, 1, user);
  }
}

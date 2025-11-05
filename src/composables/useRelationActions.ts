import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { RelationUser } from '../types/relations';

/**
 * Composable for relation management actions
 *
 * Provides methods to perform relation operations via Tauri commands.
 * All methods are async and will trigger WebSocket events that update the UI.
 *
 * Event Flow:
 * 1. Component calls action (e.g., addFriend)
 * 2. Tauri command sends API request to server
 * 3. Server processes and sends WebSocket push
 * 4. Rust IncrementalSyncManager receives push
 * 5. Storage is updated
 * 6. relation-updated event is emitted
 * 7. UI updates automatically via useRelationsDisplay
 *
 * Usage:
 * ```typescript
 * const { addFriend, blockUser, loading } = useRelationActions();
 *
 * async function handleAddFriend() {
 *   try {
 *     await addFriend('user123');
 *     // UI will update automatically via event
 *   } catch (error) {
 *     // Handle error
 *   }
 * }
 * ```
 */
export function useRelationActions() {
  const loading = ref(false);
  const error = ref<string | null>(null);

  /**
   * Search for a user by their userId
   * Used for adding new friends
   */
  async function getUserProfileById(userId: string): Promise<RelationUser> {
    try {
      loading.value = true;
      error.value = null;

      const data = await invoke<RelationUser>('get_user_profile_by_id', { userId });
      return data;
    } catch (err) {
      console.error('[useRelationActions] Failed to get user profile:', err);
      error.value = err instanceof Error ? err.message : 'Failed to get user profile';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Add a friend by userId
   * Emits: ADD_FRIEND event
   */
  async function addFriend(targetUserId: string): Promise<void> {
    try {
      loading.value = true;
      error.value = null;

      await invoke('add_friend', { targetUserId });
      console.log('[useRelationActions] Friend added:', targetUserId);
      // Backend will emit relation-updated event after WebSocket sync
    } catch (err) {
      console.error('[useRelationActions] Failed to add friend:', err);
      error.value = err instanceof Error ? err.message : 'Failed to add friend';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Update a friend's remark name (custom display name)
   * Emits: UPDATE_FRIEND event
   */
  async function updateFriendRemarkName(friendId: string, remarkName: string): Promise<void> {
    try {
      loading.value = true;
      error.value = null;

      await invoke('update_friend_display_name', {
        friendId,
        remarkName,
      });
      console.log('[useRelationActions] Remark name updated:', friendId);
      // Backend will emit relation-updated event after WebSocket sync
    } catch (err) {
      console.error('[useRelationActions] Failed to update remark name:', err);
      error.value = err instanceof Error ? err.message : 'Failed to update remark name';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Block a user
   * - If user is a friend: emits BLOCK_FRIEND event (removes from friends, adds to blocked)
   * - If user is a stranger: emits ADD_BLOCK event (adds to blocked)
   */
  async function blockUser(userId: string, displayName?: string): Promise<void> {
    try {
      loading.value = true;
      error.value = null;

      await invoke('block_user', {
        targetUserId: userId,
        displayName,
      });
      console.log('[useRelationActions] User blocked:', userId);
      // Backend will emit relation-updated event after WebSocket sync
    } catch (err) {
      console.error('[useRelationActions] Failed to block user:', err);
      error.value = err instanceof Error ? err.message : 'Failed to block user';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Remove a friend from friends list
   * Emits: REMOVE_FRIEND event
   */
  async function removeFriend(friendId: string): Promise<void> {
    try {
      loading.value = true;
      error.value = null;

      await invoke('remove_friend', {
        friendId,
      });
      console.log('[useRelationActions] Friend removed:', friendId);
      // Backend will emit relation-updated event after WebSocket sync
    } catch (err) {
      console.error('[useRelationActions] Failed to remove friend:', err);
      error.value = err instanceof Error ? err.message : 'Failed to remove friend';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Unblock a user
   * Backend determines which event to emit based on user's original relation:
   * - If user was a blocked friend: emits UNBLOCK_TO_FRIEND (restores to friends list)
   * - If user was a blocked stranger: emits REMOVE_BLOCK (just removes from blocked list)
   */
  async function unblockUser(userId: string): Promise<void> {
    try {
      loading.value = true;
      error.value = null;

      await invoke('unblock_user', {
        targetUserId: userId,
      });
      console.log('[useRelationActions] User unblocked:', userId);
      // Backend will emit relation-updated event after WebSocket sync
      // Event type depends on whether user was originally a friend
    } catch (err) {
      console.error('[useRelationActions] Failed to unblock user:', err);
      error.value = err instanceof Error ? err.message : 'Failed to unblock user';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Hide a blocked user permanently
   * User will be removed from UI and won't appear again
   * Emits: REMOVE_BLOCK event (after WebSocket sync)
   */
  async function hideBlockedUser(userId: string): Promise<void> {
    try {
      loading.value = true;
      error.value = null;

      await invoke('hide_blocked_user', {
        targetUserId: userId,
      });
      console.log('[useRelationActions] Blocked user hidden:', userId);
      // Backend will emit relation-updated event after WebSocket sync
    } catch (err) {
      console.error('[useRelationActions] Failed to hide blocked user:', err);
      error.value = err instanceof Error ? err.message : 'Failed to hide blocked user';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  return {
    // State
    loading,
    error,

    // Actions
    getUserProfileById,
    addFriend,
    updateFriendRemarkName,
    blockUser,
    removeFriend,
    unblockUser,
    hideBlockedUser,
  };
}

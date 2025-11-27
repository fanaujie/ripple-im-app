import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { RelationUser } from '../types/relations';
import { useRelationEvents } from './useRelationEvents';
import { useRelationsState } from './useRelationsState';

interface UIRelationData {
  relations: RelationUser[];
}

/**
 * Composable for displaying relations (friends and blocked users)
 *
 * Features:
 * - Automatically loads initial data from backend
 * - Listens for real-time updates via Tauri events
 * - Provides reactive friends and blockedUsers lists
 * - Includes search functionality
 * - Handles loading and error states
 *
 * Usage:
 * ```typescript
 * const { friends, blockedUsers, loading, error, searchQuery, refresh } = useRelationsDisplay();
 * ```
 */
export function useRelationsDisplay() {
  const loading = ref(false);
  const error = ref<string | null>(null);
  const searchQuery = ref('');

  // Use state management composable
  const { friends, blockedUsers, relationsMap, handleEvent, initialize } = useRelationsState();

  // Set up event listener
  useRelationEvents(handleEvent);

  /**
   * Initialize relations data from backend
   * Called automatically on mount, can also be called manually to refresh
   */
  async function initializeRelations(): Promise<void> {
    loading.value = true;
    error.value = null;

    try {
      console.log('[useRelationsDisplay] Fetching initial data...');
      const data = await invoke<UIRelationData>('get_relations');

      initialize(data.relations);
      console.log(
        `[useRelationsDisplay] Loaded ${data.relations.length} relations`
      );
    } catch (err) {
      console.error('[useRelationsDisplay] Failed to initialize relations:', err);
      error.value = err instanceof Error ? err.message : 'Failed to load relations data';
    } finally {
      loading.value = false;
    }
  }

  /**
   * Filtered friends based on search query
   * Searches in userId, nickName, and remarkName
   */
  const filteredFriends = computed(() => {
    if (!searchQuery.value.trim()) {
      return friends.value;
    }

    const query = searchQuery.value.toLowerCase();
    return friends.value.filter((user) => {
      return (
        user.userId.toLowerCase().includes(query) ||
        user.nickName.toLowerCase().includes(query) ||
        user.remarkName.toLowerCase().includes(query)
      );
    });
  });

  /**
   * Filtered blocked users based on search query
   * Searches in userId and nickName
   */
  const filteredBlockedUsers = computed(() => {
    if (!searchQuery.value.trim()) {
      return blockedUsers.value;
    }

    const query = searchQuery.value.toLowerCase();
    return blockedUsers.value.filter((user) => {
      return (
        user.userId.toLowerCase().includes(query) ||
        user.nickName.toLowerCase().includes(query)
      );
    });
  });

  // Initialize on mount
  onMounted(() => {
    initializeRelations();
  });

  return {
    // State
    friends: filteredFriends,
    blockedUsers: filteredBlockedUsers,
    loading,
    error,

    // Search
    searchQuery,

    // Methods
    refresh: initializeRelations,

    // Raw data (without search filter)
    rawFriends: friends,
    rawBlockedUsers: blockedUsers,

    // Relations map for lookup
    relationsMap,
  };
}

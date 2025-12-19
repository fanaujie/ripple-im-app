import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { RelationUser } from '../types/relations';
import { useRelationEvents } from './useRelationEvents';
import { useRelationsState } from './useRelationsState';

interface UIRelationData {
  users: RelationUser[];
}

/**
 * Composable for displaying relations (friends and blocked users)
 *
 * Features:
 * - Automatically loads initial data from backend
 * - Listens for real-time updates via Tauri events
 * - Provides reactive friends and blockedUsers lists
 * - Handles loading and error states
 *
 * Usage:
 * ```typescript
 * const { friends, blockedUsers, loading, error, refresh } = useRelationsDisplay();
 * ```
 */
export function useRelationsDisplay() {
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Use state management composable
  const { friends, blockedUsers, relationsMap, handleInsert, handleUpdate, handleDelete, handleClearAll, initialize } = useRelationsState();

  // Set up event listeners
  useRelationEvents({
    onInsert: handleInsert,
    onUpdate: handleUpdate,
    onDelete: handleDelete,
    onClearAll: handleClearAll,
  });

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

      initialize(data.users);
      console.log(
        `[useRelationsDisplay] Loaded ${data.users.length} relations`
      );
    } catch (err) {
      console.error('[useRelationsDisplay] Failed to initialize relations:', err);
      error.value = err instanceof Error ? err.message : 'Failed to load relations data';
    } finally {
      loading.value = false;
    }
  }

  // Initialize on mount
  onMounted(() => {
    initializeRelations();
  });

  return {
    // State
    friends,
    blockedUsers,
    loading,
    error,

    // Methods
    refresh: initializeRelations,

    // Relations map for lookup
    relationsMap,
  };
}

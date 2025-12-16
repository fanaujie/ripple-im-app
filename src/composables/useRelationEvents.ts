import { onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  RelationInsertedEvent,
  RelationUpdatedEvent,
  RelationDeletedEvent,
  RelationUser
} from '../types/relations';

/**
 * Event handlers for relation operations
 */
export interface RelationEventHandlers {
  onInsert: (user: RelationUser) => void;
  onUpdate: (user: RelationUser) => void;
  onDelete: (userId: string) => void;
  onClearAll: () => void;
}

/**
 * Composable for listening to relation events from Rust backend
 *
 * Listens to four separate events:
 * - relation-inserted: New relation created (friend added, user blocked)
 * - relation-updated: Existing relation modified (friend → blocked, profile updated)
 * - relation-deleted: Relation removed (unfriend, unblock)
 * - relations-cleared-all: All relations cleared
 *
 * Usage:
 * ```typescript
 * useRelationEvents({
 *   onInsert: (user) => { ... },
 *   onUpdate: (user) => { ... },
 *   onDelete: (userId) => { ... },
 *   onClearAll: () => { ... },
 * });
 * ```
 *
 * @param handlers - Object containing handler functions for each event type
 */
export function useRelationEvents(handlers: RelationEventHandlers) {
  const unlistenFns: UnlistenFn[] = [];

  onMounted(async () => {
    try {
      // Listen to relation-inserted event
      const unlistenInsert = await listen<RelationInsertedEvent>(
        'relation-inserted',
        (tauriEvent) => {
          console.log('[useRelationEvents] Received insert:', {
            userId: tauriEvent.payload.userId,
            flags: tauriEvent.payload.relationFlags,
          });
          handlers.onInsert(tauriEvent.payload);
        }
      );
      unlistenFns.push(unlistenInsert);

      // Listen to relation-updated event
      const unlistenUpdate = await listen<RelationUpdatedEvent>(
        'relation-updated',
        (tauriEvent) => {
          console.log('[useRelationEvents] Received update:', {
            userId: tauriEvent.payload.userId,
            flags: tauriEvent.payload.relationFlags,
          });
          handlers.onUpdate(tauriEvent.payload);
        }
      );
      unlistenFns.push(unlistenUpdate);

      // Listen to relation-deleted event
      const unlistenDelete = await listen<RelationDeletedEvent>(
        'relation-deleted',
        (tauriEvent) => {
          console.log('[useRelationEvents] Received delete:', tauriEvent.payload);
          handlers.onDelete(tauriEvent.payload);
        }
      );
      unlistenFns.push(unlistenDelete);

      // Listen to relations-cleared-all event
      const unlistenClearAll = await listen<void>(
        'relations-cleared-all',
        () => {
          console.log('[useRelationEvents] Received clear all');
          handlers.onClearAll();
        }
      );
      unlistenFns.push(unlistenClearAll);

      console.log('[useRelationEvents] All listeners registered');
    } catch (error) {
      console.error('[useRelationEvents] Failed to register event listeners:', error);
    }
  });

  // Cleanup on unmount
  onUnmounted(() => {
    unlistenFns.forEach((fn) => fn());
    console.log('[useRelationEvents] All listeners unregistered');
  });
}

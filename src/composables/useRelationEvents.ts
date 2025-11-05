import { onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { RelationUpdateEvent } from '../types/relations';
import { RelationAction } from '../types/relations';

/**
 * Composable for listening to relation update events from Rust backend
 *
 * IMPORTANT: Event listener is registered in onMounted() with await to ensure
 * it's fully ready before any events can be emitted. This prevents race conditions.
 *
 * Architecture:
 * 1. Component mounts
 * 2. onMounted() executes
 * 3. await listen() - blocks until listener is registered
 * 4. Listener is now ready to receive events
 * 5. User actions trigger Rust events
 * 6. Events are reliably received and processed
 *
 * Usage:
 * ```typescript
 * useRelationEvents((event) => {
 *   console.log('Received event:', event);
 *   // Handle the event...
 * });
 * ```
 *
 * @param onEvent Callback function to handle relation update events
 */
export function useRelationEvents(
  onEvent: (event: RelationUpdateEvent) => void
) {
  let unlistenFn: UnlistenFn | null = null;

  onMounted(async () => {
    try {
      // ✅ CRITICAL: await ensures listener is fully registered before proceeding
      unlistenFn = await listen<RelationUpdateEvent>(
        'relation-updated',
        (tauriEvent) => {
          const event = tauriEvent.payload;

          // Validate that action is a valid RelationAction
          if (!isValidRelationAction(event.action)) {
            console.warn(
              '[useRelationEvents] Received invalid action code:',
              event.action
            );
            return;
          }

          // Log event for debugging
          console.log('[useRelationEvents] Received event:', {
            action: RelationAction[event.action],
            userId: event.userProfile?.userId,
            flags: event.userProfile?.relationFlags,
          });

          // Call the handler
          onEvent(event);
        }
      );

      console.log('[useRelationEvents] Event listener registered');
    } catch (error) {
      console.error('[useRelationEvents] Failed to register event listener:', error);
    }
  });

  // Cleanup on unmount
  onUnmounted(() => {
    if (unlistenFn) {
      unlistenFn();
      console.log('[useRelationEvents] Event listener unregistered');
    }
  });
}

/**
 * Validate that a numeric action code is a valid RelationAction
 */
function isValidRelationAction(action: number): action is RelationAction {
  const validActions = [
    RelationAction.ADD_FRIEND,
    RelationAction.REMOVE_FRIEND,
    RelationAction.UPDATE_FRIEND,
    RelationAction.ADD_BLOCK,
    RelationAction.REMOVE_BLOCK,
    RelationAction.BLOCK_FRIEND,
    RelationAction.UNBLOCK_TO_FRIEND,
    RelationAction.CLEAR,
  ];
  return validActions.includes(action as RelationAction);
}

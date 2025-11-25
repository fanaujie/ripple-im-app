import { onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ConversationUpdateEvent } from '../../types/chat';

/**
 * Composable for listening to conversation update events from Rust backend
 *
 * Usage:
 * ```ts
 * useConversationEvents((event) => {
 *   // Handle conversation update
 *   conversationsState.handleEvent(event);
 * });
 * ```
 *
 * @param onEvent - Callback function to handle conversation updates
 */
export function useConversationEvents(onEvent: (event: ConversationUpdateEvent) => void) {
  let unlistenFn: UnlistenFn | null = null;

  onMounted(async () => {
    // ✅ await ensures listener is fully registered before continuing
    unlistenFn = await listen<ConversationUpdateEvent>('conversation-updated', (tauriEvent) => {
      console.log('[useConversationEvents] Received event:', tauriEvent.payload);
      onEvent(tauriEvent.payload);
    });

    console.log('[useConversationEvents] Listener registered');
  });

  onUnmounted(() => {
    if (unlistenFn) {
      unlistenFn();
      console.log('[useConversationEvents] Listener unregistered');
    }
  });
}

import { onMounted, onUnmounted, type Ref } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { SSEEventData } from '../../types/chat';

/**
 * Composable for listening to SSE streaming events from Rust backend
 *
 * Only processes SSE events for the currently active conversation.
 *
 * @param activeConversationId - Reactive ref to the currently active conversation ID
 * @param onEvent - Callback function to handle SSE events (only called for active conversation)
 */
export function useSSEEvents(
  activeConversationId: Ref<string | null>,
  onEvent: (event: SSEEventData) => void
) {
  let unlistenFn: UnlistenFn | null = null;

  onMounted(async () => {
    unlistenFn = await listen<SSEEventData>('sse-event', (tauriEvent) => {
      const payload = tauriEvent.payload;

      console.log('[useSSEEvents] Received SSE event:', {
        eventType: payload.eventType,
        conversationId: payload.conversationId,
        activeConversation: activeConversationId.value,
      });

      // Only process SSE events for the currently active conversation
      if (payload.conversationId === activeConversationId.value) {
        onEvent(payload);
      } else {
        console.log('[useSSEEvents] Ignoring SSE event for inactive conversation');
      }
    });

    console.log('[useSSEEvents] Listener registered');
  });

  onUnmounted(() => {
    if (unlistenFn) {
      unlistenFn();
      console.log('[useSSEEvents] Listener unregistered');
    }
  });
}

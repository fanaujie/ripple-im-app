import { onMounted, onUnmounted, type Ref } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { MessageUpdateEvent } from '../../types/chat';

/**
 * Composable for listening to message update events from Rust backend
 *
 * Only processes message events for the currently active conversation
 * to optimize memory usage and performance
 *
 * Usage:
 * ```ts
 * useMessageEvents(selectedConversationId, (event) => {
 *   // Handle message update for active conversation only
 *   messagesState.handleEvent(event);
 * });
 * ```
 *
 * @param activeConversationId - Reactive ref to the currently active conversation ID
 * @param onEvent - Callback function to handle message updates (only called for active conversation)
 */
export function useMessageEvents(
  activeConversationId: Ref<string | null>,
  onEvent: (event: MessageUpdateEvent) => void
) {
  let unlistenFn: UnlistenFn | null = null;

  onMounted(async () => {
    // ✅ await ensures listener is fully registered before continuing
    unlistenFn = await listen<MessageUpdateEvent>('message-updated', (tauriEvent) => {
      const payload = tauriEvent.payload;

      console.log('[useMessageEvents] Received message event:', {
        messageId: payload.message?.messageId,
        conversationId: payload.message?.conversationId,
        action: payload.action,
        activeConversation: activeConversationId.value,
      });

      // Only process messages for the currently active conversation
      // This optimizes memory usage by not storing messages for inactive conversations
      if (payload.message?.conversationId === activeConversationId.value) {
        console.log('[useMessageEvents] Processing message for active conversation');
        onEvent(payload);
      } else {
        console.log('[useMessageEvents] Ignoring message for inactive conversation');
      }
    });

    console.log('[useMessageEvents] Listener registered');
  });

  onUnmounted(() => {
    if (unlistenFn) {
      unlistenFn();
      console.log('[useMessageEvents] Listener unregistered');
    }
  });
}

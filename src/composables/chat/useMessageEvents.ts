import { onMounted, onUnmounted, type Ref } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { MessageUpdateEvent } from '../../types/chat';

/**
 * Composable for listening to message update events from Rust backend
 *
 * Processes all message events and stores them in the appropriate conversation
 *
 * Usage:
 * ```ts
 * useMessageEvents(selectedConversationId, (event) => {
 *   // Handle message update for any conversation
 *   messagesState.handleEvent(event);
 * });
 * ```
 *
 * @param activeConversationId - Reactive ref to the currently active conversation ID (kept for compatibility)
 * @param onEvent - Callback function to handle message updates
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
      });

      // Process all messages regardless of active conversation
      // Messages are automatically stored in the correct conversation by conversationId
      onEvent(payload);
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

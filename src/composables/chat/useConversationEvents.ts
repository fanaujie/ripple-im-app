import { onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  ConversationInsertedEvent,
  ConversationUpdatedEvent,
  ConversationDeletedEvent,
  ConversationReceivedMessageEvent,
  ConversationItem
} from '../../types/chat';

/**
 * Event handlers for conversation operations
 */
export interface ConversationEventHandlers {
  onInsert: (conversation: ConversationItem) => void;
  onUpdate: (conversation: ConversationItem) => void;
  onDelete: (conversationId: string) => void;
  onClearAll: () => void;
  onReceivedNewMessage: (event: ConversationReceivedMessageEvent) => void;
}

/**
 * Composable for listening to conversation events from Rust backend
 *
 * Listens to five separate events:
 * - conversation-inserted: New conversation created
 * - conversation-updated: Existing conversation modified
 * - conversations-deleted: Conversation removed
 * - conversations-cleared-all: All conversations cleared
 * - conversation-received-new-message: New message preview update
 *
 * Usage:
 * ```ts
 * useConversationEvents({
 *   onInsert: (conversation) => { ... },
 *   onUpdate: (conversation) => { ... },
 *   onDelete: (conversationId) => { ... },
 *   onClearAll: () => { ... },
 *   onReceivedNewMessage: (event) => { ... },
 * });
 * ```
 *
 * @param handlers - Object containing handler functions for each event type
 */
export function useConversationEvents(handlers: ConversationEventHandlers) {
  const unlistenFns: UnlistenFn[] = [];

  onMounted(async () => {
    // Listen to conversation-inserted event
    const unlistenInsert = await listen<ConversationInsertedEvent>(
      'conversation-inserted',
      (tauriEvent) => {
        console.log('[useConversationEvents] Received insert:', tauriEvent.payload);
        handlers.onInsert(tauriEvent.payload);
      }
    );
    unlistenFns.push(unlistenInsert);

    // Listen to conversation-updated event
    const unlistenUpdate = await listen<ConversationUpdatedEvent>(
      'conversation-updated',
      (tauriEvent) => {
        console.log('[useConversationEvents] Received update:', tauriEvent.payload);
        handlers.onUpdate(tauriEvent.payload);
      }
    );
    unlistenFns.push(unlistenUpdate);

    // Listen to conversations-deleted event
    const unlistenDelete = await listen<ConversationDeletedEvent>(
      'conversations-deleted',
      (tauriEvent) => {
        console.log('[useConversationEvents] Received delete:', tauriEvent.payload);
        handlers.onDelete(tauriEvent.payload);
      }
    );
    unlistenFns.push(unlistenDelete);

    // Listen to conversations-cleared-all event
    const unlistenClearAll = await listen<void>(
      'conversations-cleared-all',
      () => {
        console.log('[useConversationEvents] Received clear all');
        handlers.onClearAll();
      }
    );
    unlistenFns.push(unlistenClearAll);

    // Listen to conversation-received-new-message event
    const unlistenReceivedMessage = await listen<ConversationReceivedMessageEvent>(
      'conversation-received-new-message',
      (tauriEvent) => {
        console.log('[useConversationEvents] Received new message preview:', {
          conversationId: tauriEvent.payload.conversationId,
          message: tauriEvent.payload.message,
          unreadCount: tauriEvent.payload.unreadCount,
          timestamp: tauriEvent.payload.timestamp,
        });
        handlers.onReceivedNewMessage(tauriEvent.payload);
      }
    );
    unlistenFns.push(unlistenReceivedMessage);

    console.log('[useConversationEvents] All listeners registered');
  });

  onUnmounted(() => {
    unlistenFns.forEach((fn) => fn());
    console.log('[useConversationEvents] All listeners unregistered');
  });
}

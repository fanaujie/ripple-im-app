import { ref } from 'vue';
import type { Message, MessageUpdateEvent, UIMessageItem } from '../../types/chat';
import { MessageAction, sortMessagesByMessageId } from '../../types/chat';

/**
 * Composable for managing messages state
 *
 * Maintains messages grouped by conversationId
 * Handles message update events from backend
 *
 * @returns State and update handler
 */
export function useMessagesState() {
  // Messages grouped by conversationId
  const messagesByConversation = ref<Record<string, Message[]>>({});

  /**
   * Convert UIMessageItem from WebSocket to Message format for storage
   */
  function convertUIMessageToMessage(uiMessage: UIMessageItem): Message {
    return {
      messageId: uiMessage.messageId,
      conversationId: uiMessage.conversationId,
      senderId: uiMessage.senderId,
      sendTimestamp: uiMessage.timestamp,
      messageType: uiMessage.messageType ?? 1, // Default to single message if not provided
      text: uiMessage.content,
      commandType: uiMessage.commandType,
      commandData: uiMessage.commandData,
      fileUrl: uiMessage.fileUrl,
      fileName: uiMessage.fileName,
    };
  }

  /**
   * Handle a message update event from the backend
   */
  function handleEvent(event: MessageUpdateEvent): void {
    const { action, message: uiMessage } = event;

    // CLEAR action - reset everything
    if (action === MessageAction.CLEAR) {
      messagesByConversation.value = {};
      console.log('[useMessagesState] Cleared all messages');
      return;
    }

    // All other actions require message data
    if (!uiMessage) {
      console.warn('[useMessagesState] Received event without message data:', action);
      return;
    }

    // Convert UIMessageItem to Message format
    const message = convertUIMessageToMessage(uiMessage);

    switch (action) {
      case MessageAction.ADD:
        addMessage(message);
        console.log('[useMessagesState] Added message:', message.messageId);
        break;

      case MessageAction.UPDATE:
        updateMessage(message);
        console.log('[useMessagesState] Updated message:', message.messageId);
        break;

      case MessageAction.DELETE:
        deleteMessage(message.conversationId, message.messageId);
        console.log('[useMessagesState] Deleted message:', message.messageId);
        break;

      default:
        console.warn('[useMessagesState] Unknown action:', action);
    }
  }

  /**
   * Load messages for a specific conversation
   */
  function loadMessages(conversationId: string, messages: Message[]): void {
    messagesByConversation.value[conversationId] = sortMessagesByMessageId(messages);
    console.log(`[useMessagesState] Loaded ${messages.length} messages for ${conversationId}`);
  }

  /**
   * Get messages for a specific conversation
   */
  function getMessages(conversationId: string): Message[] {
    return messagesByConversation.value[conversationId] || [];
  }

  /**
   * Add a new message to its conversation
   */
  function addMessage(message: Message): void {
    const { conversationId } = message;
    const messages = messagesByConversation.value[conversationId] || [];

    // Check if message already exists
    const exists = messages.some((m) => m.messageId === message.messageId);
    if (exists) {
      console.warn('[useMessagesState] Message already exists:', message.messageId);
      return;
    }

    // Add and sort
    messages.push(message);
    messagesByConversation.value[conversationId] = sortMessagesByMessageId(messages);
  }

  /**
   * Update an existing message
   */
  function updateMessage(message: Message): void {
    const { conversationId, messageId } = message;
    const messages = messagesByConversation.value[conversationId];

    if (!messages) {
      console.warn('[useMessagesState] Conversation not found:', conversationId);
      return;
    }

    const index = messages.findIndex((m) => m.messageId === messageId);
    if (index >= 0) {
      messages.splice(index, 1, message);
    } else {
      console.warn('[useMessagesState] Message not found:', messageId);
    }
  }

  /**
   * Delete a message
   */
  function deleteMessage(conversationId: string, messageId: string): void {
    const messages = messagesByConversation.value[conversationId];

    if (!messages) {
      console.warn('[useMessagesState] Conversation not found:', conversationId);
      return;
    }

    const index = messages.findIndex((m) => m.messageId === messageId);
    if (index >= 0) {
      messages.splice(index, 1);
    }
  }

  /**
   * Prepend older messages to the beginning of the message list
   * Used for pagination when loading historical messages
   */
  function prependMessages(conversationId: string, messages: Message[]): void {
    const existing = messagesByConversation.value[conversationId] || [];

    // Filter out duplicates
    const existingIds = new Set(existing.map((m) => m.messageId));
    const newMessages = messages.filter((m) => !existingIds.has(m.messageId));

    if (newMessages.length === 0) {
      console.log(`[useMessagesState] No new messages to prepend for ${conversationId}`);
      return;
    }

    // Combine and sort
    const combined = [...newMessages, ...existing];
    messagesByConversation.value[conversationId] = sortMessagesByMessageId(combined);

    console.log(
      `[useMessagesState] Prepended ${newMessages.length} older messages for ${conversationId}`
    );
  }

  /**
   * Get the oldest message ID for a conversation (used for pagination)
   */
  function getOldestMessageId(conversationId: string): string | null {
    const messages = messagesByConversation.value[conversationId];
    if (!messages || messages.length === 0) {
      return null;
    }
    // Messages are sorted from old to new, so first message is oldest
    return messages[0].messageId;
  }

  /**
   * Clear messages for a specific conversation
   */
  function clearConversationMessages(conversationId: string): void {
    delete messagesByConversation.value[conversationId];
  }

  return {
    messagesByConversation,
    handleEvent,
    loadMessages,
    getMessages,
    prependMessages,
    getOldestMessageId,
    clearConversationMessages,
  };
}

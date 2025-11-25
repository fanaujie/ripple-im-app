import { invoke } from '@tauri-apps/api/core';
import type { Message } from '../../types/chat';

/**
 * Composable for chat-related actions (commands to Rust backend)
 *
 * Provides methods to:
 * - Send messages
 * - Mark conversations as read
 * - Load messages for a conversation
 *
 * @returns Action methods
 */
export function useChatActions() {
  /**
   * Send a text message to a conversation
   *
   * @param senderId - Current user's ID
   * @param conversationId - Target conversation ID
   * @param receiverId - Receiver's user ID
   * @param content - Message text content
   * @returns Promise resolving to the message ID
   */
  async function sendMessage(
    senderId: string,
    conversationId: string,
    receiverId: string,
    content: string
  ): Promise<string> {
    try {
      const messageId = await invoke<string>('send_message', {
        senderId,
        conversationId,
        receiverId,
        textContent: content,
        fileUrl: null,
        fileName: null,
      });

      console.log('[useChatActions] Message sent:', messageId);
      return messageId;
    } catch (error) {
      console.error('[useChatActions] Failed to send message:', error);
      throw error;
    }
  }

  /**
   * Mark a conversation as read up to a specific message
   *
   * @param conversationId - Conversation ID
   * @param messageId - Last read message ID
   */
  async function markConversationRead(
    conversationId: string,
    messageId: string
  ): Promise<void> {
    try {
      await invoke('mark_last_read_message_id', {
        conversationId,
        messageId,
      });

      console.log(
        '[useChatActions] Marked conversation as read:',
        conversationId,
        messageId
      );
    } catch (error) {
      console.error('[useChatActions] Failed to mark as read:', error);
      throw error;
    }
  }

  /**
   * Load all messages for a conversation
   * Note: API has a max limit of 200 messages per request, so we paginate
   *
   * @param conversationId - Conversation ID
   * @returns Promise resolving to array of all messages
   */
  async function loadConversationMessages(conversationId: string): Promise<Message[]> {
    try {
      console.log(`[useChatActions] Loading all messages for conversation: ${conversationId}`);

      const allMessages: Message[] = [];
      let messageId = '0';
      const API_MAX_SIZE = 200; // API max limit per request
      let hasMoreMessages = true;

      while (hasMoreMessages) {
        console.log(`[useChatActions] Fetching messages starting from messageId: ${messageId}`);

        const result = await invoke<{ messages: Message[] }>('read_messages', {
          conversationId,
          messageId,
          readSize: API_MAX_SIZE,
        });

        const { messages } = result;
        console.log(
          `[useChatActions] Batch loaded ${messages.length} messages for ${conversationId}`
        );

        if (messages.length === 0) {
          hasMoreMessages = false;
          break;
        }

        // Avoid duplicates: only add messages we haven't seen yet
        const existingIds = new Set(allMessages.map((m) => m.messageId));
        const newMessages = messages.filter((m) => !existingIds.has(m.messageId));

        if (newMessages.length === 0) {
          // No new messages, stop paginating
          hasMoreMessages = false;
          break;
        }

        allMessages.push(...newMessages);

        // If we got fewer than API_MAX_SIZE, we've reached the end
        if (messages.length < API_MAX_SIZE) {
          hasMoreMessages = false;
        } else {
          // Set messageId to the last message's ID for next iteration
          messageId = messages[messages.length - 1].messageId;
        }
      }

      console.log(
        `[useChatActions] Finished loading all ${allMessages.length} messages for ${conversationId}`
      );
      return allMessages;
    } catch (error) {
      console.error('[useChatActions] Failed to load messages:', error);
      throw error;
    }
  }

  /**
   * Load latest N messages for a conversation (for initial load)
   *
   * @param conversationId - Conversation ID
   * @param limit - Number of messages to load (default: 50)
   * @returns Promise resolving to array of latest messages
   */
  async function loadLatestMessages(
    conversationId: string,
    limit: number = 50
  ): Promise<Message[]> {
    try {
      console.log(
        `[useChatActions] Loading latest ${limit} messages for conversation: ${conversationId}`
      );

      const result = await invoke<{ messages: Message[] }>('read_latest_messages', {
        conversationId,
        readSize: limit,
      });

      const { messages } = result;
      console.log(
        `[useChatActions] Loaded ${messages.length} latest messages for ${conversationId}`
      );

      return messages;
    } catch (error) {
      console.error('[useChatActions] Failed to load latest messages:', error);
      throw error;
    }
  }

  /**
   * Load older messages before a specific message ID (for pagination)
   *
   * @param conversationId - Conversation ID
   * @param beforeMessageId - Load messages before this message ID
   * @param limit - Number of messages to load (default: 50)
   * @returns Promise resolving to array of older messages
   */
  async function loadOlderMessages(
    conversationId: string,
    beforeMessageId: string,
    limit: number = 50
  ): Promise<Message[]> {
    try {
      console.log(
        `[useChatActions] Loading ${limit} messages before ID ${beforeMessageId} for conversation: ${conversationId}`
      );

      const result = await invoke<{ messages: Message[] }>('read_messages_before', {
        conversationId,
        beforeMessageId,
        readSize: limit,
      });

      const { messages } = result;
      console.log(
        `[useChatActions] Loaded ${messages.length} older messages for ${conversationId}`
      );

      return messages;
    } catch (error) {
      console.error('[useChatActions] Failed to load older messages:', error);
      throw error;
    }
  }

  return {
    sendMessage,
    markConversationRead,
    loadConversationMessages,
    loadLatestMessages,
    loadOlderMessages,
  };
}

import type { RelationUser } from './relations';

/**
 * Conversation Event Actions
 * These action codes match the Rust backend's conversation_event_action constants
 */
export enum ConversationAction {
  /** Create new conversation */
  CREATE = 0,
  /** New message received - upserts conversation */
  NEW_MESSAGE = 1,
  /** Mark messages as read */
  READ_MESSAGE = 2,
  /** Update conversation name */
  UPDATE_NAME = 3,
  /** Update conversation avatar */
  UPDATE_AVATAR = 4,
  /** Delete a conversation */
  DELETE = 5,
  /** Clear all conversations (used during full sync) */
  CLEAR = -1,
}

/**
 * Message Event Actions
 * These action codes match the Rust backend's message_action constants
 */
export enum MessageAction {
  /** Add a new message */
  ADD = 0,
  /** Update message information */
  UPDATE = 1,
  /** Delete a message */
  DELETE = 2,
  /** Clear all messages (used during full sync) */
  CLEAR = -1,
}

/**
 * ConversationItem represents a chat conversation
 * Matches the Rust ConversationItem structure
 */
export interface ConversationItem {
  /** Unique conversation identifier */
  conversationId: string;
  /** Peer user ID for one-on-one chat */
  peerId?: string;
  /** Group ID for group chat */
  groupId?: string;
  /** ID of the last message in this conversation */
  lastMessageId?: string;
  /** Content preview of the last message */
  lastMessage?: string;
  /** Timestamp of the last message (Unix timestamp in milliseconds) */
  lastMessageTimestamp?: number;
  /** ID of the last read message by current user */
  lastReadMessageId?: string;
  /** Number of unread messages (calculated based on lastReadMessageId) */
  unreadCount: number;
  /** Conversation display name (for groups or custom names) */
  name?: string;
  /** Conversation avatar URL (for groups or custom avatars) */
  avatar?: string;
}

/**
 * ConversationDisplay extends ConversationItem with display-specific data
 * Adds peer profile information for UI display
 */
export interface ConversationDisplay extends ConversationItem {
  /** Populated peer profile information (for UI display) */
  peerProfile?: RelationUser;
}

/**
 * Message represents a single chat message from API
 * Matches the Rust MessageItem structure (from read_messages API)
 */
export interface Message {
  /** Unique message identifier */
  messageId: string;
  /** ID of the conversation this message belongs to */
  conversationId: string;
  /** ID of the user who sent the message */
  senderId: string;
  /** ID of the user who receives the message (for one-on-one chat) */
  receiverId?: string;
  /** ID of the group (for group chat) */
  groupId?: string;
  /** Message timestamp (Unix timestamp in milliseconds) */
  sendTimestamp: number;
  /** Message text content */
  textContent?: string;
  /** File URL if message contains file */
  fileUrl?: string;
  /** File name if message contains file */
  fileName?: string;
}

/**
 * UIMessageItem represents a message from WebSocket events
 * Matches the Rust UIMessageItem structure (simplified for real-time events)
 */
export interface UIMessageItem {
  /** Unique message identifier */
  messageId: string;
  /** ID of the conversation this message belongs to */
  conversationId: string;
  /** ID of the user who sent the message */
  senderId: string;
  /** Message content */
  content: string;
  /** Message timestamp (Unix timestamp in milliseconds) */
  timestamp: number;
}

/**
 * ConversationUpdateEvent is emitted from Rust backend via Tauri events
 *
 * Event flow:
 * 1. WebSocket receives conversation update from server
 * 2. IncrementalSyncManager processes the update
 * 3. Changes are written to storage
 * 4. This event is emitted to frontend
 * 5. Frontend updates UI reactively
 */
export interface ConversationUpdateEvent {
  /** Action type indicating what kind of update this is */
  action: ConversationAction;
  /** Conversation data for the update (null for CLEAR action) */
  conversation: ConversationItem | null;
}

/**
 * MessageUpdateEvent is emitted from Rust backend via Tauri events
 *
 * Event flow:
 * 1. WebSocket receives new message from server
 * 2. Message is stored in storage
 * 3. This event is emitted to frontend
 * 4. Frontend updates message list reactively
 */
export interface MessageUpdateEvent {
  /** Action type indicating what kind of update this is */
  action: MessageAction;
  /** Message data for the update (null for CLEAR action) - uses UIMessageItem from WebSocket */
  message: UIMessageItem | null;
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Check if conversation is one-on-one chat
 */
export function isOneOnOneChat(conversation: ConversationItem): boolean {
  return !!conversation.peerId && !conversation.groupId;
}

/**
 * Check if conversation is group chat
 */
export function isGroupChat(conversation: ConversationItem): boolean {
  return !!conversation.groupId;
}

/**
 * Get conversation display name
 * Backend provides name directly - use it or fallback to conversationId
 */
export function getConversationDisplayName(
  conversation: ConversationDisplay
): string {
  return conversation.name || conversation.conversationId;
}

/**
 * Get conversation avatar URL
 * Backend provides avatar directly
 */
export function getConversationAvatar(
  conversation: ConversationDisplay
): string | undefined {
  return conversation.avatar;
}

/**
 * Sort conversations by lastMessageTimestamp (descending)
 * Most recent conversation appears first
 */
export function sortConversationsByTime(
  conversations: ConversationDisplay[]
): ConversationDisplay[] {
  return [...conversations].sort((a, b) => {
    const timeA = a.lastMessageTimestamp || 0;
    const timeB = b.lastMessageTimestamp || 0;
    return timeB - timeA;
  });
}

/**
 * Sort messages by messageId (ascending)
 * Messages are sorted by their ID which should be monotonically increasing
 */
export function sortMessagesByMessageId(messages: Message[]): Message[] {
  return [...messages].sort((a, b) => a.messageId.localeCompare(b.messageId));
}

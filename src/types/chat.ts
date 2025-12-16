import type { RelationUser } from './relations';

/**
 * Conversation Event Actions
 * These action codes match the Rust backend's conversation_event_action constants
 */
export enum ConversationAction {
  /** Create new conversation */
  CREATE = 1,
  /** New message received - upserts conversation */
  NEW_MESSAGE = 2,
  /** Mark messages as read */
  READ_MESSAGE = 3,
  /** Update conversation name */
  UPDATE_NAME = 4,
  /** Update conversation avatar */
  UPDATE_AVATAR = 5,
  /** Update both name and avatar */
  UPDATE_NAME_AVATAR = 6,
  /** Delete a conversation */
  DELETE = 7,
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
 * Message Type enumeration
 */
export enum MessageType {
  /** Single text/file message */
  SINGLE = 1,
  /** Group command message */
  GROUP_COMMAND = 2,
}

/**
 * Command Type enumeration for group commands
 */
export enum CommandType {
  /** Member joined the group */
  MEMBER_JOIN = 1,
  /** Member left the group */
  MEMBER_QUIT = 2,
  /** Group info updated */
  INFO_UPDATE = 3,
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
  /** Message timestamp (Unix timestamp in milliseconds) - converted from UTC seconds */
  sendTimestamp: number;
  /** Message type (1=single, 2=group command) */
  messageType: number;
  /** Message text content */
  text?: string;
  /** File URL if message contains file */
  fileUrl?: string;
  /** File name if message contains file */
  fileName?: string;
  /** Command type for group commands (1=MEMBER_JOIN, 2=MEMBER_QUIT) */
  commandType?: number;
  /** Command data payload */
  commandData?: string;
}

/**
 * Message from backend (before timestamp conversion)
 * Backend sends UTC seconds as string to avoid precision loss
 */
export interface MessageFromBackend {
  messageId: string;
  conversationId: string;
  senderId: string;
  receiverId?: string;
  groupId?: string;
  /** UTC timestamp in seconds (as string to preserve precision) */
  sendTimestamp: string;
  messageType: number;
  text?: string;
  fileUrl?: string;
  fileName?: string;
  commandType?: number;
  commandData?: string;
}

/**
 * Convert backend message (UTC seconds string) to frontend message (milliseconds number)
 */
export function convertBackendMessage(backendMsg: MessageFromBackend): Message {
  // Parse UTC seconds string to number, then convert to milliseconds
  const timestampSeconds = parseInt(backendMsg.sendTimestamp, 10);
  const timestampMs = isNaN(timestampSeconds) ? 0 : timestampSeconds * 1000;

  return {
    ...backendMsg,
    sendTimestamp: timestampMs,
  };
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
  /** Message type (1=single, 2=group command) */
  messageType?: number;
  /** Command type for group commands (1=MEMBER_JOIN, 2=MEMBER_QUIT, 3=INFO_UPDATE) */
  commandType?: number;
  /** Command data payload */
  commandData?: string;
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
 *
 * @deprecated Use separate event types (ConversationInsertedEvent, ConversationUpdatedEvent, etc.) instead
 */
export interface ConversationUpdateEvent {
  /** Action type indicating what kind of update this is */
  action: ConversationAction;
  /** Conversation data for the update (null for CLEAR action) */
  conversation: ConversationItem | null;
}

/**
 * Event payload for conversation-inserted event
 * Emitted when a new conversation is created
 */
export type ConversationInsertedEvent = ConversationItem;

/**
 * Event payload for conversation-updated event
 * Emitted when an existing conversation is modified
 */
export type ConversationUpdatedEvent = ConversationItem;

/**
 * Event payload for conversations-deleted event
 * Emitted when a conversation is deleted (ID only)
 */
export type ConversationDeletedEvent = string;

/**
 * Event payload for conversations-cleared-all event
 * Emitted when all conversations are cleared (no data needed)
 */
export type ConversationClearedAllEvent = void;

/**
 * Event payload for conversation-received-new-message event
 * Emitted when a new message arrives for a conversation (for preview update)
 */
export interface ConversationReceivedMessageEvent {
  /** ID of the conversation that received the message */
  conversationId: string;
  /** Updated unread count for the conversation */
  unreadCount: number;
  /** Message content preview (text or description) */
  message: string;
  /** Message timestamp (UTC seconds as string) */
  timestamp: string;
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

// ============================================================================
// Group Creation Types
// ============================================================================

/**
 * Parameters for creating a new group
 */
export interface CreateGroupParams {
  /** ID of the user creating the group */
  senderId: string;
  /** Name of the group (1-100 characters) */
  groupName: string;
  /** Array of member user IDs (minimum 1 member) */
  memberIds: string[];
}

/**
 * Result from creating a group
 */
export interface CreateGroupResult {
  /** ID of the newly created group */
  groupId: string;
}

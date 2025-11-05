/**
 * Relation Event Actions
 * These action codes match the Rust backend's relation_event_action constants
 */
export enum RelationAction {
  /** Add a new friend to the friends list */
  ADD_FRIEND = 0,
  /** Remove a friend from the friends list */
  REMOVE_FRIEND = 1,
  /** Update friend information (remarkName, nickName, avatar) */
  UPDATE_FRIEND = 2,
  /** Add a blocked user (stranger, not a friend) */
  ADD_BLOCK = 3,
  /**
   * Remove a blocked user (unblock stranger)
   * User was blocked but never a friend, simply remove from blocked list
   */
  REMOVE_BLOCK = 4,
  /**
   * Block a friend
   * User was a friend, now move from friends list to blocked list
   */
  BLOCK_FRIEND = 5,
  /**
   * Unblock a friend (restore friendship)
   * User was a blocked friend, now restore to friends list
   */
  UNBLOCK_TO_FRIEND = 6,
  /** Clear all relations (used during full sync) */
  CLEAR = -1,
}

/**
 * Relation Flags
 * Bit flags used to identify user's relation status
 */
export enum RelationFlags {
  /** User is a friend */
  FRIEND = 0b0001,
  /** User is blocked */
  BLOCKED = 0b0010,
  /** User is hidden (permanently removed from UI) */
  HIDDEN = 0b0100,
}

/**
 * RelationUser represents a user in the relations system
 * Contains all necessary display information
 */
export interface RelationUser {
  /** Unique user identifier */
  userId: string;
  /** User's display nickname */
  nickName: string;
  /** User's avatar URL (optional) */
  avatar?: string;
  /** Custom remark name set by current user (for friends) */
  remarkName: string;
  /** Bit flags indicating relation status (FRIEND | BLOCKED | HIDDEN) */
  relationFlags: number;
}

/**
 * RelationsData structure for storing friends and blocked users
 */
export interface RelationsData {
  friends: RelationUser[];
  blockedUsers: RelationUser[];
}

/**
 * RelationUpdateEvent is emitted from Rust backend via Tauri events
 *
 * Event flow:
 * 1. Frontend calls Tauri command (e.g., add_friend, block_user)
 * 2. Command sends API request to server
 * 3. Server processes and sends WebSocket push notification
 * 4. IncrementalSyncManager receives push and syncs changes
 * 5. Changes are written to storage
 * 6. This event is emitted to frontend
 * 7. Frontend updates UI reactively
 *
 * IMPORTANT: Field name is camelCase (userProfile) to match Rust serialization
 * Rust uses #[serde(rename = "userProfile")] for JSON serialization
 */
export interface RelationUpdateEvent {
  /** Action type indicating what kind of update this is */
  action: RelationAction;
  /**
   * User data for the update
   * - Present for most operations (add, update, block, unblock)
   * - May only contain userId for delete operations
   * - Contains complete user data after fetching from storage
   *
   * Note: camelCase to match Rust's serde serialization
   */
  userProfile: RelationUser | null;
}

/**
 * User action types for UI operations
 */
export type UserActionType =
  | 'rename'
  | 'block'
  | 'remove'
  | 'unblock'
  | 'hide';

/**
 * User action definition for UI menu items
 */
export interface UserAction {
  type: UserActionType;
  label: string;
  icon: string;
  dangerous?: boolean;
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Check if user is a friend
 */
export function isFriend(user: RelationUser): boolean {
  return (user.relationFlags & RelationFlags.FRIEND) !== 0;
}

/**
 * Check if user is blocked
 */
export function isBlocked(user: RelationUser): boolean {
  return (user.relationFlags & RelationFlags.BLOCKED) !== 0;
}

/**
 * Check if user is hidden
 */
export function isHidden(user: RelationUser): boolean {
  return (user.relationFlags & RelationFlags.HIDDEN) !== 0;
}

/**
 * Get display name for a user (remarkName if available, otherwise nickName)
 */
export function getDisplayName(user: RelationUser): string {
  return user.remarkName || user.nickName;
}

/**
 * Check if user should be shown in friends list
 * Must have FRIEND flag and not be blocked
 */
export function shouldShowInFriendsList(user: RelationUser): boolean {
  return isFriend(user) && !isBlocked(user);
}

/**
 * Check if user should be shown in blocked list
 * Must have BLOCKED flag and not be hidden
 */
export function shouldShowInBlockedList(user: RelationUser): boolean {
  return isBlocked(user) && !isHidden(user);
}

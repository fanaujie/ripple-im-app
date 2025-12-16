/**
 * GroupMember represents a member of a group
 * Matches the Rust GroupMember structure
 */
export interface GroupMemberData {
  /** User ID */
  userId: string;
  /** User's nick name */
  name: string;
  /** User's avatar URL */
  avatar?: string;
}

/**
 * Command data for group member join/quit events
 */
export interface GroupCommandData {
  /** Name of the member who joined/quit */
  memberName?: string;
  /** Alternative field name for member name */
  name?: string;
}

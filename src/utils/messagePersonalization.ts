import type { RelationUser } from '../types/relations';
import { getDisplayName } from '../types/relations';
import type { GroupMemberData } from '../types/group';

/**
 * Regex pattern for matching template text: {{userId}} action
 * Captures: group 1 = userId, group 2 = action text (e.g., "sent an image")
 */
const TEMPLATE_REGEX = /\{\{(\d+)\}\}\s+(.+)/;

/**
 * Context for resolving display names during message personalization
 */
export interface PersonalizationContext {
  /** Current user's ID */
  currentUserId: string;
  /** Relations map for 1-on-1 chat name lookups */
  relations: Map<string, RelationUser>;
  /** Group members map for group chat name lookups (optional) */
  groupMembers?: Map<string, GroupMemberData>;
}

/**
 * Personalize message preview text by replacing user ID templates with display names
 *
 * Template format: {{userId}} action (e.g., "{{12345}} sent an image")
 *
 * Rules:
 * - If userId matches current user: return action only (e.g., "sent an image")
 * - If userId is different: resolve display name and return "{name} {action}"
 * - If user not found: fallback to "Someone {action}"
 * - If no template pattern: return text unchanged
 *
 * @param text - The message text (may contain {{userId}} template)
 * @param context - Context for resolving display names
 * @returns Personalized display text
 */
export function personalizeMessageText(
  text: string,
  context: PersonalizationContext
): string {
  const match = TEMPLATE_REGEX.exec(text);

  if (!match) {
    // No template pattern found, return text as-is
    return text;
  }

  const userId = match[1];
  const action = match[2]; // e.g., "sent an image"

  // Current user: show action only
  if (userId === context.currentUserId) {
    return action;
  }

  // Other user: resolve display name
  const displayName = resolveDisplayName(userId, context);
  return `${displayName} ${action}`;
}

/**
 * Resolve display name for a user ID from relations or group members
 *
 * Priority:
 * 1. Group members (if provided) - uses member name
 * 2. Relations map - uses remarkName or nickName
 * 3. Fallback to "Someone"
 */
function resolveDisplayName(
  userId: string,
  context: PersonalizationContext
): string {
  // Try group members first (if available)
  if (context.groupMembers) {
    const member = context.groupMembers.get(userId);
    if (member) {
      return member.name;
    }
  }

  // Try relations map
  const relation = context.relations.get(userId);
  if (relation) {
    return getDisplayName(relation);
  }

  // Fallback
  return 'Someone';
}

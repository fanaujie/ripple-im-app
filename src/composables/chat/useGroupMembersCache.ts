import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { GroupMemberData } from '../../types/group';

/**
 * Sender info for display in chat messages
 */
export interface SenderInfo {
  name: string;
  avatar?: string;
}

// Global cache: groupId -> Map<userId, GroupMemberData>
const groupMembersCache = ref<Map<string, Map<string, GroupMemberData>>>(new Map());

/**
 * Composable for caching group member data
 *
 * Provides efficient lookup of sender info (avatar + name) for group chat messages
 * without making redundant API calls.
 */
export function useGroupMembersCache() {
  /**
   * Fetch and cache group members for a specific group
   * Only fetches if not already cached
   *
   * @param groupId The group ID to fetch members for
   * @returns Promise resolving to the cached members map
   */
  async function fetchGroupMembers(groupId: string): Promise<Map<string, GroupMemberData>> {
    // Return cached data if available
    if (groupMembersCache.value.has(groupId)) {
      return groupMembersCache.value.get(groupId)!;
    }

    try {
      const members = await invoke<GroupMemberData[]>('get_group_members', {
        groupId,
      });

      // Build lookup map by userId
      const membersMap = new Map<string, GroupMemberData>();
      for (const member of members) {
        membersMap.set(member.userId, member);
      }

      // Cache the result
      groupMembersCache.value.set(groupId, membersMap);
      console.log('[useGroupMembersCache] Cached members for group:', groupId, 'count:', members.length);

      return membersMap;
    } catch (err) {
      console.error('[useGroupMembersCache] Failed to fetch group members:', err);
      // Return empty map on error to avoid breaking the UI
      return new Map();
    }
  }

  /**
   * Get sender info (name + avatar) for a user in a specific group
   *
   * @param groupId The group ID
   * @param userId The sender's user ID
   * @returns SenderInfo or undefined if not found
   */
  function getSenderInfo(groupId: string, userId: string): SenderInfo | undefined {
    const membersMap = groupMembersCache.value.get(groupId);
    if (!membersMap) {
      return undefined;
    }

    const member = membersMap.get(userId);
    if (!member) {
      return undefined;
    }

    return {
      name: member.name,
      avatar: member.avatar,
    };
  }

  /**
   * Check if a group's members are cached
   */
  function isGroupCached(groupId: string): boolean {
    return groupMembersCache.value.has(groupId);
  }

  /**
   * Clear cache for a specific group (useful when member list changes)
   */
  function clearGroupCache(groupId: string): void {
    groupMembersCache.value.delete(groupId);
  }

  /**
   * Clear all cached data
   */
  function clearAllCache(): void {
    groupMembersCache.value.clear();
  }

  return {
    fetchGroupMembers,
    getSenderInfo,
    isGroupCached,
    clearGroupCache,
    clearAllCache,
  };
}

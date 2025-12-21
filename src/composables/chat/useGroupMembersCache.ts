import { ref, triggerRef, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { GroupMemberData } from '../../types/group';
import { MessageType, CommandType, type MessageUpdateEvent } from '../../types/chat';

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
      // Trigger reactivity so components using getGroupMemberCount re-render
      triggerRef(groupMembersCache);
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
   * Get member count for a specific group
   * Returns undefined if group is not cached
   */
  function getGroupMemberCount(groupId: string): number | undefined {
    const membersMap = groupMembersCache.value.get(groupId);
    return membersMap?.size;
  }

  /**
   * Clear cache for a specific group (useful when member list changes)
   */
  function clearGroupCache(groupId: string): void {
    groupMembersCache.value.delete(groupId);
    triggerRef(groupMembersCache);
  }

  /**
   * Clear all cached data
   */
  function clearAllCache(): void {
    groupMembersCache.value.clear();
    triggerRef(groupMembersCache);
  }

  /**
   * Refresh group members (clear cache and re-fetch)
   * Used when group members change (join/leave events)
   */
  async function refreshGroupMembers(groupId: string): Promise<void> {
    // Clear existing cache
    groupMembersCache.value.delete(groupId);

    // Re-fetch from backend
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
      triggerRef(groupMembersCache);
      console.log('[useGroupMembersCache] Refreshed members for group:', groupId, 'count:', members.length);
    } catch (err) {
      console.error('[useGroupMembersCache] Failed to refresh group members:', err);
      triggerRef(groupMembersCache);
    }
  }

  /**
   * Setup listener for group member change events
   * Listens for message-updated events with MEMBER_JOIN or MEMBER_QUIT command types
   * and automatically refreshes the cache for affected groups
   */
  function useGroupMemberChangeListener(): void {
    let unlistenFn: UnlistenFn | null = null;

    onMounted(async () => {
      unlistenFn = await listen<MessageUpdateEvent>('message-updated', (event) => {
        const message = event.payload.message;
        if (!message) return;

        // Check if this is a group command message (member join/leave)
        const messageType = Number(message.messageType);
        const commandType = Number(message.commandType);
        const groupId = message.groupId;

        if (messageType === MessageType.GROUP_COMMAND && groupId) {
          if (commandType === CommandType.MEMBER_JOIN || commandType === CommandType.MEMBER_QUIT) {
            console.log('[useGroupMembersCache] Group member change detected:', {
              groupId,
              commandType: commandType === CommandType.MEMBER_JOIN ? 'JOIN' : 'QUIT',
            });

            // If this group is cached, refresh it
            if (groupMembersCache.value.has(groupId)) {
              console.log('[useGroupMembersCache] Refreshing cached group:', groupId);
              refreshGroupMembers(groupId);
            }
          }
        }
      });

      console.log('[useGroupMembersCache] Group member change listener registered');
    });

    onUnmounted(() => {
      if (unlistenFn) {
        unlistenFn();
        console.log('[useGroupMembersCache] Group member change listener unregistered');
      }
    });
  }

  return {
    fetchGroupMembers,
    getSenderInfo,
    isGroupCached,
    getGroupMemberCount,
    clearGroupCache,
    clearAllCache,
    refreshGroupMembers,
    useGroupMemberChangeListener,
  };
}

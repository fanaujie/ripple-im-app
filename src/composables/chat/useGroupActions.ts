import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { CreateGroupParams } from '../../types/chat';
import type { GroupMemberData } from '../../types/group';

/**
 * Composable for group-related actions
 *
 * Handles creating groups and managing group members via Tauri commands
 */
export function useGroupActions() {
  const loading = ref(false);
  const error = ref<string | null>(null);

  /**
   * Create a new group
   *
   * @param params Group creation parameters
   * @returns Promise resolving to the group ID
   * @throws Error if creation fails or API returns error
   */
  async function createGroup(params: CreateGroupParams): Promise<string> {
    try {
      const groupId = await invoke<string>('create_group', {
        senderId: params.senderId,
        groupName: params.groupName,
        memberIds: params.memberIds,
      });

      console.log('[useGroupActions] Group created successfully:', groupId);
      return groupId;
    } catch (error) {
      console.error('[useGroupActions] Failed to create group:', error);
      throw error;
    }
  }

  /**
   * Invite members to a group
   */
  async function inviteMembers(
    groupId: string,
    senderId: string,
    memberIds: string[],
    groupName: string,
    groupAvatar: string | null
  ): Promise<void> {
    try {
      loading.value = true;
      error.value = null;

      await invoke<void>('invite_members', {
        groupId,
        senderId,
        memberIds,
        groupName,
        groupAvatar,
      });

      console.log('[useGroupActions] Members invited successfully');
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      error.value = `Failed to invite members: ${message}`;
      console.error('[useGroupActions] Failed to invite members:', err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Get all members of a group
   */
  async function getGroupMembers(groupId: string): Promise<GroupMemberData[]> {
    try {
      loading.value = true;
      error.value = null;

      const response = await invoke<GroupMemberData[]>('get_group_members', {
        groupId,
      });

      console.log('[useGroupActions] Group members fetched:', response.length);
      return response;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      error.value = `Failed to fetch group members: ${message}`;
      console.error('[useGroupActions] Failed to fetch group members:', err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Update group name
   */
  async function updateGroupName(
    groupId: string,
    senderId: string,
    groupName: string
  ): Promise<void> {
    try {
      loading.value = true;
      error.value = null;

      await invoke<void>('update_group_name', {
        groupId,
        senderId,
        groupName,
      });

      console.log('[useGroupActions] Group name updated successfully');
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      error.value = `Failed to update group name: ${message}`;
      console.error('[useGroupActions] Failed to update group name:', err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Update group avatar
   */
  async function updateGroupAvatar(
    groupId: string,
    senderId: string,
    groupAvatar: string
  ): Promise<void> {
    try {
      loading.value = true;
      error.value = null;

      await invoke<void>('update_group_avatar', {
        groupId,
        senderId,
        groupAvatar,
      });

      console.log('[useGroupActions] Group avatar updated successfully');
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      error.value = `Failed to update group avatar: ${message}`;
      console.error('[useGroupActions] Failed to update group avatar:', err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Leave a group
   */
  async function leaveGroup(groupId: string): Promise<void> {
    try {
      loading.value = true;
      error.value = null;

      await invoke<void>('leave_group', {
        groupId,
      });

      console.log('[useGroupActions] Left group successfully');
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      error.value = `Failed to leave group: ${message}`;
      console.error('[useGroupActions] Failed to leave group:', err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  return {
    loading,
    error,
    createGroup,
    inviteMembers,
    getGroupMembers,
    updateGroupName,
    updateGroupAvatar,
    leaveGroup,
  };
}

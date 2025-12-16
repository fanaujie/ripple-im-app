<template>
  <div class="h-full flex flex-col">
    <!-- Header -->
    <div class="bg-white border-b border-gray-200 px-8 py-6">
      <h1 class="text-3xl font-semibold text-gray-900">People</h1>
    </div>

    <!-- Tabs -->
    <div class="bg-white border-b border-gray-200 px-8">
      <div class="flex">
        <button
          @click="activeTab = 'friends'"
          :class="[
            'py-4 px-2 font-medium transition-colors border-b-2',
            activeTab === 'friends'
              ? 'text-blue-500 border-blue-500'
              : 'text-gray-500 hover:text-gray-700 border-transparent'
          ]"
        >
          Friends
        </button>
        <button
          @click="activeTab = 'groups'"
          :class="[
            'py-4 px-2 font-medium ml-8 transition-colors border-b-2',
            activeTab === 'groups'
              ? 'text-blue-500 border-blue-500'
              : 'text-gray-500 hover:text-gray-700 border-transparent'
          ]"
        >
          Groups
        </button>
        <button
          @click="activeTab = 'blocked'"
          :class="[
            'py-4 px-2 font-medium ml-8 transition-colors border-b-2',
            activeTab === 'blocked'
              ? 'text-blue-500 border-blue-500'
              : 'text-gray-500 hover:text-gray-700 border-transparent'
          ]"
        >
          Blocked
        </button>
        <button
          @click="activeTab = 'add'"
          :class="[
            'py-4 px-2 font-medium ml-8 transition-colors border-b-2',
            activeTab === 'add'
              ? 'text-blue-500 border-blue-500'
              : 'text-gray-500 hover:text-gray-700 border-transparent'
          ]"
        >
          Add Friend
        </button>
      </div>
    </div>

    <!-- Content Area -->
    <div class="flex-1 bg-gray-50 px-8 py-6 overflow-auto">
      <!-- Friends Tab -->
      <div v-if="activeTab === 'friends'">
        <!-- Search Box -->
        <div class="relative mb-6">
          <HeroIcon name="magnifying-glass" className="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search friends by name or ID..."
            class="w-full pl-12 pr-4 py-3 bg-white border border-gray-200 rounded-lg text-gray-900 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
          />
        </div>

        <!-- Friends List -->
        <div class="space-y-2">
          <div v-if="loading" class="text-center py-16 text-gray-500">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto"></div>
          </div>

          <div v-else-if="filteredFriends.length === 0" class="text-center py-16 text-gray-500">
            {{ searchQuery ? 'No friends found' : 'No friends yet' }}
          </div>

          <div v-else v-for="friend in filteredFriends" :key="friend.userId">
            <!-- Friend Card -->
            <div class="bg-white rounded-lg p-4 border border-gray-100 hover:shadow-md transition-shadow">
              <div class="flex items-center justify-between">
                <!-- Left: Avatar and Info -->
                <div class="flex items-center gap-3">
                  <div class="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center overflow-hidden">
                    <img
                      :src="getAvatarUrl(friend.avatar)"
                      :alt="friend.remarkName || friend.nickName"
                      class="w-full h-full object-cover"
                      @error="onImageError"
                    />
                  </div>
                  <div>
                    <div class="font-medium text-gray-900">{{ friend.remarkName || friend.nickName }}</div>
                    <div class="text-sm text-gray-500">User ID: {{ friend.userId }}</div>
                  </div>
                </div>

                <!-- Right: Menu Button -->
                <div class="relative">
                  <button
                    @click.stop="toggleMenu(friend.userId)"
                    class="p-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
                  >
                    <HeroIcon name="more-vertical" className="w-5 h-5" />
                  </button>

                  <!-- Dropdown Menu -->
                  <div
                    v-if="showMenu === friend.userId"
                    v-click-outside="() => showMenu = null"
                    @click.stop
                    class="absolute right-0 mt-2 w-48 bg-white border border-gray-200 rounded-lg shadow-lg py-2 z-10"
                  >
                    <button
                      @click.stop="handleChat(friend)"
                      class="w-full px-4 py-2 text-left text-gray-700 hover:bg-gray-50 flex items-center gap-3 transition-colors"
                    >
                      <HeroIcon name="message-circle" className="w-4 h-4" />
                      <span>Chat</span>
                    </button>
                    <button
                      @click.stop="startEditingName(friend)"
                      class="w-full px-4 py-2 text-left text-gray-700 hover:bg-gray-50 flex items-center gap-3 transition-colors"
                    >
                      <HeroIcon name="edit-2" className="w-4 h-4" />
                      <span>Edit Remark Name</span>
                    </button>
                    <button
                      @click.stop="handleRemoveFriend(friend)"
                      class="w-full px-4 py-2 text-left text-red-600 hover:bg-gray-50 flex items-center gap-3 transition-colors"
                    >
                      <HeroIcon name="trash" className="w-4 h-4" />
                      <span>Remove Friend</span>
                    </button>
                    <button
                      @click.stop="handleBlock(friend)"
                      class="w-full px-4 py-2 text-left text-red-600 hover:bg-gray-50 flex items-center gap-3 transition-colors"
                    >
                      <HeroIcon name="no-symbol" className="w-4 h-4" />
                      <span>Block</span>
                    </button>
                  </div>
                </div>
              </div>
            </div>

            <!-- Edit Remark Name Interface -->
            <div v-if="editingFriend === friend.userId" class="bg-blue-50 border border-blue-200 rounded-lg p-4 mt-2">
              <!-- Display Current Nickname -->
              <div class="mb-3">
                <p class="text-sm text-gray-600 mb-1">Current Nickname:</p>
                <button
                  @click="copyNickNameToInput(friend)"
                  class="text-base font-medium text-blue-600 hover:text-blue-700 hover:underline cursor-pointer transition-colors text-left"
                  title="Click to copy to input"
                >
                  {{ friend.nickName }}
                </button>
                <p class="text-xs text-gray-500 mt-1">Click nickname to copy it to the input below</p>
              </div>

              <!-- Input for Remark Name -->
              <input
                ref="editNameInput"
                v-model="newFriendName"
                @keydown.enter="saveNewName(friend)"
                @keydown.esc="cancelEditing"
                type="text"
                placeholder="Enter remark name"
                class="w-full px-3 py-2 bg-white border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 mb-3"
              />
              <div class="flex gap-3">
                <button
                  @click="saveNewName(friend)"
                  class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
                >
                  Save
                </button>
                <button
                  @click="cancelEditing"
                  class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition-colors"
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Groups Tab -->
      <div v-if="activeTab === 'groups'">
        <!-- Header with Create Group button -->
        <div class="flex items-center justify-between mb-6">
          <div class="relative flex-1 mr-4">
            <HeroIcon name="magnifying-glass" className="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
            <input
              v-model="groupSearchQuery"
              type="text"
              placeholder="Search groups by name..."
              class="w-full pl-12 pr-4 py-3 bg-white border border-gray-200 rounded-lg text-gray-900 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
            />
          </div>
          <button
            @click="openCreateGroupDialog"
            class="px-4 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors flex items-center gap-2 whitespace-nowrap"
          >
            <HeroIcon name="plus" className="w-5 h-5" />
            <span>Create Group</span>
          </button>
        </div>

        <!-- Groups List -->
        <div class="space-y-2">
          <div v-if="loading" class="text-center py-16 text-gray-500">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto"></div>
          </div>

          <div v-else-if="filteredGroups.length === 0" class="text-center py-16 text-gray-500">
            {{ groupSearchQuery ? 'No groups found' : 'No groups yet. Create one to get started!' }}
          </div>

          <div v-else v-for="group in filteredGroups" :key="group.groupId">
            <!-- Group Card -->
            <div class="bg-white rounded-lg p-4 border border-gray-100 hover:shadow-md transition-shadow">
              <div class="flex items-center justify-between">
                <!-- Left: Avatar and Info -->
                <div class="flex items-center gap-3">
                  <div class="w-12 h-12 rounded-full bg-blue-100 flex items-center justify-center overflow-hidden">
                    <img
                      v-if="group.avatar"
                      :src="getAvatarUrl(group.avatar)"
                      :alt="group.name"
                      class="w-full h-full object-cover"
                      @error="onImageError"
                    />
                    <HeroIcon v-else name="user-group" className="w-6 h-6 text-blue-500" />
                  </div>
                  <div>
                    <div class="font-medium text-gray-900">{{ group.name }}</div>
                    <div class="text-sm text-gray-500">Group</div>
                  </div>
                </div>

                <!-- Right: Menu Button -->
                <div class="relative">
                  <button
                    @click.stop="toggleMenu(group.groupId)"
                    class="p-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
                  >
                    <HeroIcon name="more-vertical" className="w-5 h-5" />
                  </button>

                  <!-- Dropdown Menu -->
                  <div
                    v-if="showMenu === group.groupId"
                    v-click-outside="() => showMenu = null"
                    @click.stop
                    class="absolute right-0 mt-2 w-48 bg-white border border-gray-200 rounded-lg shadow-lg py-2 z-10"
                  >
                    <button
                      @click.stop="handleGroupChat(group)"
                      class="w-full px-4 py-2 text-left text-gray-700 hover:bg-gray-50 flex items-center gap-3 transition-colors"
                    >
                      <HeroIcon name="message-circle" className="w-4 h-4" />
                      <span>Chat</span>
                    </button>
                    <button
                      @click.stop="openInviteMembersDialog(group)"
                      class="w-full px-4 py-2 text-left text-gray-700 hover:bg-gray-50 flex items-center gap-3 transition-colors"
                    >
                      <HeroIcon name="user-plus" className="w-4 h-4" />
                      <span>Invite Members</span>
                    </button>
                    <button
                      @click.stop="openViewMembersDialog(group)"
                      class="w-full px-4 py-2 text-left text-gray-700 hover:bg-gray-50 flex items-center gap-3 transition-colors"
                    >
                      <HeroIcon name="users" className="w-4 h-4" />
                      <span>View Members</span>
                    </button>
                    <button
                      @click.stop="openEditGroupDialog(group)"
                      class="w-full px-4 py-2 text-left text-gray-700 hover:bg-gray-50 flex items-center gap-3 transition-colors"
                    >
                      <HeroIcon name="edit-2" className="w-4 h-4" />
                      <span>Edit Group Info</span>
                    </button>
                    <button
                      @click.stop="openLeaveGroupDialog(group)"
                      class="w-full px-4 py-2 text-left text-red-600 hover:bg-gray-50 flex items-center gap-3 transition-colors"
                    >
                      <HeroIcon name="log-out" className="w-4 h-4" />
                      <span>Leave Group</span>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Blocked Tab -->
      <div v-if="activeTab === 'blocked'">
        <!-- Search Box -->
        <div class="relative mb-6">
          <HeroIcon name="magnifying-glass" className="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search blocked users by name or ID..."
            class="w-full pl-12 pr-4 py-3 bg-white border border-gray-200 rounded-lg text-gray-900 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
          />
        </div>

        <!-- Blocked Users List -->
        <div class="space-y-2">
          <div v-if="loading" class="text-center py-16 text-gray-500">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto"></div>
          </div>

          <div v-else-if="filteredBlockedUsers.length === 0" class="text-center py-16 text-gray-500">
            No blocked users
          </div>

          <div v-else v-for="blockedUser in filteredBlockedUsers" :key="blockedUser.userId">
            <!-- Blocked User Card -->
            <div class="bg-white rounded-lg p-4 border border-gray-100 hover:shadow-md transition-shadow">
              <div class="flex items-center justify-between">
                <!-- Left: Avatar and Info -->
                <div class="flex items-center gap-3">
                  <div class="w-12 h-12 rounded-full bg-gray-100 flex items-center justify-center overflow-hidden opacity-60">
                    <img
                      :src="getAvatarUrl(blockedUser.avatar)"
                      :alt="blockedUser.remarkName || blockedUser.nickName"
                      class="w-full h-full object-cover"
                      @error="onImageError"
                    />
                  </div>
                  <div>
                    <div class="font-medium text-gray-900">{{ blockedUser.remarkName || blockedUser.nickName }}</div>
                    <div class="text-sm text-gray-500">User ID: {{ blockedUser.userId }}</div>
                  </div>
                </div>

                <!-- Right: Menu Button -->
                <div class="relative">
                  <button
                    @click.stop="toggleMenu(blockedUser.userId)"
                    class="p-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
                  >
                    <HeroIcon name="more-vertical" className="w-5 h-5" />
                  </button>

                  <!-- Dropdown Menu -->
                  <div
                    v-if="showMenu === blockedUser.userId"
                    v-click-outside="() => showMenu = null"
                    @click.stop
                    class="absolute right-0 mt-2 w-48 bg-white border border-gray-200 rounded-lg shadow-lg py-2 z-10"
                  >
                    <button
                      @click.stop="handleUnblock(blockedUser)"
                      class="w-full px-4 py-2 text-left text-gray-700 hover:bg-gray-50 flex items-center gap-3 transition-colors"
                    >
                      <HeroIcon name="eye" className="w-4 h-4" />
                      <span>Unblock</span>
                    </button>
                    <button
                      @click.stop="handleHide(blockedUser)"
                      class="w-full px-4 py-2 text-left text-gray-700 hover:bg-gray-50 flex items-center gap-3 transition-colors"
                    >
                      <HeroIcon name="eye-off" className="w-4 h-4" />
                      <span>Hide</span>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Add Friend Tab -->
      <div v-if="activeTab === 'add'" class="max-w-2xl mx-auto">
        <div class="bg-white rounded-lg p-6 border border-gray-200">
          <!-- Title -->
          <h2 class="text-xl font-semibold text-gray-900 mb-4">Add New Friend</h2>
          <p class="text-gray-600 mb-6">Enter a user ID to search and add friends</p>

          <!-- Search Input -->
          <div class="flex gap-3 mb-6">
            <input
              v-model="addFriendId"
              @keydown.enter="searchUser"
              type="text"
              placeholder="Enter user ID..."
              class="flex-1 px-4 py-3 bg-gray-50 border border-gray-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <button
              @click="searchUser"
              :disabled="searching"
              class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <HeroIcon name="magnifying-glass" className="w-4 h-4" />
              <span>Search</span>
            </button>
          </div>

          <!-- Search Result -->
          <div v-if="searchedUser" class="border-t border-gray-200 pt-6">
            <div class="bg-gray-50 rounded-lg p-4 flex items-center justify-between">
              <!-- Left: User Info -->
              <div class="flex items-center gap-4">
                <div class="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center overflow-hidden">
                  <img
                    :src="getAvatarUrl(searchedUser.avatar)"
                    :alt="searchedUser.remarkName || searchedUser.nickName"
                    class="w-full h-full object-cover"
                    @error="onImageError"
                  />
                </div>
                <div>
                  <div class="text-lg font-semibold text-gray-900">{{ searchedUser.remarkName || searchedUser.nickName }}</div>
                  <div class="text-gray-500">User ID: {{ searchedUser.userId }}</div>
                </div>
              </div>

              <!-- Right: Add Button -->
              <button
                @click="addNewFriend"
                :disabled="adding"
                class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <HeroIcon name="user-plus" className="w-4 h-4" />
                <span>Add Friend</span>
              </button>
            </div>
          </div>

          <!-- Search Error -->
          <div v-if="searchError" class="border-t border-gray-200 pt-6">
            <div class="bg-red-50 border border-red-200 rounded-lg p-4 text-red-600">
              {{ searchError }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Confirmation Modal -->
    <div
      v-if="showConfirmModal"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click="closeConfirmModal"
    >
      <div
        class="bg-white rounded-xl p-6 m-4 min-w-80 max-w-md"
        @click.stop
      >
        <h3 class="text-lg font-semibold text-gray-900 mb-4">{{ confirmAction?.title }}</h3>
        <p class="text-gray-600 mb-6">{{ confirmAction?.message }}</p>
        <div class="flex justify-end space-x-3">
          <button
            @click="closeConfirmModal"
            :disabled="isProcessing"
            class="px-4 py-2 text-sm text-gray-600 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            @click="confirmAction?.callback"
            :disabled="isProcessing"
            class="px-4 py-2 text-sm rounded-lg transition-colors disabled:opacity-50"
            :class="confirmAction?.dangerous
              ? 'bg-red-600 text-white hover:bg-red-700'
              : 'bg-blue-500 text-white hover:bg-blue-600'"
          >
            {{ isProcessing ? 'Processing...' : confirmAction?.confirmText }}
          </button>
        </div>
      </div>
    </div>

    <!-- Group Dialogs -->
    <CreateGroupDialog
      :is-open="isCreateGroupDialogOpen"
      :friends="friends"
      :current-user-id="currentUserId"
      @close="closeCreateGroupDialog"
      @success="handleGroupCreated"
    />

    <InviteMembersDialog
      :is-open="isInviteMembersDialogOpen"
      :group-id="selectedGroup?.groupId || ''"
      :group-name="selectedGroup?.name || ''"
      :group-avatar="selectedGroup?.avatar || null"
      :friends="friends"
      :current-user-id="currentUserId"
      @close="closeInviteMembersDialog"
      @success="closeInviteMembersDialog"
    />

    <ViewMembersDialog
      :is-open="isViewMembersDialogOpen"
      :group-id="selectedGroup?.groupId || ''"
      @close="closeViewMembersDialog"
    />

    <EditGroupDialog
      :is-open="isEditGroupDialogOpen"
      :group-id="selectedGroup?.groupId || ''"
      :group-name="selectedGroup?.name || ''"
      :group-avatar="selectedGroup?.avatar"
      :current-user-id="currentUserId"
      @close="closeEditGroupDialog"
      @success="closeEditGroupDialog"
    />

    <LeaveGroupDialog
      :is-open="isLeaveGroupDialogOpen"
      :group-id="selectedGroup?.groupId || ''"
      :group-name="selectedGroup?.name || ''"
      @close="closeLeaveGroupDialog"
      @success="handleLeaveGroupSuccess"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, computed } from 'vue';
import { useRouter } from 'vue-router';
import type { RelationUser } from '../types/relations';
import HeroIcon from '../components/shared/HeroIcon.vue';
import { useRelationsDisplay } from '../composables/useRelationsDisplay';
import { useRelationActions } from '../composables/useRelationActions';
import { useChatDisplay } from '../composables/chat/useChatDisplay';
import { useGroupsDisplay, type GroupDisplayItem } from '../composables/useGroupsDisplay';
import { useUserProfileDisplay } from '../composables/useUserProfileDisplay';
import CreateGroupDialog from '../components/chat/CreateGroupDialog.vue';
import InviteMembersDialog from '../components/group/InviteMembersDialog.vue';
import ViewMembersDialog from '../components/group/ViewMembersDialog.vue';
import EditGroupDialog from '../components/group/EditGroupDialog.vue';
import LeaveGroupDialog from '../components/group/LeaveGroupDialog.vue';
import defaultAvatarUrl from '../assets/default-avatar.svg';

// Define component name for KeepAlive
defineOptions({
  name: 'PeopleView'
});

type TabType = 'friends' | 'groups' | 'blocked' | 'add';

// Router
const router = useRouter();

// Local UI state
const activeTab = ref<TabType>('friends');
const addFriendId = ref('');
const searchedUser = ref<RelationUser | null>(null);
const searchError = ref<string | null>(null);
const searching = ref(false);
const adding = ref(false);
const showMenu = ref<string | null>(null);
const editingFriend = ref<string | null>(null);
const newFriendName = ref('');
const editNameInput = ref<HTMLInputElement | null>(null);

// Confirmation modal state
const showConfirmModal = ref(false);
const isProcessing = ref(false);
const confirmAction = ref<{
  title: string;
  message: string;
  confirmText: string;
  dangerous: boolean;
  callback: () => void;
} | null>(null);

// Use composables
const {
  friends: filteredFriends,
  blockedUsers: filteredBlockedUsers,
  friends,
  relationsMap,
  loading,
  searchQuery,
} = useRelationsDisplay();

const {
  getUserProfileById,
  addFriend,
  updateFriendRemarkName,
  blockUser,
  removeFriend,
  unblockUser,
  hideBlockedUser,
} = useRelationActions();

// User profile
const { userProfile } = useUserProfileDisplay();
const currentUserId = computed(() => userProfile.value?.userId || '');

// Chat display for groups (need conversations to filter groups)
const { conversations } = useChatDisplay(relationsMap);

// Groups display
const {
  filteredGroups,
  searchQuery: groupSearchQuery,
} = useGroupsDisplay(conversations);

// Group dialog states
const isCreateGroupDialogOpen = ref(false);
const isInviteMembersDialogOpen = ref(false);
const isViewMembersDialogOpen = ref(false);
const isEditGroupDialogOpen = ref(false);
const isLeaveGroupDialogOpen = ref(false);
const selectedGroup = ref<GroupDisplayItem | null>(null);

// Group dialog handlers
function openCreateGroupDialog() {
  isCreateGroupDialogOpen.value = true;
}

function closeCreateGroupDialog() {
  isCreateGroupDialogOpen.value = false;
}

function handleGroupCreated(groupId: string) {
  console.log('[PeopleView] Group created:', groupId);
  setTimeout(() => {
    closeCreateGroupDialog();
  }, 1000);
}

function openInviteMembersDialog(group: GroupDisplayItem) {
  selectedGroup.value = group;
  showMenu.value = null;
  isInviteMembersDialogOpen.value = true;
}

function closeInviteMembersDialog() {
  isInviteMembersDialogOpen.value = false;
  selectedGroup.value = null;
}

function openViewMembersDialog(group: GroupDisplayItem) {
  selectedGroup.value = group;
  showMenu.value = null;
  isViewMembersDialogOpen.value = true;
}

function closeViewMembersDialog() {
  isViewMembersDialogOpen.value = false;
  selectedGroup.value = null;
}

function openEditGroupDialog(group: GroupDisplayItem) {
  selectedGroup.value = group;
  showMenu.value = null;
  isEditGroupDialogOpen.value = true;
}

function closeEditGroupDialog() {
  isEditGroupDialogOpen.value = false;
  selectedGroup.value = null;
}

function openLeaveGroupDialog(group: GroupDisplayItem) {
  selectedGroup.value = group;
  showMenu.value = null;
  isLeaveGroupDialogOpen.value = true;
}

function closeLeaveGroupDialog() {
  isLeaveGroupDialogOpen.value = false;
  selectedGroup.value = null;
}

function handleLeaveGroupSuccess() {
  console.log('[PeopleView] Left group successfully');
}

// Navigate to group chat
function handleGroupChat(group: GroupDisplayItem) {
  showMenu.value = null;
  router.push({
    path: '/chat',
    query: { conversationId: group.conversationId }
  });
}

// Avatar handling
const getAvatarUrl = (avatarPath?: string) => {
  if (!avatarPath) return defaultAvatarUrl;
  if (avatarPath.startsWith('http://') || avatarPath.startsWith('https://')) {
    return avatarPath;
  }
  return `asset://localhost/${avatarPath}`;
};

const onImageError = (event: Event) => {
  const img = event.target as HTMLImageElement;
  if (img.src !== defaultAvatarUrl) {
    img.src = defaultAvatarUrl;
  }
};

// Menu handling
const toggleMenu = (userId: string) => {
  showMenu.value = showMenu.value === userId ? null : userId;
};

// Friends actions
const handleChat = (friend: RelationUser) => {
  showMenu.value = null;
  // Navigate to chat view with userId query parameter
  router.push({
    path: '/chat',
    query: { userId: friend.userId }
  });
};

const startEditingName = (friend: RelationUser) => {
  editingFriend.value = friend.userId;
  newFriendName.value = friend.remarkName || friend.nickName || '';
  showMenu.value = null;

  nextTick(() => {
    if (editNameInput.value) {
      editNameInput.value.focus();
    }
  });
};

const copyNickNameToInput = (friend: RelationUser) => {
  newFriendName.value = friend.nickName;
  nextTick(() => {
    if (editNameInput.value) {
      editNameInput.value.focus();
    }
  });
};

const saveNewName = async (friend: RelationUser) => {
  if (!newFriendName.value.trim()) {
    cancelEditing();
    return;
  }

  try {
    await updateFriendRemarkName(friend.userId, newFriendName.value.trim());
    cancelEditing();
  } catch (err) {
    console.error('Failed to update friend name:', err);
  }
};

const cancelEditing = () => {
  editingFriend.value = null;
  newFriendName.value = '';
};

// Confirmation modal helpers
const showConfirmationModal = (
  title: string,
  message: string,
  confirmText: string,
  dangerous: boolean,
  callback: () => void
) => {
  confirmAction.value = {
    title,
    message,
    confirmText,
    dangerous,
    callback
  };
  showConfirmModal.value = true;
};

const closeConfirmModal = () => {
  showConfirmModal.value = false;
  confirmAction.value = null;
};

const handleRemoveFriend = (friend: RelationUser) => {
  showMenu.value = null;

  showConfirmationModal(
    'Remove Friend',
    `Are you sure you want to remove ${friend.remarkName || friend.nickName} from your friends list?`,
    'Remove',
    true,
    async () => {
      if (isProcessing.value) return;
      isProcessing.value = true;

      try {
        await removeFriend(friend.userId);
        closeConfirmModal();
      } catch (err) {
        console.error('Failed to remove friend:', err);
      } finally {
        isProcessing.value = false;
      }
    }
  );
};

const handleBlock = (friend: RelationUser) => {
  showMenu.value = null;

  showConfirmationModal(
    'Block User',
    `Are you sure you want to block ${friend.remarkName || friend.nickName}? They will be moved to your blocked list and won't be able to message you.`,
    'Block',
    true,
    async () => {
      if (isProcessing.value) return;
      isProcessing.value = true;

      try {
        await blockUser(friend.userId, friend.remarkName || friend.nickName);
        closeConfirmModal();
      } catch (err) {
        console.error('Failed to block user:', err);
      } finally {
        isProcessing.value = false;
      }
    }
  );
};

// Blocked actions
const handleUnblock = (user: RelationUser) => {
  showMenu.value = null;

  showConfirmationModal(
    'Unblock User',
    `Are you sure you want to unblock ${user.remarkName || user.nickName}?

Note: If this user was originally your friend, they will be restored to your friends list. Otherwise, they will simply be removed from your blocked list.`,
    'Unblock',
    false,
    async () => {
      if (isProcessing.value) return;
      isProcessing.value = true;

      try {
        await unblockUser(user.userId);
        closeConfirmModal();
      } catch (err) {
        console.error('Failed to unblock user:', err);
      } finally {
        isProcessing.value = false;
      }
    }
  );
};

const handleHide = (user: RelationUser) => {
  showMenu.value = null;

  showConfirmationModal(
    'Hide User',
    `Are you sure you want to hide ${user.remarkName || user.nickName}? They will be permanently removed from your blocked list.`,
    'Hide',
    true,
    async () => {
      if (isProcessing.value) return;
      isProcessing.value = true;

      try {
        await hideBlockedUser(user.userId);
        closeConfirmModal();
      } catch (err) {
        console.error('Failed to hide user:', err);
      } finally {
        isProcessing.value = false;
      }
    }
  );
};

// Add friend actions
const searchUser = async () => {
  if (!addFriendId.value.trim()) {
    return;
  }

  searching.value = true;
  searchError.value = null;
  searchedUser.value = null;

  try {
    const result = await getUserProfileById(addFriendId.value.trim());
    console.log('Search result:', result);
    searchedUser.value = result;
  } catch (err) {
    console.error('Failed to search user:', err);
    searchError.value = 'User not found. Please check the user ID and try again.';
  } finally {
    searching.value = false;
  }
};

const addNewFriend = async () => {
  if (!searchedUser.value) {
    return;
  }

  adding.value = true;

  try {
    await addFriend(searchedUser.value.userId);
    // Clear search
    addFriendId.value = '';
    searchedUser.value = null;
    searchError.value = null;
    // Switch to friends tab
    activeTab.value = 'friends';
  } catch (err) {
    console.error('Failed to add friend:', err);
    searchError.value = 'Failed to add friend. Please try again.';
  } finally {
    adding.value = false;
  }
};

// Click outside directive
interface HTMLElementWithClickOutside extends HTMLElement {
  clickOutsideEvent?: (event: Event) => void;
}

const vClickOutside = {
  mounted(el: HTMLElementWithClickOutside, binding: any) {
    el.clickOutsideEvent = (event: Event) => {
      if (!(el === event.target || el.contains(event.target as Node))) {
        binding.value();
      }
    };
    document.addEventListener('click', el.clickOutsideEvent);
  },
  unmounted(el: HTMLElementWithClickOutside) {
    if (el.clickOutsideEvent) {
      document.removeEventListener('click', el.clickOutsideEvent);
    }
  }
};
</script>

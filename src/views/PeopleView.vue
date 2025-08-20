<template>
  <div class="h-full flex flex-col">
    <!-- Header -->
    <div class="bg-surface border-b border-border p-4 shadow-sm">
      <div class="flex items-center justify-between">
        <h1 class="text-xl font-semibold text-text">People</h1>
        
        <!-- Action Buttons -->
        <div class="flex space-x-2">
          <button
            @click="activeTab = 'add'"
            class="p-2 hover:bg-background rounded-lg transition-colors"
            title="Add Friend"
          >
            <HeroIcon name="plus" className="w-4 h-4 text-text-secondary" />
          </button>
          <button
            @click="activeTab = 'received'"
            class="relative p-2 hover:bg-background rounded-lg transition-colors"
            title="Friend Requests"
          >
            <HeroIcon name="bell" className="w-4 h-4 text-text-secondary" />
            <span 
              v-if="unreadRequestsCount > 0"
              class="absolute -top-1 -right-1 w-5 h-5 bg-danger text-white text-xs rounded-full flex items-center justify-center font-medium"
            >
              {{ unreadRequestsCount > 9 ? '9+' : unreadRequestsCount }}
            </span>
          </button>
        </div>
      </div>
    </div>

    <!-- Tab Navigation -->
    <div class="bg-surface border-b border-border">
      <div class="flex">
        <button
          @click="activeTab = 'friends'"
          :class="[
            'flex-1 py-3 px-4 text-sm font-medium border-b-2 transition-colors',
            activeTab === 'friends'
              ? 'border-primary text-primary bg-primary/5'
              : 'border-transparent text-text-secondary hover:text-text'
          ]"
        >
          Friends
        </button>
        
        <button
          @click="activeTab = 'received'"
          :class="[
            'flex-1 py-3 px-4 text-sm font-medium border-b-2 transition-colors flex items-center justify-center',
            activeTab === 'received'
              ? 'border-primary text-primary bg-primary/5'
              : 'border-transparent text-text-secondary hover:text-text'
          ]"
        >
          Received
        </button>
        
        <button
          @click="activeTab = 'sent'"
          :class="[
            'flex-1 py-3 px-4 text-sm font-medium border-b-2 transition-colors',
            activeTab === 'sent'
              ? 'border-primary text-primary bg-primary/5'
              : 'border-transparent text-text-secondary hover:text-text'
          ]"
        >
          Sent
        </button>
        
        <button
          @click="activeTab = 'add'"
          :class="[
            'flex-1 py-3 px-4 text-sm font-medium border-b-2 transition-colors',
            activeTab === 'add'
              ? 'border-primary text-primary bg-primary/5'
              : 'border-transparent text-text-secondary hover:text-text'
          ]"
        >
          Add Friend
        </button>
      </div>
    </div>

    <!-- Tab Content -->
    <div class="flex-1 bg-background">
      <!-- Friends List Tab -->
      <FriendsList
        v-if="activeTab === 'friends'"
        ref="friendsListRef"
        @friend-selected="handleFriendSelected"
      />
      
      <!-- Received Requests Tab -->
      <FriendRequests
        v-if="activeTab === 'received'"
        ref="friendRequestsRef"
        :active-tab="'received'"
      />
      
      <!-- Sent Requests Tab -->
      <FriendRequests
        v-if="activeTab === 'sent'"
        ref="friendRequestsRef"
        :active-tab="'sent'"
      />
      
      <!-- Add Friend Tab -->
      <AddFriend
        v-if="activeTab === 'add'"
        @request-sent="handleRequestSent"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { Friend } from '../types/friends';
import HeroIcon from '../components/shared/HeroIcon.vue';
import FriendsList from '../components/friends/FriendsList.vue';
import FriendRequests from '../components/friends/FriendRequests.vue';
import AddFriend from '../components/friends/AddFriend.vue';
// Define component name for KeepAlive
defineOptions({
  name: 'PeopleView'
});

type TabType = 'friends' | 'received' | 'sent' | 'add';

const activeTab = ref<TabType>('friends');
const friendsListRef = ref<InstanceType<typeof FriendsList>>();
const friendRequestsRef = ref<InstanceType<typeof FriendRequests>>();

// Mock data for demonstration - these would come from real API calls
const unreadRequestsCount = ref(2);

const handleFriendSelected = (friend: Friend) => {
  console.log('Friend selected:', friend);
  // Here you would typically navigate to chat or perform some action
  // For now, just switch back to friends list
  activeTab.value = 'friends';
};

const handleRequestSent = () => {
  // Refresh requests and switch to received tab
  if (friendRequestsRef.value) {
    friendRequestsRef.value.loadRequests();
  }
  activeTab.value = 'received';
};

onMounted(() => {
  // Load initial data
  if (friendsListRef.value) {
    friendsListRef.value.loadFriends();
  }
});
</script>
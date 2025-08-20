<template>
  <div class="h-full flex flex-col">
    <!-- Search Bar -->
    <div class="p-4 bg-surface border-b border-border">
      <div class="relative">
        <HeroIcon name="magnifying-glass" className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-text-secondary" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search friends..."
          class="w-full pl-10 pr-4 py-2 bg-background border border-border rounded-lg text-text placeholder-text-secondary focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent"
          @input="handleSearch"
        />
      </div>
    </div>

    <!-- Friends List -->
    <div class="flex-1 overflow-y-auto">
      <div v-if="loading" class="flex items-center justify-center p-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
      
      <div v-else-if="error" class="p-4 text-center text-danger">
        {{ error }}
      </div>
      
      <div v-else-if="displayedFriends.length === 0" class="p-8">
        <EmptyView
          title="No Friends"
          description="Add friends to start chatting."
          icon="users"
        />
      </div>
      
      <div v-else class="p-2">
        <div 
          v-for="friend in displayedFriends" 
          :key="friend.account"
          class="flex items-center p-3 hover:bg-surface rounded-lg transition-colors group cursor-pointer"
          @click="$emit('friendSelected', friend)"
        >
          <!-- Avatar -->
          <div class="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center mr-3 overflow-hidden">
            <img 
              v-if="friend.avatar" 
              :src="friend.avatar" 
              :alt="friend.nickName"
              class="w-full h-full object-cover"
            />
            <HeroIcon v-else name="user" className="w-6 h-6 text-primary" />
          </div>
          
          <!-- Friend Info -->
          <div class="flex-1 min-w-0">
            <h3 class="font-medium text-text truncate">{{ friend.nickName }}</h3>
            <p class="text-sm text-text-secondary truncate">{{ friend.account }}</p>
          </div>
          
          <!-- Actions Menu -->
          <div class="opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              @click.stop="openActionMenu(friend)"
              class="p-2 hover:bg-background rounded-lg transition-colors"
            >
              <HeroIcon name="ellipsis-horizontal" className="w-4 h-4 text-text-secondary" />
            </button>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Action Menu Modal -->
    <div 
      v-if="selectedFriend && showActionMenu" 
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click="closeActionMenu"
    >
      <div 
        class="bg-surface rounded-xl p-4 m-4 min-w-64 max-w-sm"
        @click.stop
      >
        <h3 class="font-semibold text-text mb-3">Friend Actions</h3>
        <div class="space-y-2">
          <button
            class="w-full text-left p-3 hover:bg-background rounded-lg transition-colors text-text"
            @click="startChat"
          >
            <HeroIcon name="chat-bubble-left" className="inline w-4 h-4 mr-2" />
            Start Chat
          </button>
          <button
            class="w-full text-left p-3 hover:bg-danger/10 rounded-lg transition-colors text-danger"
            @click="confirmRemoveFriend"
          >
            <HeroIcon name="trash" className="inline w-4 h-4 mr-2" />
            Remove Friend
          </button>
        </div>
        <button
          @click="closeActionMenu"
          class="w-full mt-4 p-3 bg-background hover:bg-border rounded-lg transition-colors text-text-secondary"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Friend } from '../../types/friends';
import HeroIcon from '../shared/HeroIcon.vue';
import EmptyView from '../views/EmptyView.vue';

const emit = defineEmits<{
  friendSelected: [friend: Friend];
}>();

const friends = ref<Friend[]>([]);
const loading = ref(false);
const error = ref('');
const searchQuery = ref('');
const selectedFriend = ref<Friend | null>(null);
const showActionMenu = ref(false);

const displayedFriends = computed(() => {
  if (!searchQuery.value) return friends.value;
  return friends.value.filter(friend => 
    friend.nickName.toLowerCase().includes(searchQuery.value.toLowerCase())
  );
});

const loadFriends = async () => {
  try {
    loading.value = true;
    error.value = '';
    const result = await invoke<Friend[]>('get_friends_list');
    friends.value = result;
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load friends list';
  } finally {
    loading.value = false;
  }
};

const handleSearch = async () => {
  if (!searchQuery.value.trim()) return;
  
  try {
    loading.value = true;
    const result = await invoke<Friend[]>('search_friends', { keyword: searchQuery.value });
    friends.value = result;
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Search failed';
  } finally {
    loading.value = false;
  }
};

const openActionMenu = (friend: Friend) => {
  selectedFriend.value = friend;
  showActionMenu.value = true;
};

const closeActionMenu = () => {
  showActionMenu.value = false;
  selectedFriend.value = null;
};

const startChat = () => {
  if (selectedFriend.value) {
    emit('friendSelected', selectedFriend.value);
    closeActionMenu();
  }
};

const confirmRemoveFriend = async () => {
  if (!selectedFriend.value) return;
  
  if (confirm(`Are you sure you want to remove "${selectedFriend.value.nickName}" from your friends?`)) {
    try {
      await invoke('remove_friend', { friendAccount: selectedFriend.value.account });
      await loadFriends();
      closeActionMenu();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to remove friend';
    }
  }
};

onMounted(() => {
  loadFriends();
});

defineExpose({
  loadFriends
});
</script>
<template>
  <div class="h-full flex flex-col">
    <!-- Search Header -->
    <div class="bg-surface border-b border-border p-4">
      <div class="relative">
        <HeroIcon name="magnifying-glass" className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-text-secondary" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜尋好友 nickName..."
          class="w-full pl-12 pr-4 py-3 bg-background border border-border rounded-lg text-text placeholder-text-secondary focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent"
          @input="handleSearch"
          @keyup.enter="() => performSearch()"
        />
        <button
          v-if="searchQuery"
          @click="clearSearch"
          class="absolute right-3 top-1/2 transform -translate-y-1/2 p-1 hover:bg-border rounded transition-colors"
        >
          <HeroIcon name="x-mark" className="w-4 h-4 text-text-secondary" />
        </button>
      </div>
    </div>

    <!-- Search Results -->
    <div class="flex-1 overflow-y-auto">
      <!-- Loading State -->
      <div v-if="loading" class="flex items-center justify-center p-8">
        <div class="text-center">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-3"></div>
          <p class="text-text-secondary">搜尋中...</p>
        </div>
      </div>
      
      <!-- Error State -->
      <div v-else-if="error" class="p-6">
        <div class="text-center text-danger bg-danger/10 border border-danger/20 rounded-lg p-4">
          <HeroIcon name="exclamation-triangle" className="w-8 h-8 mx-auto mb-2" />
          <p>{{ error }}</p>
        </div>
      </div>
      
      <!-- Empty Search State -->
      <div v-else-if="!hasSearched && !searchQuery" class="p-8">
        <EmptyView
          title="搜尋好友"
          description="輸入好友的 nickName 來搜尋您的好友。"
          icon="magnifying-glass"
        />
      </div>
      
      <!-- No Results -->
      <div v-else-if="hasSearched && searchResults.length === 0" class="p-8">
        <div class="text-center">
          <div class="w-16 h-16 bg-text-secondary/10 rounded-full flex items-center justify-center mx-auto mb-4">
            <HeroIcon name="face-frown" className="w-8 h-8 text-text-secondary" />
          </div>
          <h3 class="text-lg font-medium text-text mb-2">找不到好友</h3>
          <p class="text-text-secondary">
            沒有找到包含 "{{ searchQuery }}" 的好友，
            <br>
            請嘗試其他關鍵字。
          </p>
        </div>
      </div>
      
      <!-- Search Results -->
      <div v-else class="p-2">
        <div class="mb-4 px-2">
          <p class="text-sm text-text-secondary">
            找到 {{ searchResults.length }} 位好友
          </p>
        </div>
        
        <div 
          v-for="friend in searchResults" 
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
            <h3 class="font-medium text-text">
              <!-- Highlight search term -->
              <span v-html="highlightText(friend.nickName, searchQuery)"></span>
            </h3>
            <p class="text-sm text-text-secondary truncate">{{ friend.account }}</p>
          </div>
          
          <!-- Chat Icon -->
          <div class="opacity-0 group-hover:opacity-100 transition-opacity">
            <div class="p-2 hover:bg-background rounded-lg transition-colors">
              <HeroIcon name="chat-bubble-left" className="w-4 h-4 text-accent" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Search Suggestions -->
    <div v-if="!hasSearched && !loading" class="bg-surface border-t border-border p-4">
      <h4 class="text-sm font-medium text-text mb-2">搜尋建議</h4>
      <div class="flex flex-wrap gap-2">
        <button
          v-for="suggestion in searchSuggestions"
          :key="suggestion"
          @click="applySuggestion(suggestion)"
          class="px-3 py-1 text-xs bg-background border border-border rounded-full text-text-secondary hover:bg-border transition-colors"
        >
          {{ suggestion }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Friend } from '../../types/friends';
import HeroIcon from '../shared/HeroIcon.vue';
import EmptyView from '../views/EmptyView.vue';

defineEmits<{
  friendSelected: [friend: Friend];
}>();

const searchQuery = ref('');
const searchResults = ref<Friend[]>([]);
const loading = ref(false);
const error = ref('');
const hasSearched = ref(false);
const searchTimeout = ref<number>();

const searchSuggestions = ref([
  'Alice', 'Bob', 'Charlie', 'David', 'Eve'
]);

const performSearch = async (query?: string) => {
  const searchTerm = query || searchQuery.value.trim();
  
  if (!searchTerm) {
    searchResults.value = [];
    hasSearched.value = false;
    return;
  }

  try {
    loading.value = true;
    error.value = '';
    
    const results = await invoke<Friend[]>('search_friends', {
      keyword: searchTerm
    });
    
    searchResults.value = results;
    hasSearched.value = true;
  } catch (err) {
    error.value = err instanceof Error ? err.message : '搜尋失敗';
    searchResults.value = [];
  } finally {
    loading.value = false;
  }
};

const handleSearch = () => {
  // Debounce search
  if (searchTimeout.value) {
    clearTimeout(searchTimeout.value);
  }
  
  searchTimeout.value = setTimeout(() => {
    if (searchQuery.value.trim()) {
      performSearch();
    } else {
      searchResults.value = [];
      hasSearched.value = false;
    }
  }, 300);
};

const clearSearch = () => {
  searchQuery.value = '';
  searchResults.value = [];
  hasSearched.value = false;
  error.value = '';
};

const applySuggestion = (suggestion: string) => {
  searchQuery.value = suggestion;
  performSearch(suggestion);
};

const highlightText = (text: string, highlight: string): string => {
  if (!highlight.trim()) return text;
  
  const regex = new RegExp(`(${highlight})`, 'gi');
  return text.replace(regex, '<span class="bg-accent/20 text-accent font-medium">$1</span>');
};
</script>
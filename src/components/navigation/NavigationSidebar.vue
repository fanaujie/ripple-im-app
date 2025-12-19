<template>
  <div class="h-full w-sidebar bg-sidebar border-r border-border-sidebar flex flex-col">
    <!-- Logo/Header -->
    <div class="h-header flex items-center justify-between px-6 border-b border-border-sidebar">
      <div class="flex items-center">
        <h1 class="text-xl font-semibold text-text-sidebar">Ripple</h1>
      </div>
    </div>

    <!-- Navigation Items -->
    <nav class="flex-1 px-4 py-4 space-y-2">
      <button
        v-for="item in navigationItems"
        :key="item.id"
        @click="handleNavigationClick(item)"
        :disabled="!item.enabled"
        :class="[
          'w-full flex items-center gap-3 px-3 py-3 rounded-lg text-left transition-all duration-200',
          activeItem === item.id 
            ? 'bg-primary text-white shadow-lg' 
            : 'text-text-sidebar-secondary hover:text-text-sidebar hover:bg-sidebar-hover',
          !item.enabled && 'opacity-50 cursor-not-allowed'
        ]"
      >
        <HeroIcon :name="item.icon" className="w-5 h-5" />
        <span class="font-medium">{{ item.label }}</span>
      </button>
    </nav>

    <!-- User Profile & Logout -->
    <div v-if="settings.showLogoutButton" class="p-4 border-t border-border-sidebar">
      <!-- User Profile -->
      <div v-if="userProfile" class="flex items-center gap-3 px-3 py-3 mb-2">
        <img
          :src="avatarUrl"
          @error="onAvatarError"
          alt="Avatar"
          class="w-10 h-10 rounded-full flex-shrink-0 object-cover"
        />
        <span class="font-medium text-text-sidebar truncate">{{ userProfile.nickName }}</span>
      </div>
      <!-- Logout Button -->
      <button
        @click="handleLogout"
        class="w-full flex items-center gap-3 px-3 py-3 rounded-lg text-left transition-all duration-200 text-red-400 hover:text-red-300 hover:bg-red-900/20"
      >
        <HeroIcon name="arrow-left-on-rectangle" className="w-5 h-5" />
        <span class="font-medium">Logout</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import router from '../../router/router';
import HeroIcon from '../shared/HeroIcon.vue';
import {
  NavigationItem,
  NAVIGATION_ITEMS,
  NAVIGATION_SETTINGS
} from '../../types/navigation';
import { useUserProfileDisplay } from '../../composables/useUserProfileDisplay';
import defaultAvatarUrl from '../../assets/default-avatar.svg';

const navigationItems = ref<NavigationItem[]>(NAVIGATION_ITEMS);
const settings = ref(NAVIGATION_SETTINGS);

// User profile state (auto-updates when profile changes)
const { userProfile } = useUserProfileDisplay();

const avatarUrl = computed(() => {
  const avatar = userProfile.value?.avatar;
  if (!avatar) return defaultAvatarUrl;
  if (avatar.startsWith('http://') || avatar.startsWith('https://')) {
    return avatar;
  }
  return `asset://localhost/${avatar}`;
});

function onAvatarError(event: Event) {
  const img = event.target as HTMLImageElement;
  img.src = defaultAvatarUrl;
}

// Track active item based on current route
const activeItem = computed(() => {
  const currentRoute = router.currentRoute.value;
  return (currentRoute.name as string) || settings.value.defaultActive;
});

const handleNavigationClick = (item: NavigationItem) => {
  if (!item.enabled) return;
  
  router.replace(`/${item.id}`);
};

const handleLogout = async () => {
  try {
    await invoke('logout');
    router.push({ name: 'login' });
  } catch (error) {
    console.error('Failed to logout:', error);
    // Still navigate to login even if logout fails
    router.push({ name: 'login' });
  }
};
</script>
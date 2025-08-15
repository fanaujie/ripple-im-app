<template>
  <div class="h-full w-sidebar bg-sidebar border-r border-border-sidebar flex flex-col">
    <!-- Logo/Header -->
    <div class="h-header flex items-center justify-between px-6 border-b border-border-sidebar">
      <div class="flex items-center">
        <h1 class="text-xl font-semibold text-text-sidebar">ripple</h1>
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

    <!-- Logout Button -->
    <div v-if="settings.showLogoutButton" class="p-4 border-t border-border-sidebar">
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
import router from '../../router/router';
import HeroIcon from '../shared/HeroIcon.vue';
import { 
  NavigationItem, 
  NAVIGATION_ITEMS, 
  NAVIGATION_SETTINGS 
} from '../../types/navigation';

const navigationItems = ref<NavigationItem[]>(NAVIGATION_ITEMS);
const settings = ref(NAVIGATION_SETTINGS);

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
    // Direct logout logic without backend call
    console.log('Logout action triggered');
    router.push('/login');
  } catch (error) {
    console.error('Failed to logout:', error);
  }
};
</script>
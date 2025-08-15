<template>
  <div class="h-screen flex bg-background">
    <!-- Navigation Sidebar -->
    <NavigationSidebar />
    
    <!-- Main Content Area with KeepAlive -->
    <main class="flex-1 h-full overflow-hidden">
      <router-view v-slot="{ Component, route }">
        <keep-alive :include="['ChatView', 'PeopleView', 'SettingsView']">
          <component :is="Component" :key="route.name" />
        </keep-alive>
      </router-view>
    </main>
  </div>
</template>

<script setup lang="ts">
import NavigationSidebar from '../components/navigation/NavigationSidebar.vue';
import { useEventListeners } from '../composables/useEventListeners';

// Initialize event listeners
useEventListeners();
</script>
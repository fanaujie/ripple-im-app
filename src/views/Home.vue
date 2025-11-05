<template>
  <div class="h-screen flex bg-background">
    <!-- Loading overlay during global data initialization -->
    <div v-if="isInitializing" class="absolute inset-0 flex items-center justify-center bg-background/80 z-50">
      <div class="text-text">Initializing...</div>
    </div>

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
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import NavigationSidebar from '../components/navigation/NavigationSidebar.vue';

const isInitializing = ref(false);

onMounted(async () => {
  // Initialize global data on Home.vue mount
  isInitializing.value = true;
  try {
    await invoke('init_global_data');
    console.log('Global data initialized successfully');
  } catch (error) {
    console.error('Failed to initialize global data:', error);
  } finally {
    isInitializing.value = false;
  }
});
</script>
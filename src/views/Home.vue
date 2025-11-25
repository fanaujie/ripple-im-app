<template>
  <div class="h-screen flex bg-background">
    <!-- Loading screen during initialization -->
    <div v-if="isInitializing" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <div class="text-text text-lg mb-2">Initializing...</div>
        <div class="text-text-secondary text-sm">Loading your data</div>
      </div>
    </div>

    <!-- Error screen if initialization fails -->
    <div v-else-if="initError" class="flex-1 flex items-center justify-center">
      <div class="text-center max-w-md px-4">
        <div class="text-text text-lg mb-2">Failed to Initialize</div>
        <div class="text-text-secondary text-sm mb-4">{{ initError }}</div>
        <button
          @click="retryInitialization"
          class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
        >
          Retry
        </button>
      </div>
    </div>

    <!-- Main app content (only shown after successful initialization) -->
    <template v-else>
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
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useRoute, useRouter } from 'vue-router';
import NavigationSidebar from '../components/navigation/NavigationSidebar.vue';

const route = useRoute();
const router = useRouter();
const isInitializing = ref(false);
const initError = ref<string | null>(null);

async function initializeGlobalData() {
  isInitializing.value = true;
  initError.value = null;

  try {
    await invoke('preload_global_data');
    console.log('Global data initialized successfully');

    // After successful initialization, navigate to chat if still on /home
    if (route.path === '/home') {
      await router.push('/chat');
    }
  } catch (error) {
    console.error('Failed to initialize global data:', error);
    initError.value = error instanceof Error ? error.message : String(error);
  } finally {
    isInitializing.value = false;
  }
}

function retryInitialization() {
  initializeGlobalData();
}

onMounted(() => {
  // Initialize global data on Home.vue mount
  // This will cache user profile, relations, and conversations data
  // After initialization completes, we'll navigate to /chat
  // Child components will only mount after navigation, ensuring data is ready
  initializeGlobalData();
});
</script>
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
import { onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import NavigationSidebar from '../components/navigation/NavigationSidebar.vue';

const route = useRoute();
const router = useRouter();

onMounted(() => {
  // Navigate to chat if on /home route
  if (route.path === '/home') {
    router.push('/chat');
  }
});
</script>
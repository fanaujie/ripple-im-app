<template>
  <div class="h-full flex flex-col">
    <!-- Header -->
    <div class="bg-surface border-b border-border p-4 shadow-sm">
      <h1 class="text-xl font-semibold text-text">People</h1>
    </div>

    <!-- Empty State -->
    <div class="flex-1 flex items-center justify-center bg-background">
      <EmptyView
        title="No Friends"
        description="Add friends to start chatting."
        icon="users"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import EmptyView from '../components/views/EmptyView.vue';

// Define component name for KeepAlive
defineOptions({
  name: 'PeopleView'
});
</script>
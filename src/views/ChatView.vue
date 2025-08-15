<template>
  <div class="h-full flex flex-col">
    <!-- Header -->
    <div class="bg-surface border-b border-border p-4 shadow-sm">
      <h1 class="text-xl font-semibold text-text">Chat</h1>
    </div>

    <!-- Empty State -->
    <div class="flex-1 flex items-center justify-center bg-background">
      <EmptyView
        title="No Conversations"
        description="Start a conversation with your friends."
        icon="chat-bubble-left-right"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import EmptyView from '../components/views/EmptyView.vue';

// Define component name for KeepAlive
defineOptions({
  name: 'ChatView'
});
</script>
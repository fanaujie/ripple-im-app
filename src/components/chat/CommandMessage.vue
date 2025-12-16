<template>
  <div class="flex justify-center py-3">
    <div class="bg-gray-200 text-gray-600 text-sm rounded-full px-4 py-2 text-center max-w-md">
      {{ displayText }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Message } from '../../types/chat';

interface Props {
  message: Message;
}

const props = defineProps<Props>();

const displayText = computed(() => {
  // commandData is already a human-readable string from the backend
  // e.g., "pikachu、pichu、frank joined the group"
  if (props.message.commandData) {
    return props.message.commandData;
  }

  // Fallback for missing commandData
  return 'Group notification';
});
</script>

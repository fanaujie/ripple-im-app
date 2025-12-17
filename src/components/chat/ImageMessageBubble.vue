<template>
  <div
    class="image-message-bubble cursor-pointer relative"
    :class="{ 'is-self': isSelf }"
    @click="handleClick"
  >
    <!-- Loading placeholder (absolute positioned on top) -->
    <div v-if="loading && !hasError" class="image-placeholder absolute inset-0 z-10">
      <svg class="animate-spin h-6 w-6 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
      </svg>
    </div>
    <!-- Error state -->
    <div v-if="hasError" class="image-placeholder">
      <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6 text-gray-400">
        <path stroke-linecap="round" stroke-linejoin="round" d="m2.25 15.75 5.159-5.159a2.25 2.25 0 0 1 3.182 0l5.159 5.159m-1.5-1.5 1.409-1.409a2.25 2.25 0 0 1 3.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 0 0 1.5-1.5V6a1.5 1.5 0 0 0-1.5-1.5H3.75A1.5 1.5 0 0 0 2.25 6v12a1.5 1.5 0 0 0 1.5 1.5Zm10.5-11.25h.008v.008h-.008V8.25Zm.375 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Z" />
      </svg>
    </div>
    <!-- Image (always in DOM to trigger load, use opacity to hide while loading) -->
    <img
      v-if="!hasError"
      :src="fileUrl"
      :alt="fileName"
      class="message-image"
      :class="{ 'opacity-0': loading }"
      @load="onLoad"
      @error="onError"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const props = defineProps<{
  fileUrl: string;
  fileName: string;
  isSelf: boolean;
}>();

const emit = defineEmits<{
  (e: 'preview', url: string, name: string): void;
}>();

const loading = ref(true);
const hasError = ref(false);

function onLoad() {
  loading.value = false;
}

function onError() {
  loading.value = false;
  hasError.value = true;
}

function handleClick() {
  if (!hasError.value && !loading.value) {
    emit('preview', props.fileUrl, props.fileName);
  }
}
</script>

<style scoped>
.image-message-bubble {
  max-width: 300px;
  max-height: 200px;
  overflow: hidden;
  border-radius: 12px;
  background-color: #f3f4f6;
}

.image-message-bubble.is-self {
  border-bottom-right-radius: 4px;
}

.image-message-bubble:not(.is-self) {
  border-bottom-left-radius: 4px;
}

.image-placeholder {
  width: 200px;
  height: 150px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #f3f4f6;
}

.message-image {
  max-width: 300px;
  max-height: 200px;
  object-fit: contain;
  display: block;
}
</style>

<template>
  <div
    class="file-message-bubble cursor-pointer"
    :class="{ 'is-self': isSelf }"
    @click="handleClick"
  >
    <!-- File Icon -->
    <div class="file-icon">
      <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-8 h-8">
        <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
      </svg>
    </div>

    <!-- File Info -->
    <div class="file-info">
      <div class="file-name" :title="fileName">{{ displayFileName }}</div>
      <div class="file-type">{{ fileType }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { getFileType, extractFileName } from '../../utils/fileUtils';

const props = defineProps<{
  fileUrl: string;
  fileName: string;
  isSelf: boolean;
}>();

const emit = defineEmits<{
  (e: 'preview', url: string, name: string): void;
}>();

const displayFileName = computed(() => {
  const name = props.fileName || extractFileName(props.fileUrl);
  // Truncate long names
  if (name.length > 25) {
    const ext = name.split('.').pop() || '';
    const baseName = name.substring(0, name.length - ext.length - 1);
    return baseName.substring(0, 20) + '...' + (ext ? '.' + ext : '');
  }
  return name;
});

const fileType = computed(() => {
  return getFileType(props.fileName || props.fileUrl);
});

function handleClick() {
  emit('preview', props.fileUrl, props.fileName);
}
</script>

<style scoped>
.file-message-bubble {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 12px;
  background-color: #f3f4f6;
  min-width: 200px;
  max-width: 280px;
}

.file-message-bubble.is-self {
  background-color: #dbeafe;
  border-bottom-right-radius: 4px;
}

.file-message-bubble:not(.is-self) {
  border-bottom-left-radius: 4px;
}

.file-icon {
  flex-shrink: 0;
  color: #6b7280;
}

.file-message-bubble.is-self .file-icon {
  color: #3b82f6;
}

.file-info {
  flex: 1;
  min-width: 0;
  overflow: hidden;
}

.file-name {
  font-size: 14px;
  font-weight: 500;
  color: #1f2937;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-type {
  font-size: 12px;
  color: #6b7280;
  margin-top: 2px;
}
</style>

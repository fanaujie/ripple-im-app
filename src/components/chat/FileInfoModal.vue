<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="isOpen"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
        @click="handleBackdropClick"
        @keydown.escape="close"
      >
        <!-- Modal Content -->
        <div
          class="bg-white rounded-xl shadow-xl p-6 min-w-[320px] max-w-md"
          @click.stop
        >
          <!-- Close button -->
          <button
            class="absolute top-4 right-4 p-1 text-gray-400 hover:text-gray-600 transition-colors"
            @click="close"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-5 h-5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
            </svg>
          </button>

          <!-- File Icon -->
          <div class="flex justify-center mb-4">
            <div class="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-8 h-8 text-gray-500">
                <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
              </svg>
            </div>
          </div>

          <!-- File Name -->
          <h3 class="text-center text-lg font-medium text-gray-900 mb-2 break-all">
            {{ fileName }}
          </h3>

          <!-- File Type -->
          <p class="text-center text-sm text-gray-500 mb-6">
            {{ fileType }}
          </p>

          <!-- Download button -->
          <button
            class="w-full flex items-center justify-center gap-2 px-4 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 transition-colors"
            :disabled="downloading"
            @click="handleDownload"
          >
            <svg v-if="downloading" class="animate-spin h-5 w-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M16.5 12 12 16.5m0 0L7.5 12m4.5 4.5V3" />
            </svg>
            {{ downloading ? 'Downloading...' : 'Download' }}
          </button>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { getFileType, extractFileName } from '../../utils/fileUtils';

const props = defineProps<{
  isOpen: boolean;
  fileUrl: string;
  fileName: string;
  downloading?: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'download', url: string, name: string): void;
}>();

const fileType = computed(() => {
  return getFileType(props.fileName || extractFileName(props.fileUrl));
});

function close() {
  emit('close');
}

function handleBackdropClick() {
  close();
}

function handleDownload() {
  emit('download', props.fileUrl, props.fileName);
}
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="isOpen"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/80"
        @click="handleBackdropClick"
        @keydown.escape="close"
      >
        <!-- Modal Content -->
        <div class="relative max-w-[90vw] max-h-[90vh]" @click.stop>
          <!-- Close button -->
          <button
            class="absolute -top-10 right-0 p-2 text-white hover:text-gray-300 transition-colors"
            @click="close"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-6 h-6">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
            </svg>
          </button>

          <!-- Image -->
          <img
            :src="imageUrl"
            :alt="fileName"
            class="max-w-[90vw] max-h-[85vh] object-contain rounded-lg"
          />

          <!-- Download button -->
          <button
            class="absolute -bottom-12 left-1/2 -translate-x-1/2 flex items-center gap-2 px-4 py-2 bg-white text-gray-900 rounded-lg hover:bg-gray-100 disabled:opacity-50 transition-colors"
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
const props = defineProps<{
  isOpen: boolean;
  imageUrl: string;
  fileName: string;
  downloading?: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'download', url: string, name: string): void;
}>();

function close() {
  emit('close');
}

function handleBackdropClick() {
  close();
}

function handleDownload() {
  emit('download', props.imageUrl, props.fileName);
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

<template>
  <div v-if="isOpen" class="fixed inset-0 z-[60] flex items-center justify-center">
    <div class="absolute inset-0 bg-black/70" @click="$emit('close')"></div>
    <div class="relative bg-white rounded-2xl shadow-xl max-w-md w-full mx-4 overflow-hidden">
      <div class="px-6 py-4 border-b border-gray-200">
        <h3 class="text-lg font-semibold">{{ title }}</h3>
      </div>
      <div class="p-6">
        <div
          ref="containerRef"
          class="w-full rounded-xl overflow-hidden bg-gray-900 border border-gray-200 mb-6 relative select-none"
          :style="{ minHeight: '300px', maxHeight: '400px' }"
        >
          <div v-if="previewUrl" class="relative w-full h-full flex items-center justify-center">
            <img
              ref="imageRef"
              :src="previewUrl"
              alt="Avatar Preview"
              class="max-w-full h-auto pointer-events-none"
              @load="onImageLoad"
            >

            <!-- Crop preview overlay (LINE style) -->
            <div v-if="cropBox.size > 0" class="absolute inset-0 pointer-events-none">
              <!-- Dark overlay with circular cutout -->
              <svg class="absolute inset-0 w-full h-full">
                <defs>
                  <mask :id="maskId">
                    <rect width="100%" height="100%" fill="white"/>
                    <circle
                        :cx="cropBox.x + cropBox.size / 2"
                        :cy="cropBox.y + cropBox.size / 2"
                        :r="cropBox.size / 2"
                        fill="black"/>
                  </mask>
                </defs>
                <rect width="100%" height="100%" fill="rgba(0,0,0,0.6)" :mask="`url(#${maskId})`"/>
              </svg>

              <!-- Crop box with corner markers and grid -->
              <div
                :style="{
                  position: 'absolute',
                  left: cropBox.x + 'px',
                  top: cropBox.y + 'px',
                  width: cropBox.size + 'px',
                  height: cropBox.size + 'px',
                  cursor: 'ns-resize'
                }"
                class="pointer-events-auto"
                @mousedown="startDrag"
                @touchstart="startDrag"
              >
                <!-- Corner markers -->
                <div class="absolute top-0 left-0 w-6 h-6 border-t-2 border-l-2 border-white"></div>
                <div class="absolute top-0 right-0 w-6 h-6 border-t-2 border-r-2 border-white"></div>
                <div class="absolute bottom-0 left-0 w-6 h-6 border-b-2 border-l-2 border-white"></div>
                <div class="absolute bottom-0 right-0 w-6 h-6 border-b-2 border-r-2 border-white"></div>

                <!-- Grid lines (3x3) -->
                <div class="absolute inset-0">
                  <div class="absolute left-1/3 top-0 bottom-0 w-px bg-white/30"></div>
                  <div class="absolute left-2/3 top-0 bottom-0 w-px bg-white/30"></div>
                  <div class="absolute top-1/3 left-0 right-0 h-px bg-white/30"></div>
                  <div class="absolute top-2/3 left-0 right-0 h-px bg-white/30"></div>
                </div>

                <!-- Circle outline -->
                <div class="absolute inset-0 rounded-full border border-white/50"></div>
              </div>
            </div>
          </div>
        </div>

        <!-- Drag hint -->
        <p class="text-xs text-gray-500 text-center mb-4">Drag up or down to adjust crop area</p>

        <div class="flex justify-end space-x-3">
          <button
            @click="$emit('close')"
            class="px-4 py-2 text-sm text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50"
          >
            Cancel
          </button>
          <button
            @click="handleConfirm"
            :disabled="isUploading"
            class="px-4 py-2 text-sm bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50"
          >
            {{ isUploading ? 'Uploading...' : 'Confirm' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from 'vue';

interface Props {
  isOpen: boolean;
  previewUrl: string;
  title?: string;
  isUploading?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  title: 'Preview Avatar',
  isUploading: false,
});

interface Emits {
  (e: 'close'): void;
  (e: 'confirm', cropRatio: number): void; // cropRatio: 0-1, position from top
}

const emit = defineEmits<Emits>();

// Generate unique mask ID to avoid conflicts when multiple dialogs exist
const maskId = computed(() => `avatarMask_${Math.random().toString(36).substr(2, 9)}`);

const containerRef = ref<HTMLElement>();
const imageRef = ref<HTMLImageElement>();

const cropBox = ref({ x: 0, y: 0, size: 0 });
const imageRect = ref({ width: 0, height: 0, left: 0, top: 0 });
const containerRect = ref({ width: 0, height: 0, left: 0, top: 0 });

const isDragging = ref(false);
const dragStartY = ref(0);
const cropStartY = ref(0);

// Calculate the crop ratio (0-1) representing position from top
const cropRatio = computed(() => {
  if (imageRect.value.height <= cropBox.value.size) return 0;

  const imgOffsetY = imageRect.value.top - containerRect.value.top;
  const cropRelativeToImage = cropBox.value.y - imgOffsetY;
  const maxOffset = imageRect.value.height - cropBox.value.size;

  if (maxOffset <= 0) return 0;
  return Math.max(0, Math.min(1, cropRelativeToImage / maxOffset));
});

function onImageLoad() {
  nextTick(() => {
    if (!imageRef.value || !containerRef.value) return;

    const img = imageRef.value;
    const container = containerRef.value;
    const imgBounds = img.getBoundingClientRect();
    const containerBounds = container.getBoundingClientRect();

    imageRect.value = {
      width: imgBounds.width,
      height: imgBounds.height,
      left: imgBounds.left,
      top: imgBounds.top,
    };

    containerRect.value = {
      width: containerBounds.width,
      height: containerBounds.height,
      left: containerBounds.left,
      top: containerBounds.top,
    };

    // Crop size = shorter side of image
    const cropSize = Math.min(imgBounds.width, imgBounds.height);

    // Position relative to container
    const imgOffsetX = imgBounds.left - containerBounds.left;
    const imgOffsetY = imgBounds.top - containerBounds.top;

    // Center horizontally, start from top vertically
    cropBox.value = {
      x: imgOffsetX + (imgBounds.width - cropSize) / 2,
      y: imgOffsetY,
      size: cropSize,
    };
  });
}

function startDrag(e: MouseEvent | TouchEvent) {
  e.preventDefault();
  e.stopPropagation();
  isDragging.value = true;

  const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;
  dragStartY.value = clientY;
  cropStartY.value = cropBox.value.y;

  // Add document-level listeners for smooth dragging
  document.addEventListener('mousemove', handleDrag);
  document.addEventListener('mouseup', stopDrag);
  document.addEventListener('touchmove', handleDrag);
  document.addEventListener('touchend', stopDrag);
}

function handleDrag(e: MouseEvent | TouchEvent) {
  if (!isDragging.value) return;
  e.preventDefault();

  const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;
  const deltaY = clientY - dragStartY.value;

  // Calculate bounds
  const imgOffsetY = imageRect.value.top - containerRect.value.top;
  const minY = imgOffsetY;
  const maxY = imgOffsetY + imageRect.value.height - cropBox.value.size;

  // Apply new position with bounds
  const newY = Math.max(minY, Math.min(maxY, cropStartY.value + deltaY));
  cropBox.value.y = newY;
}

function stopDrag() {
  isDragging.value = false;

  // Remove document-level listeners
  document.removeEventListener('mousemove', handleDrag);
  document.removeEventListener('mouseup', stopDrag);
  document.removeEventListener('touchmove', handleDrag);
  document.removeEventListener('touchend', stopDrag);
}

function handleConfirm() {
  emit('confirm', cropRatio.value);
}

// Reset when dialog opens/closes
watch(() => props.isOpen, (newVal) => {
  if (newVal) {
    cropBox.value = { x: 0, y: 0, size: 0 };
  } else {
    // Cleanup when dialog closes
    stopDrag();
  }
});

// Cleanup on unmount
onUnmounted(() => {
  stopDrag();
});
</script>

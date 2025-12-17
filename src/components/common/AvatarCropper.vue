<template>
  <div v-if="isOpen" class="fixed inset-0 z-[60] flex items-center justify-center">
    <!-- Backdrop -->
    <div class="absolute inset-0 bg-black/70" @click="handleClose"></div>

    <!-- Dialog -->
    <div class="relative w-full max-w-[480px] mx-4 rounded-2xl overflow-hidden" style="background: #1a1a1a;">
      <!-- Top bar -->
      <div class="flex items-center justify-between px-4 py-3">
        <!-- Reset button -->
        <button
          @click="handleReset"
          class="w-10 h-10 flex items-center justify-center text-white/80 hover:text-white hover:bg-white/10 rounded-full transition-colors"
          title="Reset"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>

        <!-- Close button -->
        <button
          @click="handleClose"
          class="w-10 h-10 flex items-center justify-center text-white/80 hover:text-white hover:bg-white/10 rounded-full transition-colors"
          title="Close"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Canvas container -->
      <div
        ref="containerRef"
        class="relative mx-6 select-none"
        style="height: 360px; background: #000;"
      >
        <!-- Canvas for image -->
        <canvas
          ref="canvasRef"
          class="absolute inset-0 w-full h-full"
          :class="isDragging ? 'cursor-grabbing' : 'cursor-grab'"
          style="touch-action: none;"
          @mousedown="handleMouseDown"
          @touchstart="handleTouchStart"
        ></canvas>

        <!-- SVG Mask overlay -->
        <svg class="absolute inset-0 w-full h-full pointer-events-none">
          <defs>
            <mask :id="maskId">
              <rect width="100%" height="100%" fill="white"/>
              <circle :cx="circleCenterX" :cy="circleCenterY" :r="circleRadius" fill="black"/>
            </mask>
          </defs>
          <rect width="100%" height="100%" fill="rgba(0,0,0,0.6)" :mask="`url(#${maskId})`"/>
        </svg>

        <!-- Circle border -->
        <div
          class="absolute pointer-events-none border-[3px] border-white/80 rounded-full"
          :style="{
            width: circleRadius * 2 + 'px',
            height: circleRadius * 2 + 'px',
            left: circleCenterX - circleRadius + 'px',
            top: circleCenterY - circleRadius + 'px',
          }"
        ></div>
      </div>

      <!-- Zoom controls -->
      <div class="flex items-center justify-center gap-4 px-6 py-4">
        <!-- Zoom out button -->
        <button
          @click="handleZoomOut"
          :disabled="scale <= minScale"
          class="w-10 h-10 flex items-center justify-center text-white/80 hover:text-white hover:bg-white/10 rounded-full transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
          </svg>
        </button>

        <!-- Zoom slider -->
        <input
          type="range"
          v-model.number="scale"
          :min="minScale"
          :max="maxScale"
          step="0.01"
          class="flex-1 h-2 bg-white/20 rounded-full appearance-none cursor-pointer zoom-slider"
        />

        <!-- Zoom in button -->
        <button
          @click="handleZoomIn"
          :disabled="scale >= maxScale"
          class="w-10 h-10 flex items-center justify-center text-white/80 hover:text-white hover:bg-white/10 rounded-full transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
        </button>
      </div>

      <!-- Save button -->
      <div class="flex justify-center px-6 pb-6">
        <button
          @click="handleSave"
          :disabled="isSaving"
          class="px-12 py-3 bg-green-400 hover:bg-green-500 text-black font-semibold rounded-lg transition-all hover:-translate-y-0.5 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:translate-y-0"
          style="box-shadow: 0 4px 12px rgba(74, 222, 128, 0.3);"
        >
          {{ isSaving ? 'Saving...' : 'Save' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted, nextTick } from 'vue';

interface Props {
  isOpen: boolean;
  imageSrc: string;
}

const props = defineProps<Props>();

interface Emits {
  (e: 'close'): void;
  (e: 'save', blob: Blob): void;
}

const emit = defineEmits<Emits>();

// Generate unique mask ID
const maskId = computed(() => `avatarCropMask_${Math.random().toString(36).substr(2, 9)}`);

// Refs
const containerRef = ref<HTMLElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const imageRef = ref<HTMLImageElement | null>(null);

// State
const scale = ref(1);
const minScale = ref(0.5);
const maxScale = ref(3);
const position = ref({ x: 0, y: 0 });
const isDragging = ref(false);
const dragStart = ref({ x: 0, y: 0 });
const isSaving = ref(false);

// Initial state for reset
const initialScale = ref(1);
const initialPosition = ref({ x: 0, y: 0 });

// Circle dimensions (fixed 280px diameter centered in 360px container)
const containerSize = 360;
const circleRadius = 140;
const circleCenterX = computed(() => containerRef.value ? containerRef.value.clientWidth / 2 : containerSize / 2);
const circleCenterY = containerSize / 2;

// Load image
async function loadImage(): Promise<void> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => {
      imageRef.value = img;
      resolve();
    };
    img.onerror = reject;
    img.src = props.imageSrc;
  });
}

// Calculate initial scale to fit image in circle
function calculateInitialScale(): void {
  if (!imageRef.value || !containerRef.value) return;

  const img = imageRef.value;
  const circleSize = circleRadius * 2;

  // Scale so the smaller dimension fills the circle (this is the minimum scale)
  const scaleX = circleSize / img.width;
  const scaleY = circleSize / img.height;
  const fitScale = Math.max(scaleX, scaleY);

  scale.value = fitScale;
  initialScale.value = fitScale;
  // minScale is the scale where the smaller dimension exactly fills the circle
  minScale.value = fitScale;
  maxScale.value = fitScale * 3;
}

// Clamp position to ensure crop circle stays within image bounds
function clampPosition(pos: { x: number; y: number }, currentScale: number): { x: number; y: number } {
  if (!imageRef.value || !containerRef.value) return pos;

  const containerWidth = containerRef.value.clientWidth;
  const img = imageRef.value;
  const scaledWidth = img.width * currentScale;
  const scaledHeight = img.height * currentScale;

  const centerX = containerWidth / 2;
  const centerY = containerSize / 2;

  // Circle bounds
  const circleLeft = centerX - circleRadius;
  const circleRight = centerX + circleRadius;
  const circleTop = centerY - circleRadius;
  const circleBottom = centerY + circleRadius;

  // Image must cover the circle:
  // - Image left edge must be <= circle left edge
  // - Image right edge must be >= circle right edge
  // - Image top edge must be <= circle top edge
  // - Image bottom edge must be >= circle bottom edge

  let x = pos.x;
  let y = pos.y;

  // Clamp X: image left <= circle left, image right >= circle right
  const maxX = circleLeft; // image left edge can't go past circle left
  const minX = circleRight - scaledWidth; // image right edge must reach circle right
  x = Math.min(maxX, Math.max(minX, x));

  // Clamp Y: image top <= circle top, image bottom >= circle bottom
  const maxY = circleTop; // image top edge can't go past circle top
  const minY = circleBottom - scaledHeight; // image bottom edge must reach circle bottom
  y = Math.min(maxY, Math.max(minY, y));

  return { x, y };
}

// Center image in the circle
function centerImage(): void {
  if (!imageRef.value || !containerRef.value) return;

  const containerWidth = containerRef.value.clientWidth;
  const img = imageRef.value;
  const scaledWidth = img.width * scale.value;
  const scaledHeight = img.height * scale.value;

  const centeredPos = {
    x: (containerWidth - scaledWidth) / 2,
    y: (containerSize - scaledHeight) / 2,
  };
  position.value = clampPosition(centeredPos, scale.value);
  initialPosition.value = { ...position.value };
}

// Draw image on canvas
function drawImage(): void {
  if (!canvasRef.value || !imageRef.value || !containerRef.value) return;

  const canvas = canvasRef.value;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const containerWidth = containerRef.value.clientWidth;

  // Set canvas size
  canvas.width = containerWidth;
  canvas.height = containerSize;

  // Clear canvas
  ctx.clearRect(0, 0, containerWidth, containerSize);

  // Draw image
  const img = imageRef.value;
  const scaledWidth = img.width * scale.value;
  const scaledHeight = img.height * scale.value;

  ctx.drawImage(
    img,
    position.value.x,
    position.value.y,
    scaledWidth,
    scaledHeight
  );
}

// Mouse handlers
function handleMouseDown(e: MouseEvent): void {
  e.preventDefault();
  isDragging.value = true;
  dragStart.value = {
    x: e.clientX - position.value.x,
    y: e.clientY - position.value.y,
  };

  window.addEventListener('mousemove', handleMouseMove);
  window.addEventListener('mouseup', handleMouseUp);
}

function handleMouseMove(e: MouseEvent): void {
  if (!isDragging.value) return;
  e.preventDefault();

  const newPos = {
    x: e.clientX - dragStart.value.x,
    y: e.clientY - dragStart.value.y,
  };
  position.value = clampPosition(newPos, scale.value);
  drawImage();
}

function handleMouseUp(): void {
  isDragging.value = false;
  window.removeEventListener('mousemove', handleMouseMove);
  window.removeEventListener('mouseup', handleMouseUp);
}

// Touch handlers
function handleTouchStart(e: TouchEvent): void {
  e.preventDefault();
  if (e.touches.length !== 1) return;

  isDragging.value = true;
  const touch = e.touches[0];
  dragStart.value = {
    x: touch.clientX - position.value.x,
    y: touch.clientY - position.value.y,
  };

  window.addEventListener('touchmove', handleTouchMove, { passive: false });
  window.addEventListener('touchend', handleTouchEnd);
}

function handleTouchMove(e: TouchEvent): void {
  if (!isDragging.value || e.touches.length !== 1) return;
  e.preventDefault();

  const touch = e.touches[0];
  const newPos = {
    x: touch.clientX - dragStart.value.x,
    y: touch.clientY - dragStart.value.y,
  };
  position.value = clampPosition(newPos, scale.value);
  drawImage();
}

function handleTouchEnd(): void {
  isDragging.value = false;
  window.removeEventListener('touchmove', handleTouchMove);
  window.removeEventListener('touchend', handleTouchEnd);
}

// Zoom handlers
function handleZoomIn(): void {
  const newScale = Math.min(scale.value + 0.1, maxScale.value);
  applyZoom(newScale);
}

function handleZoomOut(): void {
  const newScale = Math.max(scale.value - 0.1, minScale.value);
  applyZoom(newScale);
}

function applyZoom(newScale: number): void {
  if (!containerRef.value || !imageRef.value) return;

  const containerWidth = containerRef.value.clientWidth;

  // Calculate the center of the circle in canvas coordinates
  const centerX = containerWidth / 2;
  const centerY = containerSize / 2;

  // Calculate the point on the image that's currently at the center
  const imgCenterX = (centerX - position.value.x) / scale.value;
  const imgCenterY = (centerY - position.value.y) / scale.value;

  // Update scale
  scale.value = newScale;

  // Adjust position to keep the same image point at center, then clamp
  const newPos = {
    x: centerX - imgCenterX * newScale,
    y: centerY - imgCenterY * newScale,
  };
  position.value = clampPosition(newPos, newScale);

  drawImage();
}

// Watch scale changes from slider
watch(scale, (newScale, oldScale) => {
  if (newScale !== oldScale && containerRef.value && imageRef.value) {
    const containerWidth = containerRef.value.clientWidth;
    const centerX = containerWidth / 2;
    const centerY = containerSize / 2;

    const imgCenterX = (centerX - position.value.x) / oldScale;
    const imgCenterY = (centerY - position.value.y) / oldScale;

    const newPos = {
      x: centerX - imgCenterX * newScale,
      y: centerY - imgCenterY * newScale,
    };
    position.value = clampPosition(newPos, newScale);

    drawImage();
  }
});

// Reset handler
function handleReset(): void {
  scale.value = initialScale.value;
  position.value = { ...initialPosition.value };
  drawImage();
}

// Close handler
function handleClose(): void {
  emit('close');
}

// Save handler
async function handleSave(): Promise<void> {
  if (!canvasRef.value || !containerRef.value || isSaving.value) return;

  isSaving.value = true;

  try {
    const containerWidth = containerRef.value.clientWidth;
    const centerX = containerWidth / 2;
    const centerY = containerSize / 2;

    // Create output canvas (460x460)
    const outputCanvas = document.createElement('canvas');
    outputCanvas.width = 460;
    outputCanvas.height = 460;
    const ctx = outputCanvas.getContext('2d');
    if (!ctx) throw new Error('Failed to get canvas context');

    // Create circular clip
    ctx.beginPath();
    ctx.arc(230, 230, 230, 0, Math.PI * 2);
    ctx.closePath();
    ctx.clip();

    // Calculate source region from the main canvas
    const sourceX = centerX - circleRadius;
    const sourceY = centerY - circleRadius;
    const sourceSize = circleRadius * 2;

    // Draw the circular region scaled to 460x460
    ctx.drawImage(
      canvasRef.value,
      sourceX,
      sourceY,
      sourceSize,
      sourceSize,
      0,
      0,
      460,
      460
    );

    // Convert to blob
    outputCanvas.toBlob((blob) => {
      if (blob) {
        emit('save', blob);
      } else {
        console.error('Failed to create blob');
      }
      isSaving.value = false;
    }, 'image/png');
  } catch (error) {
    console.error('Error saving image:', error);
    isSaving.value = false;
  }
}

// Initialize when dialog opens
async function initialize(): Promise<void> {
  if (!props.isOpen || !props.imageSrc) return;

  await nextTick();

  try {
    await loadImage();
    calculateInitialScale();
    centerImage();
    drawImage();
  } catch (error) {
    console.error('Error loading image:', error);
  }
}

// Watch for dialog open
watch(
  () => props.isOpen,
  async (isOpen) => {
    if (isOpen) {
      await initialize();
    } else {
      // Cleanup
      handleMouseUp();
      handleTouchEnd();
      isDragging.value = false;
      isSaving.value = false;
    }
  }
);

// Watch for image source change
watch(
  () => props.imageSrc,
  async () => {
    if (props.isOpen) {
      await initialize();
    }
  }
);

// Cleanup on unmount
onUnmounted(() => {
  handleMouseUp();
  handleTouchEnd();
});
</script>

<style scoped>
.zoom-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 20px;
  height: 20px;
  background: #4ade80;
  border-radius: 50%;
  border: 2px solid white;
  cursor: pointer;
  transition: transform 0.15s;
}

.zoom-slider::-webkit-slider-thumb:hover {
  transform: scale(1.1);
}

.zoom-slider::-moz-range-thumb {
  width: 20px;
  height: 20px;
  background: #4ade80;
  border-radius: 50%;
  border: 2px solid white;
  cursor: pointer;
  transition: transform 0.15s;
}

.zoom-slider::-moz-range-thumb:hover {
  transform: scale(1.1);
}
</style>

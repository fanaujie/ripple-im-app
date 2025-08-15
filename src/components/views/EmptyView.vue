<template>
  <div class="flex flex-col items-center justify-center p-8">
    <div class="text-center max-w-md bg-surface rounded-xl shadow-sm border border-border p-8">
      <!-- Icon -->
      <div class="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mx-auto mb-6">
        <HeroIcon :name="icon" className="w-8 h-8 text-primary" />
      </div>
      
      <!-- Title -->
      <h2 class="text-2xl font-semibold text-text mb-3">
        {{ title }}
      </h2>
      
      <!-- Description -->
      <p class="text-text-secondary mb-6 leading-relaxed">
        {{ description }}
      </p>
      
      <!-- Coming Soon Badge -->
      <div class="inline-flex items-center gap-2 px-4 py-2 bg-accent/10 text-accent rounded-full text-sm font-medium">
        <div class="w-2 h-2 bg-accent rounded-full animate-pulse"></div>
        Coming Soon
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import HeroIcon from '../shared/HeroIcon.vue';

interface Props {
  title: string;
  description: string;
  icon: string;
}

withDefaults(defineProps<Props>(), {
  title: 'Coming Soon',
  description: 'This feature is under development and will be available soon.',
  icon: 'cog-6-tooth'
});
</script>
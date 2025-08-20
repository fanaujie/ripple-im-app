<template>
  <div class="h-full flex flex-col">
    <!-- Header -->
    <div class="bg-surface border-b border-border p-4">
      <h2 class="text-lg font-semibold text-text">Add Friend</h2>
    </div>

    <!-- Add Friend Form -->
    <div class="flex-1 p-6">
      <div class="max-w-md mx-auto">
        <!-- Input Field -->
        <div class="mb-6">
          <div class="relative">
            <input
              id="friendAccount"
              v-model="accountInput"
              type="text"
              placeholder="user@example.com"
              class="w-full px-4 py-3 bg-background border border-border rounded-lg text-text placeholder-text-secondary focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent"
              :disabled="loading"
              @keyup.enter="sendFriendRequest"
            />
            <HeroIcon 
              name="at-symbol" 
              className="absolute right-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-text-secondary"
            />
          </div>
          <p v-if="inputError" class="text-danger text-sm mt-2">
            {{ inputError }}
          </p>
        </div>

        <!-- Send Button -->
        <button
          @click="sendFriendRequest"
          :disabled="loading || !accountInput.trim()"
          class="w-full px-6 py-3 bg-primary text-white rounded-lg font-medium hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center"
        >
          <div v-if="loading" class="animate-spin rounded-full h-4 w-4 border-2 border-white/30 border-t-white mr-2"></div>
          <HeroIcon v-else name="paper-airplane" className="w-4 h-4 mr-2" />
          {{ loading ? 'Sending...' : 'Send Request' }}
        </button>

        <!-- Success Message -->
        <div v-if="successMessage" class="mt-4 p-4 bg-success/10 border border-success/20 rounded-lg">
          <div class="flex items-center text-success">
            <HeroIcon name="check-circle" className="w-5 h-5 mr-2" />
            <span>{{ successMessage }}</span>
          </div>
        </div>

        <!-- Error Message -->
        <div v-if="errorMessage" class="mt-4 p-4 bg-danger/10 border border-danger/20 rounded-lg">
          <div class="flex items-center text-danger">
            <HeroIcon name="x-circle" className="w-5 h-5 mr-2" />
            <span>{{ errorMessage }}</span>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import HeroIcon from '../shared/HeroIcon.vue';

const emit = defineEmits<{
  requestSent: [];
}>();

const accountInput = ref('');
const loading = ref(false);
const inputError = ref('');
const successMessage = ref('');
const errorMessage = ref('');

const validateEmail = (email: string): boolean => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
};

const clearMessages = () => {
  inputError.value = '';
  successMessage.value = '';
  errorMessage.value = '';
};

const sendFriendRequest = async () => {
  if (!accountInput.value.trim()) {
    inputError.value = 'Enter account';
    return;
  }

  if (!validateEmail(accountInput.value.trim())) {
    inputError.value = 'Invalid format';
    return;
  }

  try {
    loading.value = true;
    clearMessages();

    const success = await invoke<boolean>('send_friend_request', {
      account: accountInput.value.trim()
    });

    if (success) {
      successMessage.value = `Request sent to ${accountInput.value}`;
      accountInput.value = '';
      emit('requestSent');
      
      // Auto clear success message after 3 seconds
      setTimeout(() => {
        successMessage.value = '';
      }, 3000);
    } else {
      errorMessage.value = 'Failed to send request';
    }
  } catch (err) {
    errorMessage.value = err instanceof Error ? err.message : 'Failed to send request';
  } finally {
    loading.value = false;
  }
};

// Clear input error when user types
watch(accountInput, () => {
  if (inputError.value) {
    inputError.value = '';
  }
  if (errorMessage.value) {
    errorMessage.value = '';
  }
});
</script>
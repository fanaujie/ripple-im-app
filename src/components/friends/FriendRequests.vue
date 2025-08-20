<template>
  <div class="h-full flex flex-col">
    <!-- Content -->
    <div class="flex-1 overflow-y-auto">
      <div v-if="loading" class="flex items-center justify-center p-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
      
      <div v-else-if="error" class="p-4 text-center text-danger">
        {{ error }}
      </div>
      
      <!-- Received Requests -->
      <div v-if="props.activeTab === 'received'">
        <div v-if="receivedRequests.length === 0" class="p-8">
          <EmptyView
            title="No New Requests"
            description="You have no pending friend requests."
            icon="inbox"
          />
        </div>
        
        <div v-else class="p-2">
          <div 
            v-for="request in receivedRequests" 
            :key="request.id"
            class="bg-surface rounded-lg p-4 mb-3 border border-border"
          >
            <div class="flex items-center">
              <!-- Avatar -->
              <div class="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center mr-3 overflow-hidden">
                <img 
                  v-if="request.fromAvatar" 
                  :src="request.fromAvatar" 
                  :alt="request.fromNickName"
                  class="w-full h-full object-cover"
                />
                <HeroIcon v-else name="user" className="w-6 h-6 text-primary" />
              </div>
              
              <!-- Request Info -->
              <div class="flex-1 min-w-0">
                <h3 class="font-medium text-text">{{ request.fromNickName }}</h3>
                <p class="text-sm text-text-secondary truncate">{{ request.fromAccount }}</p>
                <p class="text-xs text-text-secondary mt-1">{{ formatDate(request.createdAt) }}</p>
              </div>
              
              <!-- Action Buttons -->
              <div class="flex space-x-2 ml-4">
                <button
                  @click="handleRequest(request.id, true)"
                  :disabled="processingRequests.has(request.id)"
                  class="px-4 py-2 bg-success text-white text-sm rounded-lg hover:bg-success/90 transition-colors disabled:opacity-50"
                >
                  Accept
                </button>
                <button
                  @click="handleRequest(request.id, false)"
                  :disabled="processingRequests.has(request.id)"
                  class="px-4 py-2 bg-danger text-white text-sm rounded-lg hover:bg-danger/90 transition-colors disabled:opacity-50"
                >
                  Decline
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
      
      <!-- Sent Requests -->
      <div v-else-if="props.activeTab === 'sent'">
        <div v-if="sentRequests.length === 0" class="p-8">
          <EmptyView
            title="No Sent Requests"
            description="You haven't sent any friend requests yet."
            icon="paper-airplane"
          />
        </div>
        
        <div v-else class="p-2">
          <div 
            v-for="request in sentRequests" 
            :key="request.id"
            class="bg-surface rounded-lg p-4 mb-3 border border-border"
          >
            <div class="flex items-center">
              <!-- Status Icon -->
              <div class="w-10 h-10 rounded-full flex items-center justify-center mr-3"
                   :class="getStatusIconClass(request.status)">
                <HeroIcon :name="getStatusIcon(request.status)" className="w-5 h-5" />
              </div>
              
              <!-- Request Info -->
              <div class="flex-1 min-w-0">
                <h3 class="font-medium text-text">Sent to {{ request.toAccount }}</h3>
                <p class="text-sm" :class="getStatusTextClass(request.status)">
                  {{ getStatusText(request.status) }}
                </p>
                <p class="text-xs text-text-secondary mt-1">{{ formatDate(request.createdAt) }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { FriendRequest, FriendRequestStatus } from '../../types/friends';
import HeroIcon from '../shared/HeroIcon.vue';
import EmptyView from '../views/EmptyView.vue';

interface Props {
  activeTab: 'received' | 'sent';
}

const props = withDefaults(defineProps<Props>(), {
  activeTab: 'received'
});
const receivedRequests = ref<FriendRequest[]>([]);
const sentRequests = ref<FriendRequest[]>([]);
const loading = ref(false);
const error = ref('');
const processingRequests = ref<Set<string>>(new Set());

const loadRequests = async () => {
  try {
    loading.value = true;
    error.value = '';
    
    const [received, sent] = await Promise.all([
      invoke<FriendRequest[]>('get_friend_requests'),
      invoke<FriendRequest[]>('get_sent_requests')
    ]);
    
    receivedRequests.value = received;
    sentRequests.value = sent;
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load requests';
  } finally {
    loading.value = false;
  }
};

const handleRequest = async (requestId: string, accept: boolean) => {
  try {
    processingRequests.value.add(requestId);
    
    await invoke('handle_friend_request', { 
      requestId, 
      accept 
    });
    
    await loadRequests();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to handle request';
  } finally {
    processingRequests.value.delete(requestId);
  }
};

const formatDate = (dateString: string): string => {
  const date = new Date(dateString);
  return date.toLocaleDateString('zh-TW', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
};

const getStatusIcon = (status: FriendRequestStatus): string => {
  switch (status) {
    case 'pending': return 'clock';
    case 'accepted': return 'check-circle';
    case 'rejected': return 'x-circle';
    default: return 'question-mark-circle';
  }
};

const getStatusIconClass = (status: FriendRequestStatus): string => {
  switch (status) {
    case 'pending': return 'bg-accent/10 text-accent';
    case 'accepted': return 'bg-success/10 text-success';
    case 'rejected': return 'bg-danger/10 text-danger';
    default: return 'bg-border text-text-secondary';
  }
};

const getStatusText = (status: FriendRequestStatus): string => {
  switch (status) {
    case 'pending': return 'Pending';
    case 'accepted': return 'Accepted';
    case 'rejected': return 'Declined';
    default: return 'Unknown';
  }
};

const getStatusTextClass = (status: FriendRequestStatus): string => {
  switch (status) {
    case 'pending': return 'text-accent';
    case 'accepted': return 'text-success';
    case 'rejected': return 'text-danger';
    default: return 'text-text-secondary';
  }
};

onMounted(() => {
  loadRequests();
});

defineExpose({
  loadRequests
});
</script>
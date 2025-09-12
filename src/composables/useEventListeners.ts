import { onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { NewMessageEvent, FriendUpdateEvent, MessagesReadEvent, UserProfileData } from '../types/app-state';
import { useAppState } from './useAppState';
import { useUserProfile } from './useUserProfile';

export function useEventListeners() {
  const { updateConversationWithNewMessage, updateFriend, updateMessagesRead } = useAppState();
  const { updateUserProfile } = useUserProfile();
  
  let unlistenFns: UnlistenFn[] = [];

  const startListening = async () => {
    try {
      // Listen for new messages
      const unlistenNewMessage = await listen<NewMessageEvent>('new_message', (event) => {
        console.log('Received new message:', event.payload);
        const { message, total_unread } = event.payload;
        updateConversationWithNewMessage(message, total_unread);
      });

      // Listen for friend updates
      const unlistenFriendUpdate = await listen<FriendUpdateEvent>('friend_update', (event) => {
        console.log('Received friend update:', event.payload);
        const { friend } = event.payload;
        updateFriend(friend);
      });

      // Listen for messages read events
      const unlistenMessagesRead = await listen<MessagesReadEvent>('messages_read', (event) => {
        console.log('Messages marked as read:', event.payload);
        const { conversation_id, total_unread } = event.payload;
        updateMessagesRead(conversation_id, total_unread);
      });

      // Listen for user profile updates
      const unlistenUserProfile = await listen<UserProfileData>('user-profile-updated', (event) => {
        console.log('Received user profile update:', event.payload);
        updateUserProfile(event.payload);
      });

      // Store unlisten functions
      unlistenFns = [
        unlistenNewMessage,
        unlistenFriendUpdate,
        unlistenMessagesRead,
        unlistenUserProfile,
      ];

      console.log('Event listeners started');
    } catch (error) {
      console.error('Failed to start event listeners:', error);
    }
  };

  const stopListening = () => {
    unlistenFns.forEach(fn => fn());
    unlistenFns = [];
    console.log('Event listeners stopped');
  };

  // Auto start/stop listeners
  onMounted(() => {
    startListening();
  });

  onUnmounted(() => {
    stopListening();
  });

  return {
    startListening,
    stopListening,
  };
}
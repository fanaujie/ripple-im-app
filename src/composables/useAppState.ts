import {ref, reactive, computed, readonly} from 'vue';
import {invoke} from '@tauri-apps/api/core';
import type {AppState, Friend, Conversation, Message} from '../types/app-state';

// Global reactive state
const appState = reactive<AppState>({
    current_user: undefined,
    friends: [],
    conversations: [],
    total_unread: 0,
});

// Loading states
const loading = ref({
    app: false,
    conversations: false,
    messages: false,
    friends: false,
});

// Error states
const errors = ref({
    app: null as string | null,
    conversations: null as string | null,
    messages: null as string | null,
    friends: null as string | null,
});

export function useAppState() {
    // Computed getters
    const currentUser = computed(() => appState.current_user);
    const friends = computed(() => appState.friends);
    const conversations = computed(() => appState.conversations);
    const totalUnread = computed(() => appState.total_unread);

    // Load initial app state
    const loadAppState = async () => {
        try {
            loading.value.app = true;
            errors.value.app = null;

            const state = await invoke('get_app_state') as AppState;

            appState.current_user = state.current_user;
            appState.friends = state.friends;
            appState.conversations = state.conversations;
            appState.total_unread = state.total_unread;

        } catch (error) {
            console.error('Failed to load app state:', error);
            errors.value.app = error as string;
        } finally {
            loading.value.app = false;
        }
    };

    // Load conversations
    const loadConversations = async () => {
        try {
            loading.value.conversations = true;
            errors.value.conversations = null;

            const conversations = await invoke('get_conversations') as Conversation[];
            appState.conversations = conversations;

        } catch (error) {
            console.error('Failed to load conversations:', error);
            errors.value.conversations = error as string;
        } finally {
            loading.value.conversations = false;
        }
    };

    // Load friends
    const loadFriends = async () => {
        try {
            loading.value.friends = true;
            errors.value.friends = null;

            const friends = await invoke('get_friends_list') as Friend[];
            appState.friends = friends;

        } catch (error) {
            console.error('Failed to load friends:', error);
            errors.value.friends = error as string;
        } finally {
            loading.value.friends = false;
        }
    };

    // Get messages for a conversation
    const getMessages = async (conversationId: string): Promise<Message[]> => {
        try {
            loading.value.messages = true;
            errors.value.messages = null;

            return await invoke('get_messages', {conversationId}) as Message[];

        } catch (error) {
            console.error('Failed to load messages:', error);
            errors.value.messages = error as string;
            return [];
        } finally {
            loading.value.messages = false;
        }
    };

    // Mark messages as read
    const markMessagesRead = async (conversationId: string) => {
        try {
            await invoke('mark_messages_read', {conversationId});

            // Update local state
            const conversation = appState.conversations.find(c => c.id === conversationId);
            if (conversation) {
                conversation.unread_count = 0;
            }

            // Recalculate total unread
            appState.total_unread = appState.conversations.reduce((sum, conv) => sum + conv.unread_count, 0);

        } catch (error) {
            console.error('Failed to mark messages read:', error);
        }
    };


    // State update methods (called by event listeners)
    const updateConversationWithNewMessage = (message: Message, totalUnread: number) => {
        // Update conversation list
        const conversation = appState.conversations.find(c => c.id === message.conversation_id);
        if (conversation) {
            conversation.last_message = message;
            conversation.last_activity = message.timestamp;
            if (message.sender_id !== appState.current_user?.id) {
                conversation.unread_count += 1;
            }

            // Move conversation to top
            const index = appState.conversations.indexOf(conversation);
            appState.conversations.splice(index, 1);
            appState.conversations.unshift(conversation);
        }

        // Update total unread count
        appState.total_unread = totalUnread;
    };

    const updateFriend = (friend: Friend) => {
        const index = appState.friends.findIndex(f => f.id === friend.id);
        if (index >= 0) {
            appState.friends[index] = friend;
        }

        // Also update friend in conversations
        appState.conversations.forEach(conversation => {
            if (conversation.friend.id === friend.id) {
                conversation.friend = friend;
            }
        });
    };

    const updateMessagesRead = (conversationId: string, totalUnread: number) => {
        const conversation = appState.conversations.find(c => c.id === conversationId);
        if (conversation) {
            conversation.unread_count = 0;
        }
        appState.total_unread = totalUnread;
    };

    return {
        // State
        appState: readonly(appState),
        loading: readonly(loading),
        errors: readonly(errors),

        // Computed
        currentUser,
        friends,
        conversations,
        totalUnread,

        // Methods
        loadAppState,
        loadConversations,
        loadFriends,
        getMessages,
        markMessagesRead,

        // State updates (for event listeners)
        updateConversationWithNewMessage,
        updateFriend,
        updateMessagesRead,
    };
}
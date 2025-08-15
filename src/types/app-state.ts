export interface User {
    id: string;
    nickname: string;
    avatar_url?: string;
    is_online: boolean;
    last_seen?: string;
}

export interface Friend {
    id: string;
    nickname: string;
    avatar_url?: string;
    is_online: boolean;
    last_seen?: string;
    status: string;
}

export interface Message {
    id: string;
    conversation_id: string;
    sender_id: string;
    content: string;
    message_type: 'text' | 'image' | 'file';
    file_url?: string;
    file_name?: string;
    file_size?: number;
    timestamp: string;
    is_read: boolean;
    delivery_status: 'sent' | 'delivered' | 'read';
}

export interface Conversation {
    id: string;
    friend: Friend;
    last_message?: Message;
    unread_count: number;
    last_activity: string;
    is_muted: boolean;
}

export interface AppState {
    current_user?: User;
    friends: Friend[];
    conversations: Conversation[];
    total_unread: number;
}

// Event Types
export interface NewMessageEvent {
    message: Message;
    conversation_id: string;
    total_unread: number;
}

export interface FriendUpdateEvent {
    friend: Friend;
    action: 'online' | 'offline' | 'status_changed';
}

export interface MessagesReadEvent {
    conversation_id: string;
    total_unread: number;
}

// User Profile Types
export interface UserProfile {
    user_id: string;
    nickname: string;
    avatar_path: string;
}

export interface UpdateAvatarResponse {
    success: boolean;
    avatar_path?: string;
}
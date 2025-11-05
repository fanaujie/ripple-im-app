import { invoke } from '@tauri-apps/api/core';
import type { UserProfileData } from '../types/app-state';

/**
 * Simplified composable for loading user profile
 * All caching is now handled in Rust StorageEngine
 */
export function useUserProfile() {
    // Load user profile from Rust StorageEngine (fast, cached in Rust)
    const loadUserProfile = async (): Promise<UserProfileData> => {
        return await invoke<UserProfileData>('get_user_profile');
    };

    return {
        loadUserProfile,
    };
}
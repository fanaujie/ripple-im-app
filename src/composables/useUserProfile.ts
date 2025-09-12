import {ref, readonly} from 'vue';
import {invoke} from '@tauri-apps/api/core';
import type {UserProfileData} from '../types/app-state';

// Global reactive state (following useAppState pattern)
const userProfile = ref<UserProfileData | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);

export function useUserProfile() {
    // Load initial user profile data
    const loadUserProfile = async () => {
        loading.value = true;
        error.value = null;

        try {
            const profile = await invoke<UserProfileData>('get_user_profile');
            userProfile.value = profile;
        } catch (err) {
            console.error('Failed to load user profile:', err);
            error.value = err instanceof Error ? err.message : 'Failed to load user profile';
        } finally {
            loading.value = false;
        }
    };

    // Update user profile (called by event listener)
    const updateUserProfile = (profile: UserProfileData) => {
        userProfile.value = profile;
    };

    return {
        // State (readonly to prevent direct mutation)
        userProfile: readonly(userProfile),
        loading: readonly(loading),
        error: readonly(error),

        // Methods
        loadUserProfile,
        updateUserProfile,
    };
}
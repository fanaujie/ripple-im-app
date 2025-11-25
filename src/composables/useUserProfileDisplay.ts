import { ref, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useUserProfile } from './useUserProfile';
import type { UserProfileData } from '../types/app-state';

/**
 * Composable for components that need to display user profile information
 * Directly listens to Tauri events for real-time updates
 */
export function useUserProfileDisplay() {
  const userProfile = ref<UserProfileData | null>(null);
  const { loadUserProfile } = useUserProfile();
  const loading = ref(false);
  const error = ref<string | null>(null);

  let unlistenProfileUpdate: UnlistenFn | null = null;

  // Load profile data from Rust StorageEngine cache
  const initializeProfile = async () => {
    loading.value = true;
    error.value = null;

    try {
      // Load from Rust StorageEngine (fast, already cached by preload_global_data)
      const profile = await loadUserProfile();
      userProfile.value = profile;
    } catch (err) {
      console.error('Failed to initialize user profile:', err);
      error.value = err instanceof Error ? err.message : 'Failed to load profile';
    } finally {
      loading.value = false;
    }
  };

  onMounted(async () => {
    // Initialize profile from cache
    await initializeProfile();

    // Listen directly to Tauri event for real-time updates
    unlistenProfileUpdate = await listen<UserProfileData>('user-profile-updated', (event) => {
      userProfile.value = event.payload;
      console.log('User profile updated from WebSocket sync:', event.payload);
    });
  });

  onUnmounted(() => {
    if (unlistenProfileUpdate) {
      unlistenProfileUpdate();
    }
  });

  return {
    userProfile,
    loading,
    error,
    // Manual refresh if needed
    refresh: initializeProfile,
  };
}
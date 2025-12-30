<script setup lang="ts">
import {onMounted, ref, onBeforeUnmount} from 'vue'
import {invoke} from "@tauri-apps/api/core";
import {message} from "@tauri-apps/plugin-dialog";
import {listen} from "@tauri-apps/api/event";
import router from "../router/router.ts";


type AuthenticationState = {
  success: boolean,
  message: string,
};

const serverStarted = ref(false);
const isSigningIn = ref(false);

const handleSignup = async () => {
  try {
    await invoke('open_signup_url');
  } catch (err: any) {
    console.error('Failed to open signup page:', err);
    await message(`Failed to open signup page: ${err?.message ?? err}`, {
      title: 'Signup Error',
      kind: 'error'
    });
  }
}

const handleSignin = async () => {
  try {
    isSigningIn.value = true;

    // Listen for auth success event from backend
    const unlisten = await listen<AuthenticationState>('auth-result', async (event) => {
      if (event.payload.success) {
        await router.replace({name: 'home'});
        unlisten();
      } else {
        console.error('Authentication failed:', event.payload.message);
        await message(`Authentication failed: ${event.payload.message}`, {
          title: 'Authentication Error',
          kind: 'error'
        });
      }
      isSigningIn.value = false;
    });

    await invoke('open_auth_url');

    // Clean up listener after some time if no response
    setTimeout(() => {
      unlisten();
      if (isSigningIn.value) {
        isSigningIn.value = false;
      }
    }, 60000); // 1 minutes timeout
  } catch (err: any) {
    console.error('Failed to open signin page:', err);
    isSigningIn.value = false;
    await message(`Failed to open signin page: ${err?.message ?? err}`, {
      title: 'Signin Error',
      kind: 'error'
    });
  }
}

onMounted(async () => {
  try {
    const exist = await invoke('exists_token');
    if (exist) {
      try {
        // Resume existing session (initialize sync and WebSocket)
        console.log('Token found, resuming session...');
        await invoke('resume_session');
        console.log('Session resumed, navigating to home');
        await router.replace({name: 'home'});
      } catch (err: any) {
        console.error('Failed to resume session:', err);
        await message(`Failed to resume session: ${err?.message ?? err}`, {
          title: 'Session Error',
          kind: 'error'
        });
        // Start OAuth callback server so user can re-authenticate
        try {
          await invoke('start_server');
          serverStarted.value = true;
        } catch (serverErr: any) {
          console.error('Failed to start server after session resume failure:', serverErr);
          await message(`Failed to start server: ${serverErr?.message ?? serverErr}`, {
            title: 'Server Error',
            kind: 'error'
          });
        }
      }
    } else {
      console.log('No token found, staying on login page');
      try {
        await invoke('start_server');
        serverStarted.value = true;
      } catch (err: any) {
        console.error('Failed to start server:', err);
        await message(`Failed to start server: ${err?.message ?? err}`, {
          title: 'Server Error',
          kind: 'error'
        });
      }
    }
  } catch (err: any) {
    console.error('Token validation failed:', err);
    await message(`Token validation failed: ${err?.message ?? err}`, {
      title: 'Validation Error',
      kind: 'error'
    });
  }
});

onBeforeUnmount(async () => {
  if (serverStarted.value) {
    try {
      await invoke('stop_server');
      serverStarted.value = false;
    } catch (err: any) {
      console.error('Failed to stop server on unmount:', err);
      await message(`Failed to stop server: ${err?.message ?? err}`, {
        title: 'Server Stop Error',
        kind: 'error'
      });
    }
  }
});
</script>


<template>
  <div class="login-container">
    <div class="login-card">
      <h1 class="title">Welcome to Ripple!</h1>

      <div class="buttons">
        <button
            class="btn btn-primary"
            @click="handleSignup"
            :disabled="isSigningIn"
        >
          Sign up
        </button>

        <button
            class="btn btn-secondary"
            @click="handleSignin"
            :disabled="isSigningIn"
        >
          Sign in
        </button>
      </div>
    </div>

    <!-- Loading overlay -->
    <div v-if="isSigningIn" class="loading-overlay">
      <div class="loading-content">
        <div class="spinner"></div>
        <p>Signing you in...</p>
      </div>
    </div>
  </div>
</template>


<style scoped>
.login-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #f8fafc;
  padding: 20px;
}

.login-card {
  background: #ffffff;
  padding: 48px;
  width: 100%;
  max-width: 400px;
  text-align: center;
  border-radius: 16px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.title {
  font-size: 32px;
  font-weight: 700;
  color: #1a1a1a;
  margin: 0 0 40px 0;
}

.buttons {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.btn {
  width: 100%;
  padding: 14px;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
}

.btn-primary {
  background: #2563eb;
  color: white;
  border: none;
}

.btn-primary:hover:not(:disabled) {
  background: #1d4ed8;
}

.btn-secondary {
  background: #ffffff;
  color: #374151;
  border: 1px solid #d1d5db;
}

.btn-secondary:hover:not(:disabled) {
  background: #f9fafb;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(248, 250, 252, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.loading-content {
  text-align: center;
  background: white;
  padding: 40px;
  border-radius: 16px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #e2e8f0;
  border-top: 4px solid #2563eb;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 16px;
}

.loading-content p {
  margin: 0;
  color: #6b7280;
  font-size: 16px;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}
</style>
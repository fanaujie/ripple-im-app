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
    const r = await invoke('is_token_valid');
    if (r) {
      try {
        await router.replace({name: 'home'});
      } catch (err: any) {
        console.error('Failed to navigate to home:', err);
        await message(`Failed to navigate to home: ${err?.message ?? err}`, {
          title: 'Navigation Error',
          kind: 'error'
        });
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
    <div class="login-form">
      <div class="header">
        <h1 class="title">Welcome to Ripple!</h1>
      </div>

      <button
          class="signup-btn"
          @click="handleSignup"
          :disabled="isSigningIn"
          :class="{ disabled: isSigningIn }"
      >
        Sign up
      </button>

      <p class="signin-text">
        Already have an account?
        <a
            href="#"
            class="signin-link"
            @click="handleSignin"
            :class="{ disabled: isSigningIn }"
        >
          <span v-if="!isSigningIn">Sign in</span>
          <span v-else class="signing-in">
            <span class="spinner"></span>
            Signing in...
          </span>
        </a>
      </p>
    </div>

    <!-- Loading overlay -->
    <div v-if="isSigningIn" class="loading-overlay">
      <div class="loading-content">
        <div class="large-spinner"></div>
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
  background-color: #ffffff;
  padding: 20px;
}

.login-form {
  background: #ffffff;
  padding: 48px 40px;
  width: 100%;
  max-width: 400px;
  text-align: center;
}

.header {
  margin-bottom: 48px;
}

.app-icon {
  margin-bottom: 24px;
}

.title {
  font-size: 32px;
  font-weight: 600;
  color: #1a1a1a;
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
}

.signup-btn {
  width: 100%;
  background: #007AFF;
  color: white;
  border: none;
  padding: 16px;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s;
  margin-bottom: 24px;
}

.signup-btn:hover {
  background: #0051D0;
}

.signin-text {
  color: #6b7280;
  font-size: 14px;
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
}

.signin-link {
  color: #007AFF;
  text-decoration: none;
  font-weight: 500;
}

.signin-link:hover {
  text-decoration: underline;
}

.signup-btn.disabled {
  background: #a0a0a0;
  cursor: not-allowed;
  opacity: 0.6;
}

.signup-btn.disabled:hover {
  background: #a0a0a0;
}

.signin-link.disabled {
  color: #a0a0a0;
  cursor: not-allowed;
  pointer-events: none;
  text-decoration: none;
}

.signing-in {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid #e0e0e0;
  border-top: 2px solid #007AFF;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.loading-content {
  text-align: center;
  background: white;
  padding: 40px;
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.large-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #e0e0e0;
  border-top: 4px solid #007AFF;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 16px;
}

.loading-content p {
  margin: 0;
  color: #6b7280;
  font-size: 16px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
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
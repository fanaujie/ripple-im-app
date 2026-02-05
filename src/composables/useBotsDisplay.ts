import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { BotData, BotSessionData } from '../types/bot';

/**
 * Composable for displaying and managing bots
 *
 * Features:
 * - Loads bot list from backend
 * - Provides reactive bots list and botsMap for lookup
 * - Manages bot session operations (create)
 *
 * Usage:
 * ```typescript
 * const { bots, botsMap, loading, error, refresh, createBotSession } = useBotsDisplay();
 * ```
 */
export function useBotsDisplay() {
  const bots = ref<BotData[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Map for quick lookup by botId
  const botsMap = computed(() => {
    const map = new Map<string, BotData>();
    for (const bot of bots.value) {
      map.set(bot.botId, bot);
    }
    return map;
  });

  /**
   * Load bots list from backend
   */
  async function loadBots(): Promise<void> {
    loading.value = true;
    error.value = null;

    try {
      console.log('[useBotsDisplay] Fetching bots list...');
      const data = await invoke<BotData[]>('list_bots');
      bots.value = data;
      console.log(`[useBotsDisplay] Loaded ${data.length} bots`);
    } catch (err) {
      console.error('[useBotsDisplay] Failed to load bots:', err);
      error.value = err instanceof Error ? err.message : 'Failed to load bots';
    } finally {
      loading.value = false;
    }
  }

  /**
   * Create a new session for the specified bot
   * This clears the conversation context on the server side
   * If conversationId is provided, also clears local messages and updates session ID in DB
   */
  async function createBotSession(botId: string, conversationId?: string | null): Promise<BotSessionData> {
    console.log('[useBotsDisplay] Creating new session for bot:', botId, 'conversation:', conversationId);
    return await invoke<BotSessionData>('create_bot_session', {
      botId,
      conversationId: conversationId || null,
    });
  }

  /**
   * Check if a given peerId corresponds to a bot
   */
  function isBot(peerId: string | undefined): boolean {
    if (!peerId) return false;
    return botsMap.value.has(peerId);
  }

  /**
   * Get bot data by botId
   */
  function getBot(botId: string): BotData | undefined {
    return botsMap.value.get(botId);
  }

  // Initialize on mount
  onMounted(() => {
    loadBots();
  });

  return {
    // State
    bots,
    botsMap,
    loading,
    error,

    // Methods
    refresh: loadBots,
    createBotSession,
    isBot,
    getBot,
  };
}

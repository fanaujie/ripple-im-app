export interface BotData {
  botId: string;
  name: string;
  avatar: string | null;
  description: string | null;
  responseMode: string | null;
}

export interface BotSessionData {
  sessionId: string;
  botId: string;
  createdAt: number;
  lastActiveAt: number;
}

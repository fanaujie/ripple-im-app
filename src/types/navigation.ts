export interface NavigationItem {
  id: string;
  label: string;
  icon: string;
  component: string;
  badge: number;
  enabled: boolean;
}

export interface NavigationAction {
  id: string;
  label: string;
  icon: string;
  action: string;
  style: 'default' | 'danger';
  position: 'top' | 'bottom';
}

export interface NavigationSettings {
  defaultActive: string;
  showBadges: boolean;
  logoText: string;
  logoIcon: string;
  width: string;
  enableHoverEffects: boolean;
  showLogoutButton: boolean;
  logoutPosition: 'top' | 'bottom';
}

export interface User {
  id: string;
  nickname: string;
  avatar_url?: string;
}

export const NAVIGATION_ITEMS: NavigationItem[] = [
  {
    id: "chat",
    label: "Chat", 
    icon: "chat-bubble-left-right",
    component: "ChatView",
    badge: 0,
    enabled: true
  },
  {
    id: "people",
    label: "People",
    icon: "users", 
    component: "PeopleView",
    badge: 0,
    enabled: true
  },
  {
    id: "settings",
    label: "Settings",
    icon: "cog-6-tooth",
    component: "SettingsView",
    badge: 0,
    enabled: true
  }
];

export const NAVIGATION_ACTIONS: NavigationAction[] = [
  {
    id: "logout",
    label: "Logout",
    icon: "arrow-left-on-rectangle",
    action: "logout",
    style: "danger",
    position: "bottom"
  }
];

export const NAVIGATION_SETTINGS: NavigationSettings = {
  defaultActive: "chat",
  showBadges: true,
  logoText: "Chats",
  logoIcon: "C",
  width: "256px",
  enableHoverEffects: true,
  showLogoutButton: true,
  logoutPosition: "bottom"
};
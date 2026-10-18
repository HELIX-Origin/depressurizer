export interface SteamGame {
  app_id: number;
  name: string;
  categories: string[];
  is_hidden: boolean;
  is_favorite: boolean;
  playtime_minutes: number;
  last_played: number;
  icon_hash?: string;
  genres: string[];
  tags: string[];
  developer?: string;
  publisher?: string;
  release_year?: number;
  review_score?: number;
  platforms: string[];
  vr_support?: string;
  hltb_main_story?: number | null;
  hltb_main_extras?: number | null;
  hltb_completionist?: number | null;
}

export interface SteamAccount {
  steam_id3: string;
  steam_id64?: number;
  account_name: string;
  persona_name: string;
  most_recent: boolean;
}

export interface SteamCategory {
  name: string;
  count: number;
  is_favorite: boolean;
}

export type AutoCatFilterType =
  | "Genre"
  | "Tags"
  | "DevPub"
  | "Year"
  | "Playtime"
  | "Score"
  | "Platform"
  | "VrSupport"
  | "Manual"
  | "Hltb";

export interface AutoCatRule {
  id: string;
  name: string;
  filter_type: AutoCatFilterType;
  target_category: string;
  params: Record<string, any>;
  enabled: boolean;
}

export interface AutoCatConfig {
  rules: AutoCatRule[];
  clear_existing: boolean;
  prefix?: string;
}

export interface AppSettings {
  steam_path?: string;
  api_key?: string;
  active_user?: string;
  backup_on_save: boolean;
  dense_view: boolean;
}

export interface WorkerProgress {
  task: string;
  current: number;
  total: number;
  message: string;
  done: boolean;
}

export interface AutoCatProgress {
  current: number;
  total: number;
  done: boolean;
  game: SteamGame;
}

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AppSettings,
  AutoCatConfig,
  AutoCatProgress,
  SteamAccount,
  SteamCategory,
  SteamGame,
  WorkerProgress,
} from "./types";

/** Listens for live per-game AutoCat progress frames emitted by the Rust worker. */
export function onAutoCatProgress(callback: (progress: AutoCatProgress) => void) {
  return listen<AutoCatProgress>("autocat://progress", (event) => {
    callback(event.payload);
  });
}

export async function getSettings(): Promise<AppSettings> {
  return await invoke("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return await invoke("save_settings", { newSettings: settings });
}

export async function detectSteam(): Promise<string | null> {
  return await invoke("detect_steam");
}

export async function getAccounts(): Promise<SteamAccount[]> {
  return await invoke("get_accounts");
}

export async function loadLibrary(steamId3: string): Promise<SteamGame[]> {
  return await invoke("load_library", { steamId3 });
}

export async function runAutoCat(config: AutoCatConfig): Promise<SteamGame[]> {
  return await invoke("run_autocat", { config });
}

export async function saveToSteam(
  steamId3: string,
  games: SteamGame[]
): Promise<string> {
  return await invoke("save_to_steam", { steamId3, games });
}

export async function getCategories(): Promise<SteamCategory[]> {
  return await invoke("get_categories");
}

export interface HltbTimes {
  main_story: number | null;
  main_extras: number | null;
  completionist: number | null;
}

export async function fetchHltb(appId: number): Promise<HltbTimes> {
  return await invoke("fetch_hltb", { appId });
}

export async function saveHltbTimes(appId: number, main: number | null, extras: number | null, completionist: number | null): Promise<void> {
  await invoke("save_hltb_times", { appId, mainStory: main, mainExtras: extras, completionist });
}

export async function verifyApiKey(apiKey: string): Promise<boolean> {
  return await invoke("verify_api_key", { apiKey });
}

export async function getAutoCatRules(): Promise<any[]> {
  return await invoke("get_autocat_rules");
}

export async function saveAutoCatRules(rules: any[]): Promise<void> {
  return await invoke("save_autocat_rules", { rules });
}

export function onWorkerProgress(
  callback: (progress: WorkerProgress) => void
) {
  return listen<WorkerProgress>("worker://progress", (event) => {
    callback(event.payload);
  });
}


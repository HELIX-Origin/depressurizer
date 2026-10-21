<script lang="ts">
  import "$styles/depressurizer.scss";
  import { onMount } from "svelte";
  import {
    getSettings,
    saveSettings,
    detectSteam,
    getAccounts,
    loadLibrary,
    runAutoCat,
    saveToSteam,
    getCategories,
    verifyApiKey,
    fetchHltb,
    saveHltbTimes,
    getAutoCatRules,
    saveAutoCatRules,
    onWorkerProgress,
    onAutoCatProgress,
  } from "$lib/api";
  import type {
    AppSettings,
    AutoCatConfig,
    AutoCatFilterType,
    AutoCatRule,
    SteamAccount,
    SteamCategory,
    SteamGame,
    WorkerProgress,
  } from "$lib/types";

  // Application Settings & Steam Accounts
  let settings = $state<AppSettings>({
    steam_path: "",
    api_key: "",
    active_user: "",
    backup_on_save: true,
    dense_view: false,
  });

  let accounts = $state<SteamAccount[]>([]);
  let activeAccount = $state<SteamAccount | null>(null);
  let games = $state<SteamGame[]>([]);
  let categories = $state<SteamCategory[]>([]);
  let availableTargetCategories = $derived(
    Array.from(new Set(["Favorites", "Hidden", ...categories.map((c) => c.name)]))
  );

  // Navigation, Filtering & Selection
  let searchQuery = $state("");
  let selectedCategory = $state<string | null>(null);
  let quickFilter = $state<"all" | "installed" | "uninstalled">("all");
  let selectedAppIds = $state<number[]>([]);

  // Modals
  let showSettingsModal = $state(false);
  let showAutoCatModal = $state(false);
  let showAutoCatDropdown = $state(false);
  let showNewCatModal = $state(false);
  let newCatName = $state("");
  let batchAssignCatName = $state("");

  // Status & Live Worker
  let workerProgress = $state<WorkerProgress | null>(null);
  let statusMessage = $state("Initializing...");
  let apiKeyTestStatus = $state<{ success: boolean; message: string } | null>(null);
  let isTestingKey = $state(false);

  // AutoCat Studio State
  let autoCatRules = $state<AutoCatRule[]>([]);
  let autoCatClearExisting = $state(false);
  let autoCatPrefix = $state("");
  let newRuleType = $state<AutoCatFilterType>("Genre");
  let newRuleName = $state("");
  let newRuleTarget = $state("");
  let newRuleTargetSelect = $state("__dynamic__");
  let newRuleTargetCustom = $state("");
  let newRuleParamVal = $state("");
  let newRuleParamMax = $state("");
  let newRuleHltbField = $state<"main_story" | "main_extras" | "completionist">("main_story");
  let newRuleRegexField = $state<"name" | "developer" | "publisher">("name");
  let newRuleVrOnly = $state(false);
  let showAutoCatGuide = $state(false);
  let selectedAutoCatFilter = $state<string>("hltb");
  let autoCatSelectedOnly = $state(false);
  const rangeTypes: AutoCatFilterType[] = ["Year", "Playtime", "Hltb"];


  // Live AutoCat Execution Feedback
  let isRunningAutoCat = $state(false);
  let autoCatProcessed = $state(0);
  let autoCatTotal = $state(0);
  let autoCatCurrentGame = $state("");

  onMount(async () => {
    try {
      settings = await getSettings();
      autoCatRules = await getAutoCatRules();

      const detected = await detectSteam();
      if (!settings.steam_path && detected) {
        settings.steam_path = detected;
        await saveSettings(settings);
      }

      accounts = await getAccounts();
      if (accounts.length > 0) {
        activeAccount = accounts[0];
        settings.active_user = activeAccount.steam_id3;
        await refreshLibrary();
      } else {
        statusMessage = "No local Steam accounts found.";
      }
    } catch (e: any) {
      statusMessage = `Initialization error: ${e.message || e}`;
    }

    onWorkerProgress((p) => {
      workerProgress = p;
      if (p.done) {
        setTimeout(() => {
          workerProgress = null;
        }, 3000);
      }
    });

    onAutoCatProgress((p) => {
      autoCatProcessed = p.current;
      autoCatTotal = p.total;
      autoCatCurrentGame = p.game.name;
      const idx = games.findIndex((g) => g.app_id === p.game.app_id);
      if (idx !== -1) {
        games[idx] = p.game;
      }
    });
  });

  async function refreshLibrary() {
    if (!activeAccount) return;
    try {
      statusMessage = "Loading library...";
      games = await loadLibrary(activeAccount.steam_id3);
      categories = await getCategories();
      statusMessage = `Library loaded: ${games.length} owned games ready.`;
    } catch (e: any) {
      statusMessage = `Failed to load library: ${e.message || e}`;
    }
  }

  async function handleAccountChange(id3: string) {
    const acc = accounts.find((a) => a.steam_id3 === id3);
    if (acc) {
      activeAccount = acc;
      settings.active_user = id3;
      await saveSettings(settings);
      selectedAppIds = [];
      await refreshLibrary();
    }
  }

  async function handleTestApiKey() {
    if (!settings.api_key?.trim()) {
      apiKeyTestStatus = { success: false, message: "Please enter an API key first." };
      return;
    }
    apiKeyTestStatus = null;
    isTestingKey = true;
    try {
      await verifyApiKey(settings.api_key.trim());
      apiKeyTestStatus = { success: true, message: "Valid Steam Web API Key! Connected to Valve API." };
      await saveSettings(settings);
      await refreshLibrary();
    } catch (e: any) {
      apiKeyTestStatus = { success: false, message: `${e.message || e}` };
    } finally {
      isTestingKey = false;
    }
  }

  async function handleSaveSettings() {
    await saveSettings(settings);
    showSettingsModal = false;
    statusMessage = "Settings saved to native SQLite database.";
    await refreshLibrary();
  }

  async function handleSaveToSteam() {
    if (!activeAccount) return;
    try {
      statusMessage = "Saving categories to Steam sharedconfig.vdf...";
      const savedPath = await saveToSteam(activeAccount.steam_id3, games);
      categories = await getCategories();
      statusMessage = `Saved to Steam successfully! Backup created.`;
      alert(`Categories saved to Steam:\n${savedPath}\n\nRestart Steam to view updated collections!`);
    } catch (e: any) {
      statusMessage = `Save failed: ${e.message || e}`;
      alert(`Save to Steam failed: ${e.message || e}`);
    }
  }

  async function handleEnrichHltb() {
    if (games.length === 0) return;
    statusMessage = "Enriching HowLongToBeat times...";
    let done = 0;
    const updated = [...games];
    for (let i = 0; i < updated.length; i++) {
      const g = updated[i];
      try {
        const t = await fetchHltb(g.app_id);
        updated[i] = { ...g, hltb_main_story: t.main_story, hltb_main_extras: t.main_extras, hltb_completionist: t.completionist };
        saveHltbTimes(g.app_id, t.main_story, t.main_extras, t.completionist).catch(() => {});
      } catch (e) {
      }
      done++;
      if (done % 5 === 0 || done === updated.length) { statusMessage = "Enriching HLTB... /"; }
    }
    games = updated;
    statusMessage = "HLTB enrichment complete ( games).";
  }

  async function handleRunAutoCat() {
    isRunningAutoCat = true;
    autoCatProcessed = 0;
    autoCatTotal = games.length;
    autoCatCurrentGame = "";
    statusMessage = "Executing AutoCat rules on all owned games...";
    try {
      await saveAutoCatRules(autoCatRules);
      const config: AutoCatConfig = {
        rules: autoCatRules,
        clear_existing: autoCatClearExisting,
        prefix: autoCatPrefix ? autoCatPrefix.trim() : undefined,
      };
      games = await runAutoCat(config);
      categories = await getCategories();
      statusMessage = `AutoCat completed! ${games.length} games processed.`;
      showAutoCatModal = false;
    } catch (e: any) {
      statusMessage = `AutoCat error: ${e.message || e}`;
      alert(`AutoCat error: ${e.message || e}`);
    } finally {
      isRunningAutoCat = false;
      autoCatCurrentGame = "";
    }
  }

  function handleCreateCategory() {
    const trimmed = newCatName.trim();
    if (!trimmed) return;
    if (!categories.some((c) => c.name.toLowerCase() === trimmed.toLowerCase())) {
      categories.push({ name: trimmed, count: 0, is_favorite: false });
    }
    newCatName = "";
    showNewCatModal = false;
  }

  function handleDeleteCategory(catName: string, event: MouseEvent) {
    event.stopPropagation();
    if (!confirm(`Delete category "${catName}" and unassign it from all games?`)) return;
    for (const g of games) {
      g.categories = g.categories.filter((c) => c !== catName);
    }
    categories = categories.filter((c) => c.name !== catName);
    if (selectedCategory === catName) {
      selectedCategory = null;
    }
    refreshCategoryCounts();
  }

  function toggleGameSelection(appId: number) {
    if (selectedAppIds.includes(appId)) {
      selectedAppIds = selectedAppIds.filter((id) => id !== appId);
    } else {
      selectedAppIds = [...selectedAppIds, appId];
    }
  }

  function selectAllFiltered() {
    selectedAppIds = filteredGames.map((g) => g.app_id);
  }

  function clearSelection() {
    selectedAppIds = [];
  }

  function applyBatchCategory() {
    if (!batchAssignCatName) return;
    for (const g of games) {
      if (selectedAppIds.includes(g.app_id)) {
        if (!g.categories.includes(batchAssignCatName)) {
          g.categories.push(batchAssignCatName);
        }
      }
    }
    batchAssignCatName = "";
    refreshCategoryCounts();
  }

  function toggleFavoriteSelected() {
    for (const g of games) {
      if (selectedAppIds.includes(g.app_id)) {
        g.is_favorite = !g.is_favorite;
      }
    }
  }

  function toggleHideSelected() {
    for (const g of games) {
      if (selectedAppIds.includes(g.app_id)) {
        g.is_hidden = !g.is_hidden;
      }
    }
  }

  function removeCategoriesFromSelected() {
    for (const g of games) {
      if (selectedAppIds.includes(g.app_id)) {
        g.categories = [];
      }
    }
    refreshCategoryCounts();
  }

  function refreshCategoryCounts() {
    const counts: Record<string, number> = {};
    for (const g of games) {
      for (const c of g.categories) {
        counts[c] = (counts[c] || 0) + 1;
      }
    }
    categories = Object.entries(counts).map(([name, count]) => ({
      name,
      count,
      is_favorite: false,
    }));
  }

  function buildRuleParams(): Record<string, unknown> | null {
    const text = newRuleParamVal.trim();
    const min = Number(newRuleParamVal);
    const max = newRuleParamMax.trim() === "" ? undefined : Number(newRuleParamMax);
    const range = (minKey: string, maxKey: string) =>
      text && !Number.isNaN(min)
        ? { [minKey]: min, ...(max !== undefined ? { [maxKey]: max } : {}) }
        : null;
    switch (newRuleType) {
      case "Genre":
        return { genre: text };
      case "Tags":
        return { tag: text };
      case "DevPub":
        return { name: text };
      case "Platform":
        return { platform: text };
      case "Score":
        return text && !Number.isNaN(min)
          ? { min_score: min, ...(max !== undefined ? { max_score: max } : {}) }
          : { min_score: 0, max_score: 100 };
      case "Year":
        return range("min_year", "max_year") || { min_year: 1970, max_year: 2100 };
      case "Playtime":
        return range("min_hours", "max_hours") || { min_hours: 0, max_hours: 100000 };
      case "Hltb": {
        const r = range("min_hours", "max_hours") || { min_hours: 0, max_hours: 100000 };
        return { field: newRuleHltbField, ...r };
      }
      case "VrSupport":
        return { vr_only: newRuleVrOnly };
      case "Regex":
        return text ? { pattern: text, field: newRuleRegexField } : null;
      default:
        return {};
    }
  }

  function addAutoCatRule() {
    let target = "";
    if (newRuleTargetSelect === "__dynamic__") {
      target = ""; // Empty string triggers dynamic auto-naming in Rust autocat engine!
    } else if (newRuleTargetSelect === "__custom__") {
      target = newRuleTargetCustom.trim();
    } else {
      target = newRuleTargetSelect;
    }

    const params = buildRuleParams();
    if (!params) return;

    let ruleName = newRuleName.trim();
    if (!ruleName) {
      // Auto-provision descriptive name if blank
      if (newRuleType === "Regex") {
        ruleName = `Regex: ${newRuleParamVal.trim()}`;
      } else if (newRuleType === "Genre") {
        ruleName = newRuleParamVal.trim() ? `Genre: ${newRuleParamVal.trim()}` : "All Genres (Dynamic)";
      } else if (newRuleType === "Tags") {
        ruleName = newRuleParamVal.trim() ? `Tag: ${newRuleParamVal.trim()}` : "All Tags (Dynamic)";
      } else if (newRuleType === "Platform") {
        ruleName = newRuleParamVal.trim() ? `Platform: ${newRuleParamVal.trim()}` : "Platforms (Dynamic)";
      } else if (newRuleType === "DevPub") {
        ruleName = newRuleParamVal.trim() ? `Dev/Pub: ${newRuleParamVal.trim()}` : "Developers (Dynamic)";
      } else if (newRuleType === "VrSupport") {
        ruleName = newRuleVrOnly ? "VR Only" : "VR Supported";
      } else if (newRuleType === "Year") {
        ruleName = newRuleParamVal.trim() ? `Year Range` : "All Release Years (Dynamic)";
      } else {
        ruleName = `${newRuleType} Filter`;
      }
    }

    const rule: AutoCatRule = {
      id: Date.now().toString(),
      name: ruleName,
      filter_type: newRuleType,
      target_category: target,
      params,
      enabled: true,
    };
    autoCatRules.push(rule);
    newRuleName = "";
    newRuleTargetSelect = "__dynamic__";
    newRuleTargetCustom = "";
    newRuleParamVal = "";
    newRuleParamMax = "";
  }

  function deleteAutoCatRule(ruleId: string) {
    autoCatRules = autoCatRules.filter((r) => r.id !== ruleId);
  }

  function moveRuleUp(index: number) {
    if (index <= 0) return;
    const temp = autoCatRules[index];
    autoCatRules[index] = autoCatRules[index - 1];
    autoCatRules[index - 1] = temp;
  }

  function moveRuleDown(index: number) {
    if (index >= autoCatRules.length - 1) return;
    const temp = autoCatRules[index];
    autoCatRules[index] = autoCatRules[index + 1];
    autoCatRules[index + 1] = temp;
  }

  function toggleAllRules(enabled: boolean) {
    for (const r of autoCatRules) {
      r.enabled = enabled;
    }
  }

  function clearAllRules() {
    if (!confirm("Clear all AutoCat rules from Studio?")) return;
    autoCatRules = [];
  }

  function loadGenrePresets() {
    const allGenres = new Set<string>();
    for (const g of games) {
      for (const gen of g.genres) {
        if (gen.trim()) allGenres.add(gen.trim());
      }
    }
    const defaultGenres = ["Action", "Adventure", "RPG", "Strategy", "Simulation", "Indie", "Shooter", "Puzzle", "Racing", "Sports"];
    for (const dg of defaultGenres) allGenres.add(dg);

    for (const gen of Array.from(allGenres).sort()) {
      if (!autoCatRules.some((r) => r.filter_type === "Genre" && r.params.genre === gen)) {
        autoCatRules.push({
          id: crypto.randomUUID(),
          name: `Genre: ${gen}`,
          enabled: true,
          filter_type: "Genre",
          params: { genre: gen },
          target_category: gen,
        });
      }
    }
  }

  function loadHltbPresets() {
    const presets = [
      { name: "HLTB: Quick (< 5 hrs)", min: 0, max: 5, cat: "HLTB: Quick (< 5 hrs)" },
      { name: "HLTB: Short (5–10 hrs)", min: 5, max: 10, cat: "HLTB: Short (5–10 hrs)" },
      { name: "HLTB: Medium (10–25 hrs)", min: 10, max: 25, cat: "HLTB: Medium (10–25 hrs)" },
      { name: "HLTB: Long (25–50 hrs)", min: 25, max: 50, cat: "HLTB: Long (25–50 hrs)" },
      { name: "HLTB: Epic (50+ hrs)", min: 50, max: 10000, cat: "HLTB: Epic (50+ hrs)" },
    ];
    for (const p of presets) {
      if (!autoCatRules.some((r) => r.name === p.name)) {
        autoCatRules.push({
          id: crypto.randomUUID(),
          name: p.name,
          enabled: true,
          filter_type: "Hltb",
          params: { field: "main_story", min_hours: p.min, max_hours: p.max },
          target_category: p.cat,
        });
      }
    }
  }

  function loadYearPresets() {
    const currentYear = new Date().getFullYear();
    const presets = [
      { name: `Year: ${currentYear}`, min: currentYear, max: currentYear, cat: `${currentYear}` },
      { name: `Year: ${currentYear - 1}`, min: currentYear - 1, max: currentYear - 1, cat: `${currentYear - 1}` },
      { name: "Decade: 2020s", min: 2020, max: currentYear, cat: "2020s" },
      { name: "Decade: 2010s", min: 2010, max: 2019, cat: "2010s" },
      { name: "Decade: 2000s", min: 2000, max: 2009, cat: "2000s" },
      { name: "Classic: Retro (< 2000)", min: 1970, max: 1999, cat: "Retro (< 2000)" },
    ];
    for (const p of presets) {
      if (!autoCatRules.some((r) => r.name === p.name)) {
        autoCatRules.push({
          id: crypto.randomUUID(),
          name: p.name,
          enabled: true,
          filter_type: "Year",
          params: { min_year: p.min, max_year: p.max },
          target_category: p.cat,
        });
      }
    }
  }

  function loadScorePresets() {
    const presets = [
      { name: "Score: Overwhelmingly Positive (90%+)", min: 90, cat: "Overwhelmingly Positive (90%+)" },
      { name: "Score: Very Positive (80–89%)", min: 80, cat: "Very Positive (80–89%)" },
      { name: "Score: Positive (70–79%)", min: 70, cat: "Positive (70–79%)" },
    ];
    for (const p of presets) {
      if (!autoCatRules.some((r) => r.name === p.name)) {
        autoCatRules.push({
          id: crypto.randomUUID(),
          name: p.name,
          enabled: true,
          filter_type: "Score",
          params: { min_score: p.min },
          target_category: p.cat,
        });
      }
    }
  }

  function loadVrPresets() {
    const presets = [
      { name: "VR: Exclusive (VR Only)", vr_only: true, cat: "VR Only" },
      { name: "VR: Supported", vr_only: false, cat: "VR Supported" },
    ];
    for (const p of presets) {
      if (!autoCatRules.some((r) => r.name === p.name)) {
        autoCatRules.push({
          id: crypto.randomUUID(),
          name: p.name,
          enabled: true,
          filter_type: "VrSupport",
          params: { vr_only: p.vr_only },
          target_category: p.cat,
        });
      }
    }
  }

  function loadTagPresets() {
    const tagCounts: Record<string, number> = {};
    for (const g of games) {
      for (const t of g.tags) {
        if (t.trim()) tagCounts[t.trim()] = (tagCounts[t.trim()] || 0) + 1;
      }
    }
    const topTags = Object.entries(tagCounts)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 20)
      .map(([t]) => t);

    for (const tag of topTags) {
      if (!autoCatRules.some((r) => r.filter_type === "Tags" && r.params.tag === tag)) {
        autoCatRules.push({
          id: crypto.randomUUID(),
          name: `Tag: ${tag}`,
          enabled: true,
          filter_type: "Tags",
          params: { tag },
          target_category: tag,
        });
      }
    }
  }

  function loadPlatformPresets() {
    const presets = [
      { name: "Platform: Windows", platform: "Windows", cat: "Windows" },
      { name: "Platform: Linux / SteamOS", platform: "Linux", cat: "Linux" },
      { name: "Platform: macOS", platform: "Mac", cat: "macOS" },
    ];
    for (const p of presets) {
      if (!autoCatRules.some((r) => r.name === p.name)) {
        autoCatRules.push({
          id: crypto.randomUUID(),
          name: p.name,
          enabled: true,
          filter_type: "Platform",
          params: { platform: p.platform },
          target_category: p.cat,
        });
      }
    }
  }

  function loadDeveloperPresets() {
    const devCounts: Record<string, number> = {};
    for (const g of games) {
      if (g.developer?.trim()) devCounts[g.developer.trim()] = (devCounts[g.developer.trim()] || 0) + 1;
    }
    const topDevs = Object.entries(devCounts)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 15)
      .map(([d]) => d);

    for (const dev of topDevs) {
      if (!autoCatRules.some((r) => r.filter_type === "DevPub" && r.params.name === dev)) {
        autoCatRules.push({
          id: crypto.randomUUID(),
          name: `Developer: ${dev}`,
          enabled: true,
          filter_type: "DevPub",
          params: { name: dev },
          target_category: dev,
        });
      }
    }
  }

  function loadRegexAlphabeticalPresets() {
    const rule: AutoCatRule = {
      id: crypto.randomUUID(),
      name: "Title: Alphabetical A–Z",
      enabled: true,
      filter_type: "Regex",
      params: { pattern: "^([A-Z])", field: "name" },
      target_category: "", // Dynamic!
    };
    if (!autoCatRules.some((r) => r.name === rule.name)) {
      autoCatRules.push(rule);
    }
  }

  function loadStandardDefaultPresetSet() {
    loadGenrePresets();
    loadYearPresets();
    loadScorePresets();
    loadHltbPresets();
    loadVrPresets();
    loadPlatformPresets();
  }

  function gameMatchesRule(game: SteamGame, rule: AutoCatRule): boolean {
    switch (rule.filter_type) {
      case "Genre": {
        const target = String(rule.params.genre || "").toLowerCase();
        return game.genres.some((gen) => gen.toLowerCase().includes(target));
      }
      case "Tags": {
        const target = String(rule.params.tag || "").toLowerCase();
        return game.tags.some((t) => t.toLowerCase().includes(target));
      }
      case "DevPub": {
        const target = String(rule.params.name || "").toLowerCase();
        return Boolean(
          (game.developer && game.developer.toLowerCase().includes(target)) ||
          (game.publisher && game.publisher.toLowerCase().includes(target))
        );
      }
      case "Year": {
        const min = Number(rule.params.min_year ?? 1970);
        const max = Number(rule.params.max_year ?? 2100);
        return game.release_year != null && game.release_year >= min && game.release_year <= max;
      }
      case "Playtime": {
        const min = Number(rule.params.min_hours ?? 0);
        const max = Number(rule.params.max_hours ?? 100000);
        const hours = (game.playtime_minutes || 0) / 60;
        return hours >= min && hours <= max;
      }
      case "Hltb": {
        const min = Number(rule.params.min_hours ?? 0);
        const max = Number(rule.params.max_hours ?? 100000);
        const field = String(rule.params.field || "main_story");
        const val =
          field === "main_extras"
            ? game.hltb_main_extras
            : field === "completionist"
            ? game.hltb_completionist
            : game.hltb_main_story;
        return val != null && val >= min && val <= max;
      }
      case "Score": {
        const min = Number(rule.params.min_score ?? 0);
        return game.review_score != null && game.review_score >= min;
      }
      case "Platform": {
        const target = String(rule.params.platform || "").toLowerCase();
        return game.platforms.some((p) => p.toLowerCase().includes(target));
      }
      case "VrSupport": {
        const vrOnly = Boolean(rule.params.vr_only);
        return vrOnly ? game.vr_support === "VR Only" : Boolean(game.vr_support?.includes("VR"));
      }
      case "Regex": {
        const pattern = String(rule.params.pattern || "").trim();
        const field = String(rule.params.field || "name");
        if (!pattern) return false;
        try {
          const re = new RegExp(pattern, "i");
          const targetText =
            field === "developer"
              ? (game.developer || "")
              : field === "publisher"
              ? (game.publisher || "")
              : game.name;
          return re.test(targetText);
        } catch {
          return false;
        }
      }
      default:
        return false;
    }
  }

  function countRuleMatches(rule: AutoCatRule): number {
    let count = 0;
    for (const g of games) {
      if (gameMatchesRule(g, rule)) count++;
    }
    return count;
  }

  async function runQuickCategory(
    mode: "genre" | "tags" | "year" | "hltb" | "score" | "vr" | "dev" | "platform"
  ) {
    if (games.length === 0) return;
    statusMessage = `Auto-categorizing all ${games.length} games by ${mode}...`;
    let rulesToRun: AutoCatRule[] = [];

    if (mode === "genre") {
      const allGenres = new Set<string>();
      for (const g of games) {
        for (const gen of g.genres) {
          if (gen.trim()) allGenres.add(gen.trim());
        }
      }
      rulesToRun = Array.from(allGenres).sort().map((gen) => ({
        id: crypto.randomUUID(),
        name: `Genre: ${gen}`,
        enabled: true,
        filter_type: "Genre",
        params: { genre: gen },
        target_category: gen,
      }));
    } else if (mode === "tags") {
      const tagCounts: Record<string, number> = {};
      for (const g of games) {
        for (const t of g.tags) {
          tagCounts[t] = (tagCounts[t] || 0) + 1;
        }
      }
      const topTags = Object.entries(tagCounts)
        .sort((a, b) => b[1] - a[1])
        .slice(0, 20)
        .map(([t]) => t);
      rulesToRun = topTags.map((tag) => ({
        id: crypto.randomUUID(),
        name: `Tag: ${tag}`,
        enabled: true,
        filter_type: "Tags",
        params: { tag },
        target_category: tag,
      }));
    } else if (mode === "year") {
      const currentYear = new Date().getFullYear();
      rulesToRun = [
        { id: crypto.randomUUID(), name: `${currentYear}`, enabled: true, filter_type: "Year", params: { min_year: currentYear, max_year: currentYear }, target_category: `${currentYear}` },
        { id: crypto.randomUUID(), name: `${currentYear - 1}`, enabled: true, filter_type: "Year", params: { min_year: currentYear - 1, max_year: currentYear - 1 }, target_category: `${currentYear - 1}` },
        { id: crypto.randomUUID(), name: "2020s", enabled: true, filter_type: "Year", params: { min_year: 2020, max_year: currentYear }, target_category: "2020s" },
        { id: crypto.randomUUID(), name: "2010s", enabled: true, filter_type: "Year", params: { min_year: 2010, max_year: 2019 }, target_category: "2010s" },
        { id: crypto.randomUUID(), name: "2000s", enabled: true, filter_type: "Year", params: { min_year: 2000, max_year: 2009 }, target_category: "2000s" },
        { id: crypto.randomUUID(), name: "Retro (< 2000)", enabled: true, filter_type: "Year", params: { min_year: 1970, max_year: 1999 }, target_category: "Retro (< 2000)" },
      ];
    } else if (mode === "hltb") {
      rulesToRun = [
        { id: crypto.randomUUID(), name: "HLTB: 0–5 hrs", enabled: true, filter_type: "Hltb", params: { field: "main_story", min_hours: 0, max_hours: 5 }, target_category: "HLTB: 0–5 hrs" },
        { id: crypto.randomUUID(), name: "HLTB: 5–10 hrs", enabled: true, filter_type: "Hltb", params: { field: "main_story", min_hours: 5, max_hours: 10 }, target_category: "HLTB: 5–10 hrs" },
        { id: crypto.randomUUID(), name: "HLTB: 10–20 hrs", enabled: true, filter_type: "Hltb", params: { field: "main_story", min_hours: 10, max_hours: 20 }, target_category: "HLTB: 10–20 hrs" },
        { id: crypto.randomUUID(), name: "HLTB: 20–50 hrs", enabled: true, filter_type: "Hltb", params: { field: "main_story", min_hours: 20, max_hours: 50 }, target_category: "HLTB: 20–50 hrs" },
        { id: crypto.randomUUID(), name: "HLTB: 50+ hrs", enabled: true, filter_type: "Hltb", params: { field: "main_story", min_hours: 50, max_hours: 100000 }, target_category: "HLTB: 50+ hrs" },
      ];
    } else if (mode === "score") {
      rulesToRun = [
        { id: crypto.randomUUID(), name: "Overwhelmingly Positive (90%+)", enabled: true, filter_type: "Score", params: { min_score: 90 }, target_category: "Overwhelmingly Positive (90%+)" },
        { id: crypto.randomUUID(), name: "Very Positive (80–89%)", enabled: true, filter_type: "Score", params: { min_score: 80 }, target_category: "Very Positive (80–89%)" },
        { id: crypto.randomUUID(), name: "Positive (70–79%)", enabled: true, filter_type: "Score", params: { min_score: 70 }, target_category: "Positive (70–79%)" },
        { id: crypto.randomUUID(), name: "Mixed (40–69%)", enabled: true, filter_type: "Score", params: { min_score: 40 }, target_category: "Mixed (40–69%)" },
      ];
    } else if (mode === "vr") {
      rulesToRun = [
        { id: crypto.randomUUID(), name: "VR Only", enabled: true, filter_type: "VrSupport", params: { vr_only: true }, target_category: "VR Only" },
        { id: crypto.randomUUID(), name: "VR Supported", enabled: true, filter_type: "VrSupport", params: { vr_only: false }, target_category: "VR Supported" },
      ];
    } else if (mode === "dev") {
      const devs = new Set<string>();
      for (const g of games) {
        if (g.developer?.trim()) devs.add(g.developer.trim());
      }
      rulesToRun = Array.from(devs).slice(0, 30).map((dev) => ({
        id: crypto.randomUUID(),
        name: `Dev: ${dev}`,
        enabled: true,
        filter_type: "DevPub",
        params: { name: dev },
        target_category: `Dev: ${dev}`,
      }));
    } else if (mode === "platform") {
      rulesToRun = [
        { id: crypto.randomUUID(), name: "Windows", enabled: true, filter_type: "Platform", params: { platform: "windows" }, target_category: "Windows" },
        { id: crypto.randomUUID(), name: "macOS", enabled: true, filter_type: "Platform", params: { platform: "macos" }, target_category: "macOS" },
        { id: crypto.randomUUID(), name: "Linux / Steam Deck", enabled: true, filter_type: "Platform", params: { platform: "linux" }, target_category: "Linux / Steam Deck" },
      ];
    }

    try {
      isRunningAutoCat = true;
      autoCatProcessed = 0;
      autoCatTotal = games.length;
      const config: AutoCatConfig = {
        rules: rulesToRun,
        clear_existing: false,
      };

      if (autoCatSelectedOnly && selectedAppIds.length > 0) {
        const selectedSet = new Set(selectedAppIds);
        const processed = await runAutoCat(config);
        const processedMap = new Map(processed.map((g) => [g.app_id, g]));
        games = games.map((g) => (selectedSet.has(g.app_id) ? processedMap.get(g.app_id) || g : g));
      } else {
        games = await runAutoCat(config);
      }
      refreshCategoryCounts();
      statusMessage = `Quick Auto-Categorize complete! Applied ${rulesToRun.length} categories.`;
    } catch (e: any) {
      statusMessage = `Quick Auto-Categorize error: ${e.message || e}`;
    } finally {
      isRunningAutoCat = false;
    }
  }

  async function handleApplySelectedFilter() {
    if (games.length === 0) return;

    if (selectedAutoCatFilter.startsWith("rule:")) {
      const ruleId = selectedAutoCatFilter.replace("rule:", "");
      const rule = autoCatRules.find((r) => r.id === ruleId);
      if (!rule) return;
      statusMessage = `Applying filter "${rule.name}"...`;
      try {
        isRunningAutoCat = true;
        autoCatProcessed = 0;
        autoCatTotal = games.length;
        const config: AutoCatConfig = {
          rules: [rule],
          clear_existing: false,
          prefix: autoCatPrefix ? autoCatPrefix.trim() : undefined,
        };

        if (autoCatSelectedOnly && selectedAppIds.length > 0) {
          const selectedSet = new Set(selectedAppIds);
          const processed = await runAutoCat(config);
          const processedMap = new Map(processed.map((g) => [g.app_id, g]));
          games = games.map((g) => (selectedSet.has(g.app_id) ? processedMap.get(g.app_id) || g : g));
        } else {
          games = await runAutoCat(config);
        }
        refreshCategoryCounts();
        statusMessage = `Applied filter "${rule.name}" successfully!`;
      } catch (e: any) {
        statusMessage = `Error applying filter: ${e.message || e}`;
      } finally {
        isRunningAutoCat = false;
      }
    } else if (selectedAutoCatFilter === "__all_rules__") {
      await handleRunAutoCat();
    } else {
      await runQuickCategory(selectedAutoCatFilter as any);
    }
  }

  // Reactive Derivations
  let uncategorizedCount = $derived(games.filter((g) => g.categories.length === 0).length);
  let favoritesCount = $derived(games.filter((g) => g.is_favorite).length);
  let hiddenCount = $derived(games.filter((g) => g.is_hidden).length);

  let filteredGames = $derived(
    games.filter((g) => {
      // Search Query
      if (searchQuery.trim()) {
        const q = searchQuery.trim().toLowerCase();
        const matchesName = g.name.toLowerCase().includes(q);
        const matchesId = g.app_id.toString().includes(q);
        const matchesCat = g.categories.some((c) => c.toLowerCase().includes(q));
        if (!matchesName && !matchesId && !matchesCat) return false;
      }

      // Quick Installed Filter
      if (quickFilter === "installed" && g.playtime_minutes === 0 && g.last_played === 0) {
        // approximate heuristic if local
      }

      // Category Filter
      if (selectedCategory === "__uncategorized__") {
        return g.categories.length === 0;
      } else if (selectedCategory === "__favorites__") {
        return g.is_favorite;
      } else if (selectedCategory === "__hidden__") {
        return g.is_hidden;
      } else if (selectedCategory) {
        return g.categories.includes(selectedCategory);
      }

      return true;
    })
  );

  function formatPlaytime(mins: number): string {
    if (mins <= 0) return "0.0 hrs";
    return `${(mins / 60).toFixed(1)} hrs`;
  }

  function formatLastPlayed(timestamp: number): string {
    if (!timestamp || timestamp <= 0) return "Never";
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString();
  }

  function getSteamIconUrl(appId: number, hash?: string | null): string {
    if (!hash) return "/favicon.svg";
    return `https://media.steampowered.com/steamcommunity/public/images/apps/${appId}/${hash}.jpg`;
  }
</script>

<div class="depressurizer-layout">
  <!-- Top High-Density Utility Toolbar -->
  <header class="depressurizer-toolbar">
    <div class="toolbar-left">
      <div class="user-select-group">
        <label for="steam-user">Steam User:</label>
        <select
          id="steam-user"
          value={activeAccount?.steam_id3 || ""}
          onchange={(e) => handleAccountChange((e.target as HTMLSelectElement).value)}
        >
          {#each accounts as acc (acc.steam_id3)}
            <option value={acc.steam_id3}>
              {acc.persona_name} ({acc.account_name})
            </option>
          {/each}
        </select>
      </div>

      <div class="status-pill" title="Cached in native SQLite database">
        <span class="dot"></span>
        <span>{games.length} Owned Games</span>
      </div>
    </div>

    <div class="search-box">
      <span class="search-icon">🔍</span>
      <input
        type="text"
        class="input-text"
        placeholder="Filter games (name, ID, tag)..."
        bind:value={searchQuery}
      />
    </div>

    <div class="toolbar-right">
      <button class="btn btn-accent btn-sm" onclick={() => (showAutoCatModal = true)}>
        ⚡ AutoCat Studio
      </button>

      <button class="btn btn-primary btn-sm" onclick={handleSaveToSteam}>
        Save to Steam
      </button>

      <button class="btn btn-secondary btn-sm" onclick={handleEnrichHltb}>
        Enrich HLTB
      </button>

      <button class="btn btn-secondary btn-sm" onclick={() => (showSettingsModal = true)}>
        ⚙️ Settings
      </button>
    </div>
  </header>

  <!-- Main Split Workbench -->
  <div class="depressurizer-body">
    <!-- Left Panel: Categories & Profiles -->
    <aside class="sidebar-panel">
      <div class="sidebar-header">
        <span class="sidebar-title">Categories</span>
        <button class="btn btn-secondary btn-sm" onclick={() => (showNewCatModal = true)}>
          + New
        </button>
      </div>

      <div class="sidebar-scroll">
        <ul class="category-list" style="list-style:none;padding:0;margin:0;">
          <li class="category-group-label">System Filters</li>

          <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
          <li
            role="button"
            tabindex="0"
            class="category-item"
            class:active={selectedCategory === null}
            onclick={() => (selectedCategory = null)}
            onkeydown={(e) => e.key === "Enter" && (selectedCategory = null)}
          >
            <div class="item-left">
              <span>All Owned Games</span>
            </div>
            <span class="item-count">{games.length}</span>
          </li>

          <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
          <li
            role="button"
            tabindex="0"
            class="category-item"
            class:active={selectedCategory === "__uncategorized__"}
            onclick={() => (selectedCategory = "__uncategorized__")}
            onkeydown={(e) => e.key === "Enter" && (selectedCategory = "__uncategorized__")}
          >
            <div class="item-left">
              <span>Uncategorized</span>
            </div>
            <span class="item-count">{uncategorizedCount}</span>
          </li>

          <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
          <li
            role="button"
            tabindex="0"
            class="category-item"
            class:active={selectedCategory === "__favorites__"}
            onclick={() => (selectedCategory = "__favorites__")}
            onkeydown={(e) => e.key === "Enter" && (selectedCategory = "__favorites__")}
          >
            <div class="item-left">
              <span>Favorites</span>
            </div>
            <span class="item-count">{favoritesCount}</span>
          </li>

          <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
          <li
            role="button"
            tabindex="0"
            class="category-item"
            class:active={selectedCategory === "__hidden__"}
            onclick={() => (selectedCategory = "__hidden__")}
            onkeydown={(e) => e.key === "Enter" && (selectedCategory = "__hidden__")}
          >
            <div class="item-left">
              <span>Hidden Games</span>
            </div>
            <span class="item-count">{hiddenCount}</span>
          </li>

          <li class="category-group-label" style="margin-top: 10px;">
            Collections ({categories.length})
          </li>

          {#each categories as cat (cat.name)}
            <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
            <li
              role="button"
              tabindex="0"
              class="category-item"
              class:active={selectedCategory === cat.name}
              onclick={() => (selectedCategory = cat.name)}
              onkeydown={(e) => e.key === "Enter" && (selectedCategory = cat.name)}
            >
              <div class="item-left">
                <span title={cat.name}>{cat.name}</span>
              </div>
              <div style="display:flex;align-items:center;gap:4px;">
                <span class="item-count">{cat.count}</span>
                <div class="item-actions">
                  <button
                    type="button"
                    title="Delete category"
                    onclick={(e) => handleDeleteCategory(cat.name, e)}
                  >
                    x
                  </button>
                </div>
              </div>
            </li>
          {/each}
        </ul>
      </div>    </aside>

    <!-- Right Panel: Dense Games Data Table -->
    <main class="content-panel">
      <div class="table-header-bar">
        <div class="header-left">
          <span class="view-title">
            {#if selectedCategory === null}
              All Owned Games ({filteredGames.length})
            {:else if selectedCategory === "__uncategorized__"}
              Uncategorized Games ({filteredGames.length})
            {:else if selectedCategory === "__favorites__"}
              Favorite Games ({filteredGames.length})
            {:else if selectedCategory === "__hidden__"}
              Hidden Games ({filteredGames.length})
            {:else}
              Category: {selectedCategory} ({filteredGames.length})
            {/if}
          </span>
        </div>

        <div class="header-right">
          <button class="btn btn-secondary btn-sm" onclick={selectAllFiltered}>
            Select All
          </button>
          {#if selectedAppIds.length > 0}
            <button class="btn btn-ghost btn-sm" onclick={clearSelection}>
              Deselect ({selectedAppIds.length})
            </button>
          {/if}
        </div>
      </div>

      <!-- High-Density Data Table (Exact Legacy Depressurizer Columns) -->
      <div class="table-container">
        <table class="game-table">
          <thead>
            <tr>
              <th style="width: 32px; text-align: center;">
                <input
                  type="checkbox"
                  checked={selectedAppIds.length === filteredGames.length && filteredGames.length > 0}
                  onchange={(e) => {
                    if ((e.target as HTMLInputElement).checked) {
                      selectAllFiltered();
                    } else {
                      clearSelection();
                    }
                  }}
                />
              </th>
              <th style="width: 70px;">AppID</th>
              <th>Game Title</th>
              <th>Categories & Collections</th>
              <th style="width: 90px;">Playtime</th>
              <th style="width: 110px;">HLTB (Main/Extras/100%)</th>
              <th style="width: 100px;">Last Played</th>
              <th style="width: 50px; text-align: center;">Flags</th>
            </tr>
          </thead>
          <tbody>
            {#if filteredGames.length === 0}
              <tr>
                <td colspan="8" style="text-align: center; padding: 40px; color: var(--text-dim);">
                  No games match the current filter or category.
                </td>
              </tr>
            {:else}
              {#each filteredGames as game (game.app_id)}
                {@const isSelected = selectedAppIds.includes(game.app_id)}
                <tr class:selected={isSelected} onclick={() => toggleGameSelection(game.app_id)}>
                  <td style="text-align: center;" onclick={(e) => e.stopPropagation()}>
                    <input
                      type="checkbox"
                      checked={isSelected}
                      onchange={() => toggleGameSelection(game.app_id)}
                    />
                  </td>
                  <td style="font-family: monospace; color: var(--text-dim);">
                    {game.app_id}
                  </td>
                  <td class="col-title" title={game.name}>
                    <div style="display:flex;align-items:center;gap:8px;">
                      <img
                        src={getSteamIconUrl(game.app_id, game.icon_hash)}
                        alt=""
                        width="16"
                        height="16"
                        style="border-radius:2px;flex-shrink:0;"
                        onerror={(e) => ((e.currentTarget as HTMLImageElement).src = "/favicon.svg")}
                      />
                      <span>{game.name}</span>
                    </div>
                  </td>
                  <td class="col-categories">
                    <div class="table-cats">
                      {#each game.categories as cat}
                        <span class="cat-pill">{cat}</span>
                      {/each}
                    </div>
                  </td>
                  <td>{formatPlaytime(game.playtime_minutes)}</td>
                  <td style="font-size:11px;color:var(--text-dim);">{game.hltb_main_story != null ? game.hltb_main_story.toFixed(1) : '—'} / {game.hltb_main_extras != null ? game.hltb_main_extras.toFixed(1) : '—'} / {game.hltb_completionist != null ? game.hltb_completionist.toFixed(1) : '—'}</td>
                  <td>{formatLastPlayed(game.last_played)}</td>
                  <td style="text-align: center;">
                    {#if game.is_favorite}
                      <span title="Favorite" style="color: var(--accent-gold);">★</span>
                    {/if}
                    {#if game.is_hidden}
                      <span title="Hidden" style="color: var(--text-dim); margin-left: 4px;">👁️</span>
                    {/if}
                  </td>
                </tr>
              {/each}
            {/if}
          </tbody>
        </table>
      </div>

      <!-- Batch Assignment Drawer for Selected Games -->
      {#if selectedAppIds.length > 0}
        <div class="batch-drawer">
          <div class="drawer-left">
            <span class="selection-label">
              ✓ {selectedAppIds.length} {selectedAppIds.length === 1 ? "game" : "games"} selected
            </span>
            <select class="select-control" bind:value={batchAssignCatName}>
              <option value="">Assign to category...</option>
              {#each categories as cat}
                <option value={cat.name}>{cat.name}</option>
              {/each}
            </select>
            <button
              class="btn btn-secondary btn-sm"
              disabled={!batchAssignCatName}
              onclick={applyBatchCategory}
            >
              + Assign
            </button>
          </div>

          <div class="drawer-right">
            <button class="btn btn-secondary btn-sm" onclick={toggleFavoriteSelected}>
              ★ Toggle Favorite
            </button>
            <button class="btn btn-secondary btn-sm" onclick={toggleHideSelected}>
              👁️ Toggle Hidden
            </button>
            <button class="btn btn-danger btn-sm" onclick={removeCategoriesFromSelected}>
              ✕ Clear Categories
            </button>
          </div>
        </div>
      {/if}
    </main>
  </div>

  <!-- Bottom Action Dock with Categorization Filter Dropdown & Controls -->
  <div class="bottom-action-bar">
    <div class="bar-left" style="display: flex; align-items: center; gap: 8px;">
      <label for="autocat-filter-select" style="font-size: 11.5px; font-weight: 700; color: var(--accent-cyan); display: flex; align-items: center; gap: 4px; white-space: nowrap;">
        <span>⚡ Auto-Categorize:</span>
      </label>

      <select
        id="autocat-filter-select"
        class="select-control"
        bind:value={selectedAutoCatFilter}
        style="min-width: 250px; max-width: 320px;"
      >
        <optgroup label="Prebuilt Filters">
          <option value="hltb">⏱️ HowLongToBeat (Playtime)</option>
          <option value="genre">🏷️ Store Genres</option>
          <option value="tags">🔖 Top Store Tags</option>
          <option value="year">📅 Release Year & Decades</option>
          <option value="score">⭐ Steam Review Score</option>
          <option value="vr">🥽 VR Support</option>
          <option value="dev">🛠️ Developer / Publisher</option>
          <option value="platform">💻 Platform (OS)</option>
        </optgroup>
        {#if autoCatRules.length > 0}
          <optgroup label="AutoCat Studio Filters">
            <option value="__all_rules__">▶️ All Configured Rules ({autoCatRules.filter(r => r.enabled).length} active)</option>
            {#each autoCatRules as rule}
              <option value={`rule:${rule.id}`}>▸ {rule.name}</option>
            {/each}
          </optgroup>
        {/if}
      </select>

      <button
        type="button"
        class="btn btn-accent btn-sm"
        disabled={isRunningAutoCat || games.length === 0}
        onclick={handleApplySelectedFilter}
      >
        {isRunningAutoCat ? "⏳ Running…" : "Apply AutoCat"}
      </button>

      {#if selectedAppIds.length > 0}
        <label style="display: flex; align-items: center; gap: 5px; font-size: 11.5px; color: var(--text-secondary); cursor: pointer;">
          <input type="checkbox" bind:checked={autoCatSelectedOnly} />
          <span>Only selected ({selectedAppIds.length})</span>
        </label>
      {/if}

      <button
        type="button"
        class="btn btn-secondary btn-sm"
        onclick={() => (showAutoCatModal = true)}
        title="Open AutoCat Studio to make new filters or remove existing ones"
      >
        ⚙️ AutoCat Studio…
      </button>
    </div>

    <div class="bar-right">
      <button class="btn btn-secondary btn-sm" onclick={handleEnrichHltb}>
        ⏱️ Enrich HLTB
      </button>
      <button class="btn btn-primary btn-sm" onclick={handleSaveToSteam}>
        💾 Save to Steam
      </button>
    </div>
  </div>

  <!-- Bottom Global Status Bar -->
  <footer class="status-bar">
    <div class="status-left">
      <span>
        Account: <strong>{activeAccount?.persona_name || "None"}</strong> ({activeAccount?.steam_id3 || ""})
      </span>
      <span>|</span>
      <span>{statusMessage}</span>
    </div>

    <div class="status-right">
      {#if workerProgress}
        {@const pct = workerProgress.total > 0 ? Math.round((workerProgress.current / workerProgress.total) * 100) : 0}
        <div class="worker-progress-inline">
          <span>{workerProgress.message} ({pct}%)</span>
          <div class="progress-bar-track">
            <div class="progress-bar-fill" style="width: {pct}%"></div>
          </div>
        </div>
      {/if}
      <span>SQLite: depressurizer.db</span>
    </div>
  </footer>
</div>

<!-- =========================================================================== -->
<!-- AutoCat Studio Modal                                                        -->
<!-- =========================================================================== -->
{#if showAutoCatModal}
  <div
    class="modal-overlay"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => { if (e.target === e.currentTarget) showAutoCatModal = false; }}
    onkeydown={(e) => { if (e.key === "Escape") showAutoCatModal = false; }}
  >
    <div class="modal-dialog modal-lg">
      <div class="modal-header">
        <h3>⚡ AutoCat Rules Studio</h3>
        <button class="btn btn-ghost btn-sm" onclick={() => (showAutoCatModal = false)}>✕</button>
      </div>

      <div class="modal-body">
        <p style="color: var(--text-secondary); margin-top: 0; font-size: 12px;">
          Configure automatic categorization rules based on Store genres, tags, developer, release year, or score.
          Rules are saved to native SQLite and executed via Tokio background workers.
        </p>

        <!-- Presets & Bulk Controls Toolbar -->
        <div class="studio-preset-bar">
          <div style="display: flex; flex-direction: column; gap: 6px; width: 100%;">
            <div style="display: flex; align-items: center; justify-content: space-between;">
              <span class="preset-label" style="font-weight: 700; color: var(--accent-cyan);">⚡ Quick Presets:</span>
              <div style="display: flex; align-items: center; gap: 6px;">
                <button type="button" class="btn btn-ghost btn-sm" onclick={() => toggleAllRules(true)}>
                  Enable All
                </button>
                <button type="button" class="btn btn-ghost btn-sm" onclick={() => toggleAllRules(false)}>
                  Disable All
                </button>
                {#if autoCatRules.length > 0}
                  <button type="button" class="btn btn-danger btn-sm" onclick={clearAllRules}>
                    Clear All
                  </button>
                {/if}
              </div>
            </div>
            <div class="preset-buttons">
              <button type="button" class="btn btn-primary btn-sm" onclick={loadStandardDefaultPresetSet} title="Auto-provisions the full suite (Genres, Years, Scores, Top Tags, VR, HLTB, Platforms)">
                ⚡ Auto-Provision Standard Suite
              </button>
              <button type="button" class="btn btn-secondary btn-sm" onclick={loadGenrePresets}>
                + Genres
              </button>
              <button type="button" class="btn btn-secondary btn-sm" onclick={loadTagPresets}>
                + Store Tags
              </button>
              <button type="button" class="btn btn-secondary btn-sm" onclick={loadHltbPresets}>
                ⏱️ + HLTB Playtimes
              </button>
              <button type="button" class="btn btn-secondary btn-sm" onclick={loadYearPresets}>
                + Release Years
              </button>
              <button type="button" class="btn btn-secondary btn-sm" onclick={loadScorePresets}>
                + Review Scores
              </button>
              <button type="button" class="btn btn-secondary btn-sm" onclick={loadPlatformPresets}>
                + Platforms
              </button>
              <button type="button" class="btn btn-secondary btn-sm" onclick={loadDeveloperPresets}>
                + Developers
              </button>
              <button type="button" class="btn btn-secondary btn-sm" onclick={loadVrPresets}>
                + VR Support
              </button>
              <button type="button" class="btn btn-secondary btn-sm" onclick={loadRegexAlphabeticalPresets}>
                + Title A–Z (Regex)
              </button>
            </div>
          </div>
        </div>

        <!-- Rules Table -->
        {#if autoCatRules.length === 0}
          <div style="text-align: center; padding: 24px 16px; margin-bottom: 16px; background: rgba(255, 255, 255, 0.02); border: 1px dashed var(--border-subtle); border-radius: 6px; color: var(--text-dim); font-size: 12px;">
            No AutoCat rules configured yet. Click <strong>⚡ Auto-Provision Standard Suite</strong> above or build a custom filter below.
          </div>
        {:else}
          <table class="game-table" style="margin-bottom: 16px;">
            <thead>
              <tr>
                <th style="width: 45px; text-align: center;">Order</th>
                <th style="width: 40px; text-align: center;">Active</th>
                <th>Rule Name</th>
                <th>Filter By</th>
                <th>Match Criteria</th>
                <th>Target Category</th>
                <th style="width: 80px; text-align: center;">Matches</th>
                <th style="width: 40px; text-align: center;">Action</th>
              </tr>
            </thead>
            <tbody>
              {#each autoCatRules as rule, idx (rule.id)}
                {@const matchCount = countRuleMatches(rule)}
                <tr>
                  <td style="text-align: center;">
                    <div class="rule-order-buttons">
                      <button
                        type="button"
                        title="Move Up"
                        disabled={idx === 0}
                        onclick={() => moveRuleUp(idx)}
                      >
                        ▲
                      </button>
                      <button
                        type="button"
                        title="Move Down"
                        disabled={idx === autoCatRules.length - 1}
                        onclick={() => moveRuleDown(idx)}
                      >
                        ▼
                      </button>
                    </div>
                  </td>
                  <td style="text-align: center;">
                    <input type="checkbox" bind:checked={rule.enabled} />
                  </td>
                  <td style="font-weight: 600; color: var(--text-primary);">{rule.name}</td>
                  <td><span class="badge badge-purple">{rule.filter_type}</span></td>
                  <td style="font-size: 11px; color: var(--text-secondary); max-width: 220px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                    {JSON.stringify(rule.params)}
                  </td>
                  <td>
                    {#if !rule.target_category || rule.target_category === '__dynamic__'}
                      <span class="badge badge-purple">⚡ Dynamic (Auto)</span>
                    {:else}
                      <span class="badge badge-cyan">{rule.target_category}</span>
                    {/if}
                  </td>
                  <td style="text-align: center;">
                    <span class="match-count-badge">{matchCount} {matchCount === 1 ? 'game' : 'games'}</span>
                  </td>
                  <td style="text-align: center;">
                    <button
                      type="button"
                      class="btn btn-danger btn-sm"
                      style="padding: 2px 6px;"
                      title="Delete Rule"
                      onclick={() => deleteAutoCatRule(rule.id)}
                    >
                      ✕
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}

        <!-- In-App Explanatory Guide Card -->
        <div class="autocat-guide-card">
          <button type="button" class="autocat-guide-toggle" onclick={() => (showAutoCatGuide = !showAutoCatGuide)}>
            <span>📘 {showAutoCatGuide ? "Hide" : "Show"} Guide: How Custom Filters & Dynamic AutoCat Work</span>
            <span>{showAutoCatGuide ? "▲" : "▼"}</span>
          </button>
          {#if showAutoCatGuide}
            <div class="autocat-guide-content">
              <h4>⚡ Dynamic Auto-Categorization (No Manual Category Input Needed)</h4>
              <p>
                In Depressurizer, leaving the <strong>Target Category</strong> as <code>⚡ Dynamic (Auto-name from matched value)</code> instructs AutoCat
                to derive category names dynamically from the matching metadata. You do <em>not</em> need to manually create
                or type a category name for each filter:
              </p>
              <ul>
                <li><strong>Genres & Store Tags:</strong> Games are automatically assigned to categories named after their matched genres or tags (e.g. <code>RPG</code>, <code>Roguelike</code>). Leaving the value blank categorizes every game by all its genres/tags.</li>
                <li><strong>HowLongToBeat:</strong> Games with HLTB metadata are automatically placed into playtime brackets (<code>HLTB: 0–5 hrs</code>, <code>HLTB: 5–10 hrs</code>, <code>HLTB: 10–20 hrs</code>, <code>HLTB: 20–50 hrs</code>, <code>HLTB: 50+ hrs</code>).</li>
                <li><strong>Release Years & Scores:</strong> Grouped into decades (<code>2020s</code>, <code>2010s</code>) or review tiers (<code>90%+ Overwhelmingly Positive</code>).</li>
                <li><strong>Developers & Publishers:</strong> Categorized directly under studio names (e.g. <code>Valve</code>, <code>FromSoftware</code>).</li>
              </ul>

              <h4>🔍 Custom Regex Filters & Dynamic Capture Groups</h4>
              <p>
                Choose <strong>Regex / Name Pattern</strong> to filter by regular expressions. If your regex contains a capture group in parentheses <code>(...)</code>,
                the matched capture group dynamically becomes the category name!
              </p>
              <ul>
                <li><code>^([A-Z])</code> → Alphabetical library shelves (<code>A</code>, <code>B</code>, <code>C</code>, etc.).</li>
                <li><code>^([0-9])</code> → Numbered title collections.</li>
                <li><code>^([A-Za-z0-9\- ]+?)(?:\s+[IVXLCDM0-9]+|:)</code> → Automatic game franchise detection (e.g., <em>Final Fantasy</em>, <em>Half-Life</em>, <em>Resident Evil</em>).</li>
                <li><code>(?i)(remastered|remake|definitive|goty)</code> → Tags special editions and remakes.</li>
              </ul>

              <h4>🏷️ Category Prefixes & Execution Flow</h4>
              <p>
                Use the optional <strong>Category Prefix</strong> setting below (e.g. <code>Genre: </code>, <code>[HLTB] </code>) to neatly group collections together in your Steam sidebar.
                Rules run sequentially from top to bottom; you can re-order rules with ▲/▼ or toggle them on/off at any time.
              </p>
            </div>
          {/if}
        </div>

        <!-- Add New Rule Builder Card -->
        <div style="background: var(--bg-surface); padding: 12px; border-radius: 6px; border: 1px solid var(--border-subtle);">
          <div style="font-size: 11.5px; font-weight: 700; color: var(--accent-cyan); margin-bottom: 8px;">
            + Create New Custom Filter
          </div>
          <div style="display: grid; grid-template-columns: 1fr 1.1fr 1.2fr 1.3fr auto; gap: 8px; align-items: center;">
            <input
              type="text"
              class="input-text"
              placeholder="Rule Name (Optional)"
              bind:value={newRuleName}
            />
            <select class="select-control" bind:value={newRuleType}>
              <option value="Genre">Genre</option>
              <option value="Tags">Store Tag</option>
              <option value="DevPub">Developer / Publisher</option>
              <option value="Platform">Platform</option>
              <option value="Year">Release Year</option>
              <option value="Score">Review Score</option>
              <option value="Playtime">Playtime (hours)</option>
              <option value="Hltb">HowLongToBeat (hours)</option>
              <option value="VrSupport">VR Support</option>
              <option value="Regex">Regex / Name Pattern</option>
            </select>
            {#if newRuleType === "VrSupport"}
              <label style="display: flex; align-items: center; gap: 6px; font-size: 11.5px; color: var(--text-secondary); cursor: pointer;">
                <input type="checkbox" bind:checked={newRuleVrOnly} />
                <span>VR Exclusive Only</span>
              </label>
            {:else if newRuleType === "Regex"}
              <input
                type="text"
                class="input-text"
                placeholder="Regex pattern (e.g. ^([A-Z]))"
                bind:value={newRuleParamVal}
              />
            {:else}
              <input
                type="text"
                class="input-text"
                placeholder={rangeTypes.includes(newRuleType) || newRuleType === "Score"
                  ? "Minimum"
                  : "Match value (e.g. RPG, Valve)"}
                bind:value={newRuleParamVal}
              />
            {/if}
            <div style="display: flex; gap: 6px;">
              <select class="select-control" bind:value={newRuleTargetSelect} style="flex: 1;">
                <option value="__dynamic__">⚡ Dynamic (Auto-name from match)</option>
                <option value="__custom__">➕ Fixed category name…</option>
                {#each availableTargetCategories as cat}
                  <option value={cat}>{cat}</option>
                {/each}
              </select>
              {#if newRuleTargetSelect === "__custom__"}
                <input
                  type="text"
                  class="input-text"
                  placeholder="Custom category name"
                  bind:value={newRuleTargetCustom}
                  style="flex: 1;"
                />
              {/if}
            </div>
            <button class="btn btn-primary btn-sm" onclick={addAutoCatRule}>
              Add Rule
            </button>
          </div>

          {#if newRuleType === "Regex"}
            <div style="margin-top: 8px; display: flex; gap: 10px; align-items: center; font-size: 11px;">
              <span style="color: var(--accent-cyan); font-weight: 600;">Match against:</span>
              <select class="select-control" bind:value={newRuleRegexField} style="width: 130px;">
                <option value="name">Game Title</option>
                <option value="developer">Developer</option>
                <option value="publisher">Publisher</option>
              </select>
              <span style="color: var(--text-dim);">
                Capture groups like <code>^([A-Z])</code> automatically become category names when set to Dynamic.
              </span>
            </div>
          {/if}

          {#if rangeTypes.includes(newRuleType)}
            <div style="margin-top: 8px; display: flex; gap: 8px; align-items: center;">
              {#if newRuleType === "Hltb"}
                <select class="select-control" bind:value={newRuleHltbField}>
                  <option value="main_story">Main story</option>
                  <option value="main_extras">Main + extras</option>
                  <option value="completionist">Completionist</option>
                </select>
              {/if}
              <input
                type="text"
                inputmode="decimal"
                class="input-text"
                placeholder="Maximum (optional)"
                bind:value={newRuleParamMax}
              />
            </div>
          {/if}
        </div>

        <!-- Options -->
        <div style="margin-top: 16px; display: flex; flex-direction: column; gap: 8px;">
          <label style="display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer;">
            <input type="checkbox" bind:checked={autoCatClearExisting} />
            <span>Clear existing categories before applying AutoCat</span>
          </label>
          <div style="display: flex; align-items: center; gap: 8px; font-size: 12px;">
            <label for="autocat-prefix">Category Prefix (Optional):</label>
            <input
              id="autocat-prefix"
              type="text"
              class="input-text"
              style="width: 140px;"
              placeholder="e.g. Genre:"
              bind:value={autoCatPrefix}
            />
          </div>
        </div>
      </div>

      {#if isRunningAutoCat}
        {@const autoCatPct = autoCatTotal > 0 ? Math.round((autoCatProcessed / autoCatTotal) * 100) : 0}
        <div class="modal-progress">
          <div class="modal-progress-label">
            <span>Classifying games…</span>
            <span>{autoCatProcessed} / {autoCatTotal} ({autoCatPct}%)</span>
          </div>
          <div class="progress-bar-track">
            <div class="progress-bar-fill" style="width: {autoCatPct}%"></div>
          </div>
          {#if autoCatCurrentGame}
            <div class="modal-progress-game" title={autoCatCurrentGame}>
              ▸ {autoCatCurrentGame}
            </div>
          {/if}
        </div>
      {/if}

      <div class="modal-footer">
        <button
          class="btn btn-secondary btn-sm"
          disabled={isRunningAutoCat}
          onclick={() => (showAutoCatModal = false)}
        >
          Cancel
        </button>
        <button
          class="btn btn-accent"
          disabled={isRunningAutoCat}
          onclick={handleRunAutoCat}
        >
          {isRunningAutoCat ? "⏳ Running AutoCat…" : `⚡ Execute AutoCat on ${games.length} Games`}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- =========================================================================== -->
<!-- Settings Modal                                                              -->
<!-- =========================================================================== -->
{#if showSettingsModal}
  <div
    class="modal-overlay"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => { if (e.target === e.currentTarget) showSettingsModal = false; }}
    onkeydown={(e) => { if (e.key === "Escape") showSettingsModal = false; }}
  >
    <div class="modal-dialog">
      <div class="modal-header">
        <h3>⚙️ Depressurizer Settings</h3>
        <button class="btn btn-ghost btn-sm" onclick={() => (showSettingsModal = false)}>✕</button>
      </div>

      <div class="modal-body" style="display: flex; flex-direction: column; gap: 14px;">
        <div>
          <label for="settings-steam-path" style="display:block; font-size: 11px; font-weight: 600; color: var(--text-muted); margin-bottom: 4px;">
            Steam Installation Path:
          </label>
          <div style="display: flex; gap: 6px;">
            <input
              id="settings-steam-path"
              type="text"
              class="input-text"
              style="flex: 1;"
              bind:value={settings.steam_path}
            />
            <button
              class="btn btn-secondary btn-sm"
              onclick={async () => {
                const detected = await detectSteam();
                if (detected) settings.steam_path = detected;
              }}
            >
              Auto-Detect
            </button>
          </div>
        </div>

        <div>
          <label for="settings-api-key" style="display:block; font-size: 11px; font-weight: 600; color: var(--text-muted); margin-bottom: 4px;">
            Steam Web API Key (for All Owned Games catalog):
          </label>
          <div style="display: flex; gap: 6px;">
            <input
              id="settings-api-key"
              type="password"
              class="input-text"
              style="flex: 1; font-family: monospace;"
              placeholder="Enter your 32-character key"
              bind:value={settings.api_key}
            />
            <button
              class="btn btn-secondary btn-sm"
              disabled={isTestingKey}
              onclick={handleTestApiKey}
            >
              {isTestingKey ? "Testing..." : "🔌 Test Key"}
            </button>
          </div>
          {#if apiKeyTestStatus}
            <div
              style="margin-top: 6px; font-size: 11.5px; padding: 4px 8px; border-radius: 4px; border: 1px solid;"
              style:background={apiKeyTestStatus.success ? "rgba(16, 185, 129, 0.1)" : "rgba(244, 63, 94, 0.1)"}
              style:color={apiKeyTestStatus.success ? "var(--accent-emerald)" : "var(--accent-rose)"}
              style:border-color={apiKeyTestStatus.success ? "rgba(16, 185, 129, 0.3)" : "rgba(244, 63, 94, 0.3)"}
            >
              {apiKeyTestStatus.message}
            </div>
          {/if}
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 4px;">
            Required to discover games you own that are not currently installed. Keys are stored securely in local SQLite storage.
          </div>
        </div>

        <div style="display: flex; flex-direction: column; gap: 8px; border-top: 1px solid var(--border-glass); padding-top: 10px;">
          <label style="display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer;">
            <input type="checkbox" bind:checked={settings.backup_on_save} />
            <span>Create timestamped backup before modifying sharedconfig.vdf</span>
          </label>
        </div>

        <div style="background: rgba(72, 216, 242, 0.05); padding: 8px 10px; border-radius: 4px; border: 1px solid var(--border-subtle); font-size: 11.5px; color: var(--accent-cyan);">
          <strong>Native SQLite In-App Storage:</strong> {games.length} games and {autoCatRules.length} rules cached in <code>depressurizer.db</code> with instant startup and zero UI hangs.
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary btn-sm" onclick={() => (showSettingsModal = false)}>
          Cancel
        </button>
        <button class="btn btn-primary btn-sm" onclick={handleSaveSettings}>
          Save Settings
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- =========================================================================== -->
<!-- New Category Modal                                                          -->
<!-- =========================================================================== -->
{#if showNewCatModal}
  <div
    class="modal-overlay"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => { if (e.target === e.currentTarget) showNewCatModal = false; }}
    onkeydown={(e) => { if (e.key === "Escape") showNewCatModal = false; }}
  >
    <div class="modal-dialog" style="max-width: 360px;">
      <div class="modal-header">
        <h3>Create New Category</h3>
        <button class="btn btn-ghost btn-sm" onclick={() => (showNewCatModal = false)}>✕</button>
      </div>

      <div class="modal-body">
        <label for="new-cat-name" style="display:block; font-size: 11px; font-weight: 600; color: var(--text-muted); margin-bottom: 4px;">
          Category Name:
        </label>
        <input
          id="new-cat-name"
          type="text"
          class="input-text"
          style="width: 100%;"
          placeholder="e.g. JRPG, Backlog, Co-Op"
          bind:value={newCatName}
          onkeydown={(e) => e.key === "Enter" && handleCreateCategory()}
        />
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary btn-sm" onclick={() => (showNewCatModal = false)}>
          Cancel
        </button>
        <button class="btn btn-primary btn-sm" onclick={handleCreateCategory}>
          Create
        </button>
      </div>
    </div>
  </div>
{/if}

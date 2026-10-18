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
  let newRuleTargetSelect = $state("__custom__");
  let newRuleTargetCustom = $state("");
  let newRuleParamVal = $state("");
  let newRuleParamMax = $state("");
  let newRuleHltbField = $state<"main_story" | "main_extras" | "completionist">("main_story");
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
        return text ? { genre: text } : null;
      case "Tags":
        return text ? { tag: text } : null;
      case "DevPub":
        return text ? { name: text } : null;
      case "Platform":
        return text ? { platform: text } : null;
      case "Score":
        return text && !Number.isNaN(min) ? { min_score: min } : null;
      case "Year":
        return range("min_year", "max_year");
      case "Playtime":
        return range("min_hours", "max_hours");
      case "Hltb": {
        const r = range("min_hours", "max_hours");
        return r ? { field: newRuleHltbField, ...r } : null;
      }
      default:
        return null;
    }
  }

  function addAutoCatRule() {

    const target =
      newRuleTargetSelect === "__custom__"
        ? newRuleTargetCustom.trim()
        : newRuleTargetSelect;
    const params = buildRuleParams();
    if (!newRuleName.trim() || !target || !params) return;
    const rule: AutoCatRule = {
      id: Date.now().toString(),
      name: newRuleName.trim(),
      filter_type: newRuleType,
      target_category: target,
      params,
      enabled: true,
    };
    autoCatRules.push(rule);
    newRuleName = "";
    newRuleTarget = "";
    newRuleTargetSelect = "__custom__";
    newRuleTargetCustom = "";
    newRuleParamVal = "";
    newRuleParamMax = "";
  }

  function deleteAutoCatRule(ruleId: string) {
    autoCatRules = autoCatRules.filter((r) => r.id !== ruleId);
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

        <!-- Existing Rules Table -->
        <table class="game-table" style="margin-bottom: 16px;">
          <thead>
            <tr>
              <th style="width: 40px;">Active</th>
              <th>Rule Name</th>
              <th>Filter By</th>
              <th>Match Criteria</th>
              <th>Target Category</th>
              <th style="width: 50px;">Action</th>
            </tr>
          </thead>
          <tbody>
            {#each autoCatRules as rule (rule.id)}
              <tr>
                <td style="text-align: center;">
                  <input type="checkbox" bind:checked={rule.enabled} />
                </td>
                <td style="font-weight: 600; color: var(--text-primary);">{rule.name}</td>
                <td><span class="badge badge-purple">{rule.filter_type}</span></td>
                <td>{JSON.stringify(rule.params)}</td>
                <td><span class="badge badge-cyan">{rule.target_category}</span></td>
                <td>
                  <button
                    class="btn btn-danger btn-sm"
                    style="padding: 2px 6px;"
                    onclick={() => deleteAutoCatRule(rule.id)}
                  >
                    ✕
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>

        <!-- Add New Rule Builder Card -->
        <div style="background: var(--bg-surface); padding: 12px; border-radius: 6px; border: 1px solid var(--border-subtle);">
          <div style="font-size: 11.5px; font-weight: 700; color: var(--accent-cyan); margin-bottom: 8px;">
            + Add New AutoCat Rule
          </div>
          <div style="display: grid; grid-template-columns: 1fr 1fr 1.2fr 1.2fr auto; gap: 8px; align-items: center;">
            <input
              type="text"
              class="input-text"
              placeholder="Rule Name"
              bind:value={newRuleName}
            />
            <select class="select-control" bind:value={newRuleType}>
              <option value="Genre">Genre</option>
              <option value="Tags">Tag</option>
              <option value="DevPub">Developer / Publisher</option>
              <option value="Platform">Platform</option>
              <option value="Year">Year</option>
              <option value="Score">Review Score</option>
              <option value="Playtime">Playtime (hours)</option>
              <option value="Hltb">HowLongToBeat (hours)</option>
            </select>
            <input
              type="text"
              class="input-text"
              placeholder={rangeTypes.includes(newRuleType) || newRuleType === "Score"
                ? "Minimum"
                : "Value (e.g. RPG, Indie)"}
              bind:value={newRuleParamVal}
            />
            <div style="display: flex; gap: 6px;">
              <select class="select-control" bind:value={newRuleTargetSelect} style="flex: 1;">
                <option value="__custom__">➕ Create new…</option>
                {#each availableTargetCategories as cat}
                  <option value={cat}>{cat}</option>
                {/each}
              </select>
              {#if newRuleTargetSelect === "__custom__"}
                <input
                  type="text"
                  class="input-text"
                  placeholder="New category name"
                  bind:value={newRuleTargetCustom}
                  style="flex: 1;"
                />
              {/if}
            </div>
            <button class="btn btn-primary btn-sm" onclick={addAutoCatRule}>
              Add Rule
            </button>
          </div>
        </div>

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

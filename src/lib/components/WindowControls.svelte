<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { getPlatform } from "$lib/platform";

  let { position = "right" }: { position?: "left" | "right" } = $props();

  const platform = getPlatform();
  let isMaximized = $state(false);

  async function checkMaximized() {
    try {
      isMaximized = await getCurrentWindow().isMaximized();
    } catch {}
  }

  async function onMinimize() {
    try {
      await getCurrentWindow().minimize();
    } catch {}
  }

  async function onToggleMaximize() {
    try {
      await getCurrentWindow().toggleMaximize();
      await checkMaximized();
    } catch {}
  }

  async function onClose() {
    try {
      await getCurrentWindow().close();
    } catch {}
  }

  onMount(() => {
    checkMaximized();
    let unlistenResize: (() => void) | undefined;
    try {
      getCurrentWindow()
        .onResized(() => checkMaximized())
        .then((unlisten) => {
          unlistenResize = unlisten;
        })
        .catch(() => {});
    } catch {}

    return () => {
      unlistenResize?.();
    };
  });
</script>

{#if platform === "macos"}
  <div class="window-controls macos pos-{position}" role="group" aria-label="Window controls">
    <button type="button" class="mac-btn mac-close" onclick={onClose} title="Close" aria-label="Close">
      <svg class="mac-glyph" width="6" height="6" viewBox="0 0 6 6" aria-hidden="true">
        <line x1="1" y1="1" x2="5" y2="5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
        <line x1="5" y1="1" x2="1" y2="5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      </svg>
    </button>
    <button type="button" class="mac-btn mac-min" onclick={onMinimize} title="Minimize" aria-label="Minimize">
      <svg class="mac-glyph" width="6" height="6" viewBox="0 0 6 6" aria-hidden="true">
        <line x1="1" y1="3" x2="5" y2="3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      </svg>
    </button>
    <button type="button" class="mac-btn mac-max" onclick={onToggleMaximize} title="Maximize" aria-label="Maximize">
      <svg class="mac-glyph" width="6" height="6" viewBox="0 0 6 6" aria-hidden="true">
        <polyline points="1.5,4.5 4.5,4.5 4.5,1.5" stroke="currentColor" stroke-width="1" fill="none" />
        <line x1="1.5" y1="4.5" x2="4.5" y2="1.5" stroke="currentColor" stroke-width="1" />
      </svg>
    </button>
  </div>
{:else}
  <div class="window-controls {platform} pos-{position}" role="group" aria-label="Window controls">
    {#if position === "left"}
      <button type="button" class="win-btn win-btn-close" onclick={onClose} title="Close" aria-label="Close">
        <svg width="9" height="9" viewBox="0 0 10 10" aria-hidden="true">
          <line x1="2" y1="2" x2="8" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          <line x1="8" y1="2" x2="2" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
        </svg>
      </button>
    {/if}
    <button type="button" class="win-btn" onclick={onMinimize} title="Minimize" aria-label="Minimize">
      <svg width="9" height="9" viewBox="0 0 10 10" aria-hidden="true">
        <line x1="1" y1="5" x2="9" y2="5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      </svg>
    </button>
    <button type="button" class="win-btn" onclick={onToggleMaximize} title="Maximize" aria-label="Maximize">
      {#if isMaximized}
        <svg width="9" height="9" viewBox="0 0 10 10" aria-hidden="true">
          <path d="M3 2.5h5a.5.5 0 0 1 .5.5v5" fill="none" stroke="currentColor" stroke-width="1.1" />
          <rect x="1.5" y="3.5" width="5.5" height="5.5" rx="0.75" fill="none" stroke="currentColor" stroke-width="1.1" />
        </svg>
      {:else}
        <svg width="9" height="9" viewBox="0 0 10 10" aria-hidden="true">
          <rect x="1.5" y="1.5" width="7" height="7" rx="0.75" fill="none" stroke="currentColor" stroke-width="1.2" />
        </svg>
      {/if}
    </button>
    {#if position === "right"}
      <button type="button" class="win-btn win-btn-close" onclick={onClose} title="Close" aria-label="Close">
        <svg width="9" height="9" viewBox="0 0 10 10" aria-hidden="true">
          <line x1="2" y1="2" x2="8" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          <line x1="8" y1="2" x2="2" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
        </svg>
      </button>
    {/if}
  </div>
{/if}

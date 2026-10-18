<script lang="ts">
  import "$styles/app.scss";
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import WindowControls from "$lib/components/WindowControls.svelte";
  import { getPlatform } from "$lib/platform";

  let { children } = $props();

  const platform = getPlatform();
  let topBarEl = $state<HTMLElement | null>(null);

  onMount(() => {
    function onTopBarMouseDown(e: MouseEvent) {
      if (e.buttons === 1 && !(e.target as HTMLElement)?.closest('input, button, a, select, [role="button"]')) {
        try {
          getCurrentWindow().startDragging().catch(() => {});
        } catch {}
      }
    }

    function onTopBarDblClick(e: MouseEvent) {
      if (!(e.target as HTMLElement)?.closest('input, button, a, select, [role="button"]')) {
        try {
          getCurrentWindow().toggleMaximize().catch(() => {});
        } catch {}
      }
    }

    topBarEl?.addEventListener("mousedown", onTopBarMouseDown);
    topBarEl?.addEventListener("dblclick", onTopBarDblClick);

    return () => {
      topBarEl?.removeEventListener("mousedown", onTopBarMouseDown);
      topBarEl?.removeEventListener("dblclick", onTopBarDblClick);
    };
  });
</script>

<div class="app-shell">
  <!-- HELIX Origin Standard Custom Titlebar -->
  <header class="titlebar" bind:this={topBarEl} data-tauri-drag-region>
    {#if platform === "macos"}
      <WindowControls position="left" />
    {/if}

    <div class="tb-brand" data-tauri-drag-region>
      <div class="tb-mark">
        <img src="/favicon.svg" alt="Depressurizer" width="20" height="20" />
      </div>
      <div class="tb-title-group">
        <span class="tb-title">Depressurizer</span>
        <span class="tb-badge">v0.8.0</span>
      </div>
    </div>

    <div class="tb-drag-spacer" data-tauri-drag-region></div>

    {#if platform !== "macos"}
      <WindowControls position="right" />
    {/if}
  </header>

  <!-- Page Content Viewport -->
  <main class="main-viewport">
    {@render children?.()}
  </main>
</div>

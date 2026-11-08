<script lang="ts">
  import type { Snippet } from 'svelte';
  import { closeSheet } from '$lib/state.svelte';
  import Icon from './Icon.svelte';

  let {
    title = '',
    tall = false,
    level = 0,
    header,
    children,
  }: { title?: string; tall?: boolean; level?: number; header?: Snippet; children: Snippet } = $props();
</script>

<div class="scrim" style:z-index={40 + level * 2} onclick={closeSheet} aria-hidden="true"></div>
<div
  class="sheet"
  class:tall
  style:z-index={41 + level * 2}
  role="dialog"
  aria-modal="true"
  aria-label={title}
>
  <div class="grip" aria-hidden="true"></div>
  <header>
    {#if header}
      {@render header()}
    {:else}
      <h2>{title}</h2>
      <button class="icon-btn" onclick={closeSheet} aria-label="Close"><Icon name="x" /></button>
    {/if}
  </header>
  <div class="body">
    {@render children()}
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: var(--scrim);
    animation: fade 0.2s ease;
  }
  .sheet {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    max-height: calc(100dvh - var(--safe-t) - 24px);
    display: flex;
    flex-direction: column;
    background: var(--paper);
    border-radius: var(--r-lg) var(--r-lg) 0 0;
    padding-bottom: var(--safe-b);
    animation: rise 0.28s cubic-bezier(0.2, 0.9, 0.25, 1);
    max-width: 640px;
    margin: 0 auto;
  }
  .sheet.tall {
    height: calc(100dvh - var(--safe-t) - 24px);
  }
  .grip {
    width: 36px;
    height: 4px;
    border-radius: 99px;
    background: var(--line);
    margin: 8px auto 0;
    flex: none;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 8px 6px var(--pad);
    min-height: 56px;
    flex: none;
  }
  header h2 {
    font-size: 1.25rem;
  }
  .body {
    flex: 1;
    overflow-y: auto;
    overscroll-behavior: contain;
    display: flex;
    flex-direction: column;
  }
  @keyframes rise {
    from {
      transform: translateY(100%);
    }
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
</style>

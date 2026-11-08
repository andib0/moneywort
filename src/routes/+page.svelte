<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { app, load, onPopState, openSheet, setTab, type Tab } from '$lib/state.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Home from '$lib/screens/Home.svelte';
  import Activity from '$lib/screens/Activity.svelte';
  import Insights from '$lib/screens/Insights.svelte';
  import Plan from '$lib/screens/Plan.svelte';
  import TxSheet from '$lib/components/TxSheet.svelte';
  import SettingsSheet from '$lib/components/SettingsSheet.svelte';
  import AccountSheet from '$lib/components/AccountSheet.svelte';
  import CategorySheet from '$lib/components/CategorySheet.svelte';
  import BudgetSheet from '$lib/components/BudgetSheet.svelte';
  import RecurringSheet from '$lib/components/RecurringSheet.svelte';
  import ConfirmSheet from '$lib/components/ConfirmSheet.svelte';

  onMount(() => {
    load();
    window.addEventListener('popstate', onPopState);
    return () => window.removeEventListener('popstate', onPopState);
  });

  // Theme: 'system' follows the OS; 'light'/'dark' force it.
  $effect(() => {
    const t = app.settings.theme;
    if (t === 'light' || t === 'dark') document.documentElement.dataset.theme = t;
    else delete document.documentElement.dataset.theme;
  });

  let main: HTMLElement | undefined = $state();
  $effect(() => {
    void app.tab;
    main?.scrollTo({ top: 0 });
  });

  const tabs: { id: Tab; label: string; icon: string }[] = [
    { id: 'home', label: 'Home', icon: 'home' },
    { id: 'activity', label: 'Activity', icon: 'list' },
    { id: 'insights', label: 'Insights', icon: 'chart' },
    { id: 'plan', label: 'Plan', icon: 'target' },
  ];
</script>

{#if app.error}
  <div class="fatal">
    <h1>Penny couldn't open its data</h1>
    <p>{app.error}</p>
    <button class="btn primary" onclick={() => location.reload()}>Try again</button>
  </div>
{:else if app.ready}
  <main bind:this={main}>
    {#if app.tab === 'home'}<Home />
    {:else if app.tab === 'activity'}<Activity />
    {:else if app.tab === 'insights'}<Insights />
    {:else}<Plan />{/if}
  </main>

  <nav aria-label="Sections">
    {#each tabs.slice(0, 2) as t (t.id)}
      <button class="tab" aria-current={app.tab === t.id ? 'page' : undefined} onclick={() => setTab(t.id)}>
        <Icon name={t.icon} size={23} stroke={app.tab === t.id ? 2.2 : 1.7} />
        <span>{t.label}</span>
      </button>
    {/each}
    <button class="add" onclick={() => openSheet({ type: 'tx' })} aria-label="Add entry">
      <Icon name="plus" size={28} stroke={2.4} />
    </button>
    {#each tabs.slice(2) as t (t.id)}
      <button class="tab" aria-current={app.tab === t.id ? 'page' : undefined} onclick={() => setTab(t.id)}>
        <Icon name={t.icon} size={23} stroke={app.tab === t.id ? 2.2 : 1.7} />
        <span>{t.label}</span>
      </button>
    {/each}
  </nav>

  {#each app.sheets as s, i (i)}
    {#if s.type === 'tx'}<TxSheet tx={s.tx} kind={s.kind} level={i} />
    {:else if s.type === 'settings'}<SettingsSheet level={i} />
    {:else if s.type === 'account'}<AccountSheet account={s.account} level={i} />
    {:else if s.type === 'category'}<CategorySheet category={s.category} kind={s.kind} level={i} />
    {:else if s.type === 'budget'}<BudgetSheet categoryId={s.categoryId} level={i} />
    {:else if s.type === 'recurring'}<RecurringSheet rule={s.rule} level={i} />
    {:else if s.type === 'confirm'}<ConfirmSheet {...s} level={i} />{/if}
  {/each}

  {#if app.toast}
    <div class="toast" role="status">{app.toast}</div>
  {/if}
{/if}

<style>
  main {
    position: fixed;
    inset: 0;
    overflow-y: auto;
    padding-top: var(--safe-t);
    padding-bottom: calc(var(--nav-h) + var(--safe-b) + 28px);
    max-width: 640px;
    margin: 0 auto;
  }
  nav {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 20;
    display: flex;
    align-items: center;
    justify-content: space-around;
    height: calc(var(--nav-h) + var(--safe-b));
    padding: 0 6px var(--safe-b);
    background: color-mix(in srgb, var(--paper) 88%, transparent);
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
    border-top: 1px solid var(--line);
  }
  .tab {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    padding: 8px 0;
    color: var(--faint);
    font-size: 0.72rem;
    font-weight: 600;
  }
  .tab[aria-current='page'] {
    color: var(--ink);
  }
  .add {
    width: 58px;
    height: 58px;
    border-radius: 20px;
    background: var(--ink);
    color: var(--paper);
    display: grid;
    place-items: center;
    flex: none;
    margin: 0 6px;
    transform: translateY(-10px);
    box-shadow: 0 8px 20px -8px color-mix(in srgb, var(--ink) 60%, transparent);
    transition: transform 0.1s;
  }
  .add:active {
    transform: translateY(-10px) scale(0.94);
  }
  .toast {
    position: fixed;
    left: 50%;
    bottom: calc(var(--nav-h) + var(--safe-b) + 24px);
    transform: translateX(-50%);
    z-index: 100;
    background: var(--ink);
    color: var(--paper);
    padding: 12px 18px;
    border-radius: 999px;
    font-size: 0.92rem;
    font-weight: 550;
    max-width: calc(100vw - 40px);
    animation: pop 0.22s cubic-bezier(0.2, 0.9, 0.25, 1.2);
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translate(-50%, 10px);
    }
  }
  .fatal {
    padding: 60px var(--pad);
    display: flex;
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
  }
  .fatal p {
    color: var(--muted);
    margin: 0;
  }
</style>

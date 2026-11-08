<script lang="ts">
  import { api } from '$lib/api';
  import { app, currency } from '$lib/state.svelte';
  import { money, monthName } from '$lib/fmt';
  import type { Tx, TxKind } from '$lib/types';
  import MonthSwitch from '$lib/components/MonthSwitch.svelte';
  import TxList from '$lib/components/TxList.svelte';
  import Icon from '$lib/components/Icon.svelte';

  let items = $state<Tx[]>([]);
  let query = $state('');
  let kind = $state<TxKind | ''>('');
  let accountId = $state<number | 0>(0);
  let loaded = $state(false);

  $effect(() => {
    void app.version;
    const filter = {
      // Searching looks across all time; browsing is per month.
      month: query.trim() ? undefined : app.month,
      kind: kind || undefined,
      account_id: accountId || undefined,
      query: query.trim() || undefined,
      limit: query.trim() ? 300 : undefined,
    };
    api.transactions(filter).then((t) => {
      items = t;
      loaded = true;
    });
  });

  const totals = $derived.by(() => {
    let i = 0;
    let e = 0;
    for (const t of items) {
      if (t.kind === 'income') i += t.amount;
      if (t.kind === 'expense') e += t.amount;
    }
    return { i, e };
  });

  const accounts = $derived(app.accounts.filter((a) => !a.archived));
</script>

<header class="top">
  <h1>Activity</h1>
  {#if !query.trim()}<MonthSwitch />{/if}
</header>

<div class="search">
  <Icon name="search" size={18} />
  <input type="search" placeholder="Search notes and categories" bind:value={query} enterkeyhint="search" />
  {#if query}
    <button class="icon-btn" onclick={() => (query = '')} aria-label="Clear search"><Icon name="x" size={18} /></button>
  {/if}
</div>

<div class="chips filters">
  <button class="chip" aria-pressed={kind === ''} onclick={() => (kind = '')}>All</button>
  <button class="chip" aria-pressed={kind === 'expense'} onclick={() => (kind = 'expense')}>Expenses</button>
  <button class="chip" aria-pressed={kind === 'income'} onclick={() => (kind = 'income')}>Income</button>
  <button class="chip" aria-pressed={kind === 'transfer'} onclick={() => (kind = 'transfer')}>Transfers</button>
  {#if accounts.length > 1}
    <span class="sep" aria-hidden="true"></span>
    {#each accounts as a (a.id)}
      <button class="chip" aria-pressed={accountId === a.id} onclick={() => (accountId = accountId === a.id ? 0 : a.id)}>{a.name}</button>
    {/each}
  {/if}
</div>

{#if items.length}
  <div class="totals num">
    <span>In <b class="in">{money(totals.i, currency())}</b></span>
    <span>Out <b>{money(totals.e, currency())}</b></span>
    <span class="muted">{items.length} {items.length === 1 ? 'entry' : 'entries'}</span>
  </div>
  <TxList {items} />
{:else if loaded}
  <div class="empty">
    {#if query.trim()}
      <strong>No matches for “{query.trim()}”</strong>
      Try a shorter word or a category name.
    {:else}
      <strong>No entries in {monthName(app.month)}</strong>
      Tap + to add one, or switch months above.
    {/if}
  </div>
{/if}

<style>
  .top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 8px 4px var(--pad);
    min-height: 60px;
  }
  h1 {
    font-size: 1.7rem;
  }
  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 8px var(--pad) 12px;
    padding: 0 4px 0 14px;
    border-radius: 999px;
    background: var(--sunk);
    color: var(--muted);
    min-height: 46px;
  }
  .search input {
    flex: 1;
    border: 0;
    background: transparent;
    min-height: 44px;
    color: var(--ink);
  }
  .search input:focus {
    outline: none;
  }
  .search input::-webkit-search-cancel-button {
    display: none;
  }
  .sep {
    width: 1px;
    background: var(--line);
    flex: none;
    margin: 6px 2px;
  }
  .totals {
    display: flex;
    gap: 18px;
    padding: 16px var(--pad) 0;
    font-size: 0.9rem;
    color: var(--muted);
  }
  .totals b {
    color: var(--ink);
  }
  .totals b.in {
    color: var(--mint);
  }
  .totals .muted {
    margin-left: auto;
  }
</style>

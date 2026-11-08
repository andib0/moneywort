<script lang="ts">
  import { api, openTextFile, saveTextFile } from '$lib/api';
  import { app, confirmAction, load, mutate, openSheet, toast } from '$lib/state.svelte';
  import { todayISO } from '$lib/fmt';
  import type { CatKind } from '$lib/types';
  import Sheet from './Sheet.svelte';
  import Icon from './Icon.svelte';

  let { level }: { level: number } = $props();

  const currencies = ['EUR', 'USD', 'GBP', 'CHF', 'ALL', 'MKD', 'RSD', 'PLN', 'CZK', 'SEK', 'NOK', 'DKK', 'TRY', 'CAD', 'AUD', 'JPY', 'INR', 'BRL'];
  let catKind = $state<CatKind>('expense');

  const accounts = $derived(app.accounts);
  const cats = $derived(app.categories.filter((c) => c.kind === catKind));

  async function setCurrency(e: Event) {
    const v = (e.target as HTMLSelectElement).value;
    await mutate(() => api.setSetting('currency', v));
  }

  async function setTheme(v: string) {
    await mutate(() => api.setSetting('theme', v));
  }

  async function exportBackup() {
    try {
      const ok = await saveTextFile(`penny-backup-${todayISO()}.json`, await api.exportJson(), 'json');
      if (ok) toast('Backup saved');
    } catch (e) {
      toast(String(e));
    }
  }

  async function exportCsv() {
    try {
      const ok = await saveTextFile(`penny-${todayISO()}.csv`, await api.exportCsv(), 'csv');
      if (ok) toast('Spreadsheet saved');
    } catch (e) {
      toast(String(e));
    }
  }

  async function restore() {
    let text: string | null;
    try {
      text = await openTextFile('json');
    } catch (e) {
      toast(String(e));
      return;
    }
    if (!text) return;
    confirmAction(
      'Restore this backup?',
      'Everything currently in Penny will be replaced by the backup. Save a backup first if you want to keep it.',
      'Restore',
      async () => {
        await mutate(() => api.importJson(text!), 'Backup restored');
      },
    );
  }

  function erase() {
    confirmAction(
      'Erase all data?',
      'All entries, accounts, budgets and recurring items will be permanently deleted from this phone. This cannot be undone.',
      'Erase everything',
      async () => {
        await mutate(() => api.wipe(), 'All data erased');
        await load();
      },
    );
  }
</script>

<Sheet title="Settings" tall {level}>
  <div class="group">
    <div class="setting">
      <label for="currency">Currency</label>
      <select id="currency" class="input select" value={app.settings.currency} onchange={setCurrency}>
        {#each currencies as c (c)}<option value={c}>{c}</option>{/each}
        {#if !currencies.includes(app.settings.currency)}<option value={app.settings.currency}>{app.settings.currency}</option>{/if}
      </select>
    </div>
    <div class="setting">
      <span class="label">Appearance</span>
      <div class="seg" role="group" aria-label="Appearance">
        <button aria-pressed={app.settings.theme === 'system'} onclick={() => setTheme('system')}>Auto</button>
        <button aria-pressed={app.settings.theme === 'light'} onclick={() => setTheme('light')}>Light</button>
        <button aria-pressed={app.settings.theme === 'dark'} onclick={() => setTheme('dark')}>Dark</button>
      </div>
    </div>
  </div>

  <div class="section-head">
    <h2>Accounts</h2>
    <button class="link" onclick={() => openSheet({ type: 'account' })}>Add account</button>
  </div>
  <ul class="list">
    {#each accounts as a (a.id)}
      <li>
        <button class="row" class:archived={a.archived} onclick={() => openSheet({ type: 'account', account: a })}>
          <span class="glyph" style:--c={a.color}><Icon name="wallet" size={19} /></span>
          <span class="grow"><span class="title">{a.name}</span>{#if a.archived}<span class="sub">Hidden</span>{/if}</span>
          <Icon name="right" size={18} />
        </button>
      </li>
    {/each}
  </ul>

  <div class="section-head">
    <h2>Categories</h2>
    <button class="link" onclick={() => openSheet({ type: 'category', kind: catKind })}>Add category</button>
  </div>
  <div class="pad">
    <div class="seg" role="group" aria-label="Category type">
      <button aria-pressed={catKind === 'expense'} onclick={() => (catKind = 'expense')}>Expenses</button>
      <button aria-pressed={catKind === 'income'} onclick={() => (catKind = 'income')}>Income</button>
    </div>
  </div>
  <ul class="list">
    {#each cats as c (c.id)}
      <li>
        <button class="row" class:archived={c.archived} onclick={() => openSheet({ type: 'category', category: c })}>
          <span class="glyph" style:--c={c.color}>{c.icon}</span>
          <span class="grow"><span class="title">{c.name}</span>{#if c.archived}<span class="sub">Hidden</span>{/if}</span>
          <Icon name="right" size={18} />
        </button>
      </li>
    {/each}
  </ul>

  <div class="section-head"><h2>Your data</h2></div>
  <ul class="list">
    <li><button class="row" onclick={exportBackup}><span class="glyph"><Icon name="download" size={19} /></span><span class="grow"><span class="title">Save a backup</span><span class="sub">A file you can restore later or on a new phone</span></span></button></li>
    <li><button class="row" onclick={restore}><span class="glyph"><Icon name="upload" size={19} /></span><span class="grow"><span class="title">Restore from backup</span><span class="sub">Replaces what's here now</span></span></button></li>
    <li><button class="row" onclick={exportCsv}><span class="glyph"><Icon name="list" size={19} /></span><span class="grow"><span class="title">Export spreadsheet</span><span class="sub">CSV for Excel or Google Sheets</span></span></button></li>
    <li><button class="row danger" onclick={erase}><span class="glyph"><Icon name="trash" size={19} /></span><span class="grow"><span class="title">Erase all data</span></span></button></li>
  </ul>

  <p class="about">
    <Icon name="shield" size={18} />
    <span>Penny keeps everything on this phone. No account, no cloud, no tracking. Save a backup now and then so you never lose your history.</span>
  </p>
</Sheet>

<style>
  .group {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 4px var(--pad) 0;
  }
  .setting {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }
  .setting label,
  .setting .label {
    font-weight: 550;
  }
  .setting .seg {
    width: 220px;
  }
  .select {
    width: 120px;
    min-height: 44px;
  }
  .pad {
    padding: 0 var(--pad) 6px;
  }
  .archived {
    opacity: 0.55;
  }
  .grow {
    display: flex;
    flex-direction: column;
  }
  .row :global(svg) {
    flex: none;
  }
  .row > :global(svg:last-child) {
    color: var(--faint);
  }
  .danger .title,
  .danger .glyph {
    color: var(--rust);
  }
  .about {
    display: flex;
    gap: 10px;
    margin: 24px var(--pad) 32px;
    padding: 14px;
    border-radius: var(--r-md);
    background: var(--mint-soft);
    color: var(--ink);
    font-size: 0.88rem;
  }
  .about :global(svg) {
    flex: none;
    color: var(--mint);
    margin-top: 1px;
  }
</style>

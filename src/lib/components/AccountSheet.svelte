<script lang="ts">
  import type { Account } from '$lib/types';
  import { api } from '$lib/api';
  import { app, closeSheet, confirmAction, mutate, toast } from '$lib/state.svelte';
  import { fromCents } from '$lib/fmt';
  import Sheet from './Sheet.svelte';
  import ColorPick from './ColorPick.svelte';

  let { account, level }: { account?: Account; level: number } = $props();

  const kinds = [
    ['cash', 'Cash'],
    ['bank', 'Bank'],
    ['card', 'Card'],
    ['savings', 'Savings'],
  ] as const;

  let name = $state(account?.name ?? '');
  let kind = $state(account?.kind ?? 'bank');
  let color = $state(account?.color ?? '#4a7bb7');
  let opening = $state(account ? (account.opening_balance < 0 ? '-' : '') + fromCents(Math.abs(account.opening_balance)) : '');
  let archived = $state(account?.archived ?? false);

  function parse(v: string): number | null {
    const t = v.replace(',', '.').trim();
    if (!t) return 0;
    if (!/^-?\d+(\.\d{0,2})?$/.test(t)) return null;
    return Math.round(parseFloat(t) * 100);
  }
  const openingCents = $derived(parse(opening));

  async function save() {
    if (!name.trim() || openingCents === null) return;
    const ok = await mutate(() =>
      api.saveAccount({ id: account?.id ?? 0, name, kind, color, opening_balance: openingCents, archived, sort: account?.sort ?? 0 }),
    );
    if (ok) closeSheet();
  }

  function remove() {
    confirmAction(
      `Delete ${account!.name}?`,
      'If this account has entries it will be hidden instead, so your history stays correct.',
      'Delete',
      async () => {
        const r = await mutate(() => api.deleteAccount(account!.id));
        if (r) {
          toast(r.value ? `${account!.name} hidden, it has entries` : `${account!.name} deleted`);
          closeSheet();
        }
      },
    );
  }

  const lastActive = $derived(!!account && !account.archived && app.accounts.filter((a) => !a.archived).length <= 1);
</script>

<Sheet title={account ? 'Edit account' : 'New account'} {level}>
  <form class="form" onsubmit={(e) => { e.preventDefault(); save(); }}>
    <div class="field">
      <label for="aname">Name</label>
      <input id="aname" class="input" bind:value={name} placeholder="e.g. Main bank" maxlength="40" />
    </div>
    <div class="field">
      <span class="label">Type</span>
      <div class="seg" role="group" aria-label="Account type">
        {#each kinds as [k, l] (k)}
          <button type="button" aria-pressed={kind === k} onclick={() => (kind = k)}>{l}</button>
        {/each}
      </div>
    </div>
    <div class="field">
      <label for="aopen">Starting balance</label>
      <input id="aopen" class="input num" inputmode="decimal" bind:value={opening} placeholder="0.00" />
      {#if openingCents === null}<span class="err">Use a number like 1250.50</span>{/if}
    </div>
    <div class="field">
      <span class="label">Colour</span>
      <ColorPick bind:value={color} />
    </div>
    {#if account}
      <label class="toggle">
        <input type="checkbox" bind:checked={archived} disabled={lastActive && !archived} />
        <span>Hide from lists</span>
      </label>
    {/if}
    <button class="btn primary block" disabled={!name.trim() || openingCents === null}>{account ? 'Save changes' : 'Add account'}</button>
    {#if account && !lastActive}
      <button type="button" class="btn danger block" onclick={remove}>Delete account</button>
    {/if}
  </form>
</Sheet>

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 4px var(--pad) 24px;
  }
  .err {
    color: var(--rust);
    font-size: 0.84rem;
  }
  .toggle {
    display: flex;
    gap: 10px;
    align-items: center;
    min-height: 44px;
  }
  .toggle input {
    width: 20px;
    height: 20px;
    accent-color: var(--mint);
  }
</style>

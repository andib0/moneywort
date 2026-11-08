<script lang="ts">
  import type { Tx, TxKind } from '$lib/types';
  import { api } from '$lib/api';
  import { app, closeSheet, confirmAction, currency, mutate } from '$lib/state.svelte';
  import { dayLabel, fromCents, iso, moneyParts, toCents, todayISO } from '$lib/fmt';
  import Sheet from './Sheet.svelte';
  import Keypad from './Keypad.svelte';
  import Icon from './Icon.svelte';

  let { tx, kind: initialKind, level }: { tx?: Tx; kind?: TxKind; level: number } = $props();

  const editing = !!tx;
  const activeAccounts = $derived(app.accounts.filter((a) => !a.archived || a.id === tx?.account_id || a.id === tx?.to_account_id));

  function lastAccount(): number {
    try {
      const id = Number(localStorage.getItem('penny-last-account'));
      if (app.accounts.some((a) => a.id === id && !a.archived)) return id;
    } catch {
      /* ignore */
    }
    return app.accounts.find((a) => !a.archived)?.id ?? 0;
  }

  let kind = $state<TxKind>(tx?.kind ?? initialKind ?? 'expense');
  let amount = $state(fromCents(tx?.amount ?? 0));
  let categoryId = $state<number | null>(tx?.category_id ?? null);
  let accountId = $state<number>(tx?.account_id ?? lastAccount());
  let toAccountId = $state<number | null>(
    tx?.to_account_id ?? app.accounts.find((a) => !a.archived && a.id !== accountId)?.id ?? null,
  );
  let date = $state(tx?.date ?? todayISO());
  let note = $state(tx?.note ?? '');
  let saving = $state(false);
  let dateInput: HTMLInputElement | undefined = $state();

  const cats = $derived(app.categories.filter((c) => c.kind === kind && (!c.archived || c.id === tx?.category_id)));
  const cents = $derived(toCents(amount));
  const parts = $derived(moneyParts(cents, currency()));

  // Switching type clears a category that no longer fits.
  $effect(() => {
    if (kind !== 'transfer' && categoryId != null && !cats.some((c) => c.id === categoryId)) categoryId = null;
  });

  const yesterday = (() => {
    const d = new Date();
    d.setDate(d.getDate() - 1);
    return iso(d);
  })();

  const problem = $derived.by(() => {
    if (cents <= 0) return 'Enter an amount';
    if (!accountId) return 'Add an account first';
    if (kind === 'transfer') {
      if (!toAccountId) return 'Pick where it goes';
      if (toAccountId === accountId) return 'Pick two different accounts';
    } else if (categoryId == null) return 'Pick a category';
    return '';
  });

  async function save() {
    if (problem || saving) return;
    saving = true;
    const ok = await mutate(() =>
      api.saveTx({
        id: tx?.id ?? 0,
        kind,
        amount: cents,
        account_id: accountId,
        to_account_id: kind === 'transfer' ? toAccountId : null,
        category_id: kind === 'transfer' ? null : categoryId,
        date,
        note,
        recurring_id: tx?.recurring_id ?? null,
      }),
    );
    saving = false;
    if (ok) {
      try {
        localStorage.setItem('penny-last-account', String(accountId));
      } catch {
        /* ignore */
      }
      closeSheet();
    }
  }

  function remove() {
    confirmAction('Delete this entry?', 'It will be removed from your history and balances.', 'Delete', async () => {
      await mutate(() => api.deleteTx(tx!.id), 'Entry deleted');
      closeSheet();
    });
  }

  function pickDate() {
    try {
      dateInput?.showPicker();
    } catch {
      dateInput?.focus();
    }
  }
</script>

<Sheet title={editing ? 'Edit entry' : 'New entry'} tall {level}>
  {#snippet header()}
    <div class="seg kinds" role="group" aria-label="Entry type">
      <button aria-pressed={kind === 'expense'} onclick={() => (kind = 'expense')}>Expense</button>
      <button aria-pressed={kind === 'income'} onclick={() => (kind = 'income')}>Income</button>
      <button aria-pressed={kind === 'transfer'} onclick={() => (kind = 'transfer')}>Transfer</button>
    </div>
    {#if editing}
      <button class="icon-btn" onclick={remove} aria-label="Delete entry"><Icon name="trash" size={20} /></button>
    {/if}
    <button class="icon-btn" onclick={closeSheet} aria-label="Close"><Icon name="x" /></button>
  {/snippet}

  <div class="amount" class:empty={!cents} class:in={kind === 'income'} aria-live="polite">
    <span class="main num">{parts.main}</span><span class="frac num">{parts.frac}</span>
  </div>
  <input class="note" placeholder="Add a note" bind:value={note} maxlength="120" enterkeyhint="done" />

  <div class="scroll">
    {#if kind === 'transfer'}
      <div class="xfer">
        <div class="field">
          <span class="label">From</span>
          <div class="chips flush">
            {#each activeAccounts as a (a.id)}
              <button class="chip" aria-pressed={accountId === a.id} onclick={() => (accountId = a.id)}>{a.name}</button>
            {/each}
          </div>
        </div>
        <div class="field">
          <span class="label">To</span>
          <div class="chips flush">
            {#each activeAccounts as a (a.id)}
              <button class="chip" aria-pressed={toAccountId === a.id} disabled={a.id === accountId} onclick={() => (toAccountId = a.id)}>{a.name}</button>
            {/each}
          </div>
        </div>
      </div>
    {:else}
      <div class="cats" role="radiogroup" aria-label="Category">
        {#each cats as c (c.id)}
          <button class="cat" role="radio" aria-checked={categoryId === c.id} style:--c={c.color} onclick={() => (categoryId = c.id)}>
            <span class="glyph">{c.icon}</span>
            <span class="cname">{c.name}</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <div class="meta">
    {#if kind !== 'transfer' && activeAccounts.length > 1}
      <div class="chips">
        {#each activeAccounts as a (a.id)}
          <button class="chip" aria-pressed={accountId === a.id} onclick={() => (accountId = a.id)}>
            <i class="dot" style:background={a.color}></i>{a.name}
          </button>
        {/each}
      </div>
    {/if}
    <div class="chips">
      <button class="chip" aria-pressed={date === todayISO()} onclick={() => (date = todayISO())}>Today</button>
      <button class="chip" aria-pressed={date === yesterday} onclick={() => (date = yesterday)}>Yesterday</button>
      <button class="chip" aria-pressed={date !== todayISO() && date !== yesterday} onclick={pickDate}>
        <Icon name="calendar" size={16} />
        {date !== todayISO() && date !== yesterday ? dayLabel(date) : 'Other day'}
      </button>
      <input class="hidden-date" type="date" bind:this={dateInput} bind:value={date} max={todayISO()} tabindex="-1" aria-hidden="true" />
    </div>
  </div>

  <Keypad bind:value={amount} />

  <div class="save">
    <button class="btn primary block" disabled={!!problem || saving} onclick={save}>
      {problem || (editing ? 'Save changes' : kind === 'income' ? 'Add income' : kind === 'transfer' ? 'Add transfer' : 'Add expense')}
    </button>
  </div>
</Sheet>

<style>
  .kinds {
    flex: 1;
    max-width: 330px;
  }
  .amount {
    text-align: center;
    font-family: var(--display);
    font-weight: 600;
    letter-spacing: -0.03em;
    padding: 6px var(--pad) 0;
    line-height: 1;
    flex: none;
  }
  .amount .main {
    font-size: clamp(2.8rem, 15vw, 4rem);
  }
  .amount .frac {
    font-size: 1.6rem;
    color: var(--muted);
  }
  .amount.empty {
    color: var(--faint);
  }
  .amount.in .main {
    color: var(--mint);
  }
  .note {
    display: block;
    margin: 8px auto 4px;
    border: 0;
    background: transparent;
    text-align: center;
    width: 80%;
    min-height: 40px;
    color: var(--ink);
    font-size: 0.95rem;
    flex: none;
  }
  .note::placeholder {
    color: var(--faint);
  }
  .note:focus {
    outline: none;
  }
  .scroll {
    flex: 1;
    min-height: 120px;
    overflow-y: auto;
    padding: 6px 0;
  }
  .cats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 4px 2px;
    padding: 0 12px;
  }
  .cat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    padding: 8px 2px;
    border-radius: 14px;
    min-width: 0;
  }
  .cat .glyph {
    width: 46px;
    height: 46px;
    border-radius: 16px;
    font-size: 1.3rem;
    transition: transform 0.15s;
  }
  .cat[aria-checked='true'] .glyph {
    background: var(--c);
    color: #fff;
    transform: scale(1.06);
  }
  .cname {
    font-size: 0.76rem;
    color: var(--muted);
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .cat[aria-checked='true'] .cname {
    color: var(--ink);
    font-weight: 600;
  }
  .xfer {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 4px var(--pad);
  }
  .chips.flush {
    padding: 0;
    flex-wrap: wrap;
  }
  .chip:disabled {
    opacity: 0.35;
  }
  .meta {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px 0 10px;
    border-top: 1px solid var(--line);
    flex: none;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .hidden-date {
    position: absolute;
    opacity: 0;
    pointer-events: none;
    width: 1px;
    height: 1px;
  }
  .save {
    padding: 10px var(--pad) 12px;
    flex: none;
  }
</style>

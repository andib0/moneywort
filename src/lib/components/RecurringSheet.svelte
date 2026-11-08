<script lang="ts">
  import type { CatKind, Interval, Recurring } from '$lib/types';
  import { api } from '$lib/api';
  import { app, closeSheet, confirmAction, mutate } from '$lib/state.svelte';
  import { fromCents, todayISO } from '$lib/fmt';
  import Sheet from './Sheet.svelte';

  let { rule, level }: { rule?: Recurring; level: number } = $props();

  let kind = $state<CatKind>(rule?.kind ?? 'expense');
  let amount = $state(fromCents(rule?.amount ?? 0));
  let note = $state(rule?.note ?? '');
  let categoryId = $state<number | null>(rule?.category_id ?? null);
  let accountId = $state(rule?.account_id ?? app.accounts.find((a) => !a.archived)?.id ?? 0);
  let interval = $state<Interval>(rule?.interval ?? 'monthly');
  let start = $state(rule?.next_date || todayISO());
  let active = $state(rule?.active ?? true);

  const cats = $derived(app.categories.filter((c) => c.kind === kind && (!c.archived || c.id === rule?.category_id)));
  const accounts = $derived(app.accounts.filter((a) => !a.archived || a.id === rule?.account_id));

  $effect(() => {
    if (categoryId != null && !cats.some((c) => c.id === categoryId)) categoryId = null;
  });

  function cents(v: string): number | null {
    const t = v.replace(',', '.').trim();
    if (!/^\d+(\.\d{0,2})?$/.test(t)) return null;
    return Math.round(parseFloat(t) * 100);
  }
  const c = $derived(cents(amount));
  const problem = $derived(!c ? 'Enter an amount' : categoryId == null ? 'Pick a category' : !accountId ? 'Add an account first' : '');

  async function save() {
    if (problem) return;
    const r = await mutate(
      () =>
        api.saveRecurring({
          id: rule?.id ?? 0, kind, amount: c!, account_id: accountId, category_id: categoryId, note,
          interval, start_date: start, posted: rule?.posted ?? 0, active, next_date: '',
        }),
      rule ? 'Recurring updated' : 'Recurring added',
    );
    if (r) closeSheet();
  }

  function remove() {
    confirmAction('Stop this recurring entry?', 'Entries already logged stay in your history. No new ones will be added.', 'Delete', async () => {
      await mutate(() => api.deleteRecurring(rule!.id), 'Recurring deleted');
      closeSheet();
    });
  }
</script>

<Sheet title={rule ? 'Edit recurring' : 'New recurring'} tall {level}>
  <form class="form" onsubmit={(e) => { e.preventDefault(); save(); }}>
    <div class="seg" role="group" aria-label="Type">
      <button type="button" aria-pressed={kind === 'expense'} onclick={() => (kind = 'expense')}>Expense</button>
      <button type="button" aria-pressed={kind === 'income'} onclick={() => (kind = 'income')}>Income</button>
    </div>
    <div class="two">
      <div class="field">
        <label for="ramt">Amount</label>
        <input id="ramt" class="input num" inputmode="decimal" bind:value={amount} placeholder="0.00" />
      </div>
      <div class="field">
        <label for="rnote">Name</label>
        <input id="rnote" class="input" bind:value={note} placeholder={kind === 'expense' ? 'e.g. Rent' : 'e.g. Salary'} maxlength="60" />
      </div>
    </div>
    <div class="field">
      <span class="label">Category</span>
      <div class="chips flush">
        {#each cats as cat (cat.id)}
          <button type="button" class="chip" aria-pressed={categoryId === cat.id} onclick={() => (categoryId = cat.id)}>{cat.icon} {cat.name}</button>
        {/each}
      </div>
    </div>
    {#if accounts.length > 1}
      <div class="field">
        <span class="label">Account</span>
        <div class="chips flush">
          {#each accounts as a (a.id)}
            <button type="button" class="chip" aria-pressed={accountId === a.id} onclick={() => (accountId = a.id)}>{a.name}</button>
          {/each}
        </div>
      </div>
    {/if}
    <div class="field">
      <span class="label">Repeats</span>
      <div class="seg" role="group" aria-label="Repeats">
        <button type="button" aria-pressed={interval === 'weekly'} onclick={() => (interval = 'weekly')}>Weekly</button>
        <button type="button" aria-pressed={interval === 'monthly'} onclick={() => (interval = 'monthly')}>Monthly</button>
        <button type="button" aria-pressed={interval === 'yearly'} onclick={() => (interval = 'yearly')}>Yearly</button>
      </div>
    </div>
    <div class="field">
      <label for="rstart">{rule ? 'Next date' : 'First date'}</label>
      <input id="rstart" class="input" type="date" bind:value={start} />
      <span class="hint">Dates in the past are logged straight away.</span>
    </div>
    {#if rule}
      <label class="toggle">
        <input type="checkbox" bind:checked={active} />
        <span>Active</span>
      </label>
    {/if}
    <button class="btn primary block" disabled={!!problem}>{problem || (rule ? 'Save changes' : 'Add recurring')}</button>
    {#if rule}<button type="button" class="btn danger block" onclick={remove}>Delete recurring</button>{/if}
  </form>
</Sheet>

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 4px var(--pad) 24px;
  }
  .two {
    display: grid;
    grid-template-columns: 2fr 3fr;
    gap: 12px;
  }
  .chips.flush {
    padding: 0;
    flex-wrap: wrap;
  }
  .hint {
    font-size: 0.82rem;
    color: var(--muted);
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

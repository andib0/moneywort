<script lang="ts">
  import { api } from '$lib/api';
  import { app, category, closeSheet, currency, mutate } from '$lib/state.svelte';
  import { fromCents, moneyParts, toCents } from '$lib/fmt';
  import Sheet from './Sheet.svelte';
  import Keypad from './Keypad.svelte';

  let { categoryId, level }: { categoryId?: number; level: number } = $props();

  const existing = categoryId != null ? app.budgets.find((b) => b.category_id === categoryId) : undefined;
  let catId = $state<number | null>(categoryId ?? null);
  let amount = $state(fromCents(existing?.amount ?? 0));

  const choices = $derived(
    app.categories.filter((c) => c.kind === 'expense' && !c.archived && (c.id === categoryId || !app.budgets.some((b) => b.category_id === c.id))),
  );
  const parts = $derived(moneyParts(toCents(amount), currency()));
  const cat = $derived(category(catId));

  async function save() {
    if (catId == null || !toCents(amount)) return;
    const r = await mutate(() => api.setBudget(catId!, toCents(amount)));
    if (r) closeSheet();
  }

  async function remove() {
    await mutate(() => api.setBudget(categoryId!, 0), 'Budget removed');
    closeSheet();
  }
</script>

<Sheet title={existing ? `${cat?.name} budget` : 'New budget'} {level}>
  {#if !existing}
    <div class="chips wrap" role="radiogroup" aria-label="Category">
      {#each choices as c (c.id)}
        <button class="chip" role="radio" aria-checked={catId === c.id} onclick={() => (catId = c.id)}>{c.icon} {c.name}</button>
      {/each}
    </div>
  {/if}
  <p class="per">Monthly limit</p>
  <div class="amount" class:empty={!toCents(amount)}>
    <span class="num">{parts.main}</span><span class="frac num">{parts.frac}</span>
  </div>
  <Keypad bind:value={amount} />
  <div class="actions">
    <button class="btn primary block" disabled={catId == null || !toCents(amount)} onclick={save}>
      {catId == null ? 'Pick a category' : existing ? 'Save budget' : 'Add budget'}
    </button>
    {#if existing}<button class="btn danger block" onclick={remove}>Remove budget</button>{/if}
  </div>
</Sheet>

<style>
  .wrap {
    flex-wrap: wrap;
  }
  .per {
    text-align: center;
    color: var(--muted);
    margin: 18px 0 0;
    font-size: 0.9rem;
  }
  .amount {
    text-align: center;
    font-family: var(--display);
    font-weight: 600;
    font-size: 3.2rem;
    letter-spacing: -0.03em;
    padding: 2px 0 14px;
  }
  .amount.empty {
    color: var(--faint);
  }
  .frac {
    font-size: 1.4rem;
    color: var(--muted);
  }
  .actions {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px var(--pad) 20px;
  }
</style>

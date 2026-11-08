<script lang="ts">
  import type { Tx } from '$lib/types';
  import { account, app, category, currency, openSheet } from '$lib/state.svelte';
  import { dayLabel, money } from '$lib/fmt';
  import Icon from './Icon.svelte';

  let { items, showDayTotals = true }: { items: Tx[]; showDayTotals?: boolean } = $props();

  const groups = $derived.by(() => {
    const out: { date: string; items: Tx[]; net: number }[] = [];
    for (const t of items) {
      let g = out[out.length - 1];
      if (!g || g.date !== t.date) {
        g = { date: t.date, items: [], net: 0 };
        out.push(g);
      }
      g.items.push(t);
      if (t.kind === 'expense') g.net -= t.amount;
      if (t.kind === 'income') g.net += t.amount;
    }
    return out;
  });

  function title(t: Tx): string {
    if (t.kind === 'transfer') return `${account(t.account_id)?.name ?? '?'} → ${account(t.to_account_id)?.name ?? '?'}`;
    return category(t.category_id)?.name ?? 'Uncategorised';
  }

  function sub(t: Tx): string {
    const parts: string[] = [];
    if (t.note) parts.push(t.note);
    if (t.kind !== 'transfer' && app.accounts.length > 1) parts.push(account(t.account_id)?.name ?? '');
    return parts.filter(Boolean).join(', ');
  }
</script>

{#each groups as g (g.date)}
  <section>
    <div class="day">
      <span>{dayLabel(g.date)}</span>
      {#if showDayTotals && g.net !== 0}
        <span class="num">{money(g.net, currency(), { sign: true })}</span>
      {/if}
    </div>
    <ul class="list">
      {#each g.items as t (t.id)}
        {@const c = category(t.category_id)}
        <li>
          <button class="row" onclick={() => openSheet({ type: 'tx', tx: t })}>
            {#if t.kind === 'transfer'}
              <span class="glyph"><Icon name="swap" size={19} /></span>
            {:else}
              <span class="glyph" style:--c={c?.color}>{c?.icon ?? '•'}</span>
            {/if}
            <span class="grow">
              <span class="title">{title(t)}</span>
              {#if sub(t)}<span class="sub">{sub(t)}</span>{/if}
            </span>
            {#if t.recurring_id}<span class="rec" title="Recurring"><Icon name="repeat" size={14} /></span>{/if}
            <span class="amt num" class:in={t.kind === 'income'} class:xfer={t.kind === 'transfer'}>
              {t.kind === 'income' ? '+' : t.kind === 'expense' ? '−' : ''}{money(t.amount, currency())}
            </span>
          </button>
        </li>
      {/each}
    </ul>
  </section>
{/each}

<style>
  .day {
    display: flex;
    justify-content: space-between;
    padding: 18px var(--pad) 4px;
    font-size: 0.84rem;
    font-weight: 600;
    color: var(--muted);
  }
  .grow {
    display: flex;
    flex-direction: column;
  }
  .amt {
    font-weight: 600;
    white-space: nowrap;
  }
  .amt.in {
    color: var(--mint);
  }
  .amt.xfer {
    color: var(--muted);
  }
  .rec {
    color: var(--faint);
    display: grid;
  }
</style>

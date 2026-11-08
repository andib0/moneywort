<script lang="ts">
  import { api } from '$lib/api';
  import { account, app, category, currency, openSheet } from '$lib/state.svelte';
  import { dayLabel, money, monthName, pct, shortDate, thisMonth } from '$lib/fmt';
  import type { Summary } from '$lib/types';
  import Icon from '$lib/components/Icon.svelte';

  let s = $state<Summary | null>(null);
  const month = thisMonth();

  $effect(() => {
    void app.version;
    api.summary(month).then((r) => (s = r));
  });

  const rows = $derived(
    app.budgets
      .map((b) => {
        const cat = category(b.category_id);
        const spent = s?.by_category.find((c) => c.category_id === b.category_id)?.total ?? 0;
        return { ...b, cat, spent, left: b.amount - spent };
      })
      .filter((r) => r.cat && !r.cat.archived)
      .sort((a, b) => b.spent / b.amount - a.spent / a.amount),
  );

  const total = $derived(rows.reduce((a, r) => a + r.amount, 0));
  const totalSpent = $derived(rows.reduce((a, r) => a + r.spent, 0));
  const hasFreeCategories = $derived(
    app.categories.some((c) => c.kind === 'expense' && !c.archived && !app.budgets.some((b) => b.category_id === c.id)),
  );

  const intervals = { weekly: 'Every week', monthly: 'Every month', yearly: 'Every year' } as const;
  const monthlyRecurring = $derived(
    app.recurring
      .filter((r) => r.active && r.kind === 'expense')
      .reduce((a, r) => a + (r.interval === 'weekly' ? Math.round((r.amount * 52) / 12) : r.interval === 'yearly' ? Math.round(r.amount / 12) : r.amount), 0),
  );
</script>

<header class="top">
  <h1>Plan</h1>
</header>

<div class="section-head">
  <h2>Budgets for {monthName(month)}</h2>
  {#if hasFreeCategories}
    <button class="link" onclick={() => openSheet({ type: 'budget' })}>Add budget</button>
  {/if}
</div>

{#if rows.length}
  <div class="overall">
    <div class="line num">
      <span><b>{money(totalSpent, currency())}</b> of {money(total, currency())}</span>
      <span class:over={totalSpent > total}>{totalSpent > total ? `${money(totalSpent - total, currency())} over` : `${money(total - totalSpent, currency())} left`}</span>
    </div>
    <span class="bar thick" style:--c={totalSpent > total ? 'var(--rust)' : 'var(--mint)'}><i style:width="{Math.min(100, pct(totalSpent, total))}%"></i></span>
  </div>
  <ul class="list">
    {#each rows as r (r.category_id)}
      {@const over = r.left < 0}
      <li>
        <button class="row" onclick={() => openSheet({ type: 'budget', categoryId: r.category_id })}>
          <span class="glyph" style:--c={r.cat?.color}>{r.cat?.icon}</span>
          <span class="grow">
            <span class="line">
              <span class="title">{r.cat?.name}</span>
              <span class="num state" class:over>{over ? `${money(-r.left, currency())} over` : `${money(r.left, currency())} left`}</span>
            </span>
            <span class="bar" style:--c={over ? 'var(--rust)' : r.spent / r.amount > 0.85 ? '#c9a227' : r.cat?.color}><i style:width="{Math.min(100, pct(r.spent, r.amount))}%"></i></span>
            <span class="sub num">{money(r.spent, currency())} of {money(r.amount, currency())}</span>
          </span>
        </button>
      </li>
    {/each}
  </ul>
{:else}
  <div class="empty">
    <strong>Set a monthly limit</strong>
    Pick a category like groceries and Moneywort tracks how much is left as you spend.
    <div class="cta"><button class="btn primary" onclick={() => openSheet({ type: 'budget' })}>Add budget</button></div>
  </div>
{/if}

<div class="section-head">
  <h2>Recurring</h2>
  <button class="link" onclick={() => openSheet({ type: 'recurring' })}>Add recurring</button>
</div>
{#if app.recurring.length}
  {#if monthlyRecurring}
    <p class="hint">Fixed costs come to about <b class="num">{money(monthlyRecurring, currency())}</b> a month.</p>
  {/if}
  <ul class="list">
    {#each app.recurring as r (r.id)}
      {@const cat = category(r.category_id)}
      <li>
        <button class="row" class:paused={!r.active} onclick={() => openSheet({ type: 'recurring', rule: r })}>
          <span class="glyph" style:--c={cat?.color}>{cat?.icon ?? '•'}</span>
          <span class="grow">
            <span class="title">{r.note || cat?.name || 'Recurring'}</span>
            <span class="sub">{[
              intervals[r.interval],
              r.active ? `next ${dayLabel(r.next_date) === 'Today' ? 'today' : shortDate(r.next_date)}` : 'paused',
              app.accounts.length > 1 ? account(r.account_id)?.name : '',
            ].filter(Boolean).join(', ')}</span>
          </span>
          <span class="num amt" class:in={r.kind === 'income'}>{r.kind === 'income' ? '+' : '−'}{money(r.amount, currency())}</span>
        </button>
      </li>
    {/each}
  </ul>
{:else}
  <div class="empty">
    <strong>Rent, salary, subscriptions</strong>
    Add anything that repeats and Moneywort logs it on the right day automatically.
  </div>
{/if}

<style>
  .top {
    padding: 14px var(--pad) 4px;
    min-height: 60px;
    display: flex;
    align-items: center;
  }
  h1 {
    font-size: 1.7rem;
  }
  .overall {
    padding: 0 var(--pad) 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .line {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    align-items: baseline;
  }
  .overall .line {
    color: var(--muted);
  }
  .overall b {
    color: var(--ink);
    font-family: var(--display);
    font-size: 1.3rem;
  }
  .bar.thick {
    height: 10px;
  }
  .over {
    color: var(--rust) !important;
  }
  .grow {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .state {
    font-size: 0.9rem;
    font-weight: 600;
    white-space: nowrap;
  }
  .amt {
    font-weight: 600;
    white-space: nowrap;
  }
  .amt.in {
    color: var(--mint);
  }
  .paused {
    opacity: 0.5;
  }
  .hint {
    margin: 0 var(--pad) 6px;
    color: var(--muted);
    font-size: 0.9rem;
  }
  .hint b {
    color: var(--ink);
  }
  .cta {
    margin-top: 16px;
  }
</style>

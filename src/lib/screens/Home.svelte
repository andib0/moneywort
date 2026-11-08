<script lang="ts">
  import { api } from '$lib/api';
  import { account, app, category, currency, openSheet, setTab } from '$lib/state.svelte';
  import { addMonths, money, moneyParts, monthName, pct, thisMonth } from '$lib/fmt';
  import type { Summary, Tx } from '$lib/types';
  import MonthSwitch from '$lib/components/MonthSwitch.svelte';
  import TxList from '$lib/components/TxList.svelte';
  import Icon from '$lib/components/Icon.svelte';

  let summary = $state<Summary | null>(null);
  let recent = $state<Tx[]>([]);

  $effect(() => {
    const m = app.month;
    void app.version;
    api.summary(m).then((s) => (summary = s));
    api.transactions({ month: m, limit: 8 }).then((t) => (recent = t));
  });

  const spent = $derived(moneyParts(summary?.expense ?? 0, currency()));
  const budgetTotal = $derived(app.budgets.reduce((s, b) => s + b.amount, 0));
  // Only spending in budgeted categories counts against the budget.
  const budgetSpent = $derived(
    (summary?.by_category ?? [])
      .filter((c) => app.budgets.some((b) => b.category_id === c.category_id))
      .reduce((s, c) => s + c.total, 0),
  );
  const isNow = $derived(app.month === thisMonth());

  const ribbon = $derived.by(() => {
    if (!summary || !summary.expense) return [];
    const top = summary.by_category.slice(0, 5);
    const rest = summary.by_category.slice(5).reduce((s, c) => s + c.total, 0);
    const out = top.map((c) => {
      const cat = category(c.category_id);
      return { id: String(c.category_id), name: cat?.name ?? 'Uncategorised', color: cat?.color ?? '#8d8d8d', total: c.total };
    });
    if (rest > 0) out.push({ id: 'rest', name: 'Everything else', color: 'var(--faint)', total: rest });
    return out;
  });

  const compare = $derived.by(() => {
    if (!summary || !summary.prev_expense || !summary.expense) return '';
    const diff = summary.expense - summary.prev_expense;
    const p = Math.abs(pct(diff, summary.prev_expense));
    const prev = monthName(addMonths(app.month, -1));
    if (p < 2) return `About the same as ${prev}`;
    return `${p}% ${diff < 0 ? 'less' : 'more'} than ${prev}${isNow ? ' so far' : ''}`;
  });

  const accounts = $derived(app.accounts.filter((a) => !a.archived));
  const balanceOf = (id: number) => summary?.balances.find((b) => b.account_id === id)?.balance ?? 0;
</script>

<header class="top">
  <MonthSwitch />
  <button class="icon-btn" onclick={() => openSheet({ type: 'settings' })} aria-label="Settings">
    <Icon name="gear" />
  </button>
</header>

<section class="hero">
  <p class="label">{isNow ? 'Spent this month' : `Spent in ${monthName(app.month)}`}</p>
  <p class="figure num" aria-label={money(summary?.expense ?? 0, currency())}>
    {spent.main}<span class="frac">{spent.frac}</span>
  </p>
  <p class="context">
    {#if budgetTotal > 0}
      {@const left = budgetTotal - budgetSpent}
      {#if left >= 0}
        <span>{money(left, currency())} left</span> in your {money(budgetTotal, currency())} of budgets
      {:else}
        <span class="over">{money(-left, currency())} over</span> your {money(budgetTotal, currency())} of budgets
      {/if}
    {:else}
      {money(summary?.income ?? 0, currency())} came in
    {/if}
  </p>

  {#if ribbon.length}
    <div class="ribbon" role="img" aria-label="Spending by category">
      {#each ribbon as r (r.id)}
        <i style:flex-grow={r.total} style:background={r.color}></i>
      {/each}
    </div>
    <ul class="legend">
      {#each ribbon.slice(0, 4) as r (r.id)}
        <li><i style:background={r.color}></i>{r.name} <span class="num">{pct(r.total, summary!.expense)}%</span></li>
      {/each}
    </ul>
  {/if}
  {#if compare}<p class="compare">{compare}</p>{/if}
</section>

<div class="section-head">
  <h2>Accounts</h2>
  <span class="muted num">Total {money(summary?.net_worth ?? 0, currency())}</span>
</div>
<div class="accounts chips">
  {#each accounts as a (a.id)}
    <button class="acct" onclick={() => openSheet({ type: 'account', account: a })}>
      <span class="aname"><i style:background={a.color}></i>{a.name}</span>
      <span class="abal num" class:neg={balanceOf(a.id) < 0}>{money(balanceOf(a.id), currency())}</span>
    </button>
  {/each}
  <button class="acct add" onclick={() => openSheet({ type: 'account' })} aria-label="Add account">
    <Icon name="plus" size={20} />
  </button>
</div>

<div class="section-head">
  <h2>Latest</h2>
  {#if recent.length}<button class="link" onclick={() => setTab('activity')}>See all</button>{/if}
</div>
{#if recent.length}
  <TxList items={recent} showDayTotals={false} />
{:else}
  <div class="empty">
    <strong>Nothing logged in {monthName(app.month)}</strong>
    Tap + to add your first expense or income.
  </div>
{/if}

<style>
  .top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 8px 0 var(--pad);
  }
  .hero {
    padding: 18px var(--pad) 4px;
  }
  .label {
    margin: 0;
    color: var(--muted);
    font-weight: 550;
  }
  .figure {
    margin: 2px 0 6px;
    font-family: var(--display);
    font-size: clamp(3.2rem, 17vw, 5rem);
    font-weight: 650;
    letter-spacing: -0.045em;
    line-height: 0.95;
    font-variation-settings: 'opsz' 96;
  }
  .frac {
    font-size: 0.4em;
    letter-spacing: -0.01em;
    color: var(--muted);
    margin-left: 2px;
  }
  .context {
    margin: 0;
    color: var(--muted);
  }
  .context span {
    color: var(--ink);
    font-weight: 600;
  }
  .context span.over {
    color: var(--rust);
  }
  .ribbon {
    display: flex;
    gap: 3px;
    height: 14px;
    margin: 22px 0 12px;
  }
  .ribbon i {
    flex-basis: 0;
    min-width: 6px;
    border-radius: 4px;
  }
  .legend {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    font-size: 0.86rem;
  }
  .legend li {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .legend i {
    width: 8px;
    height: 8px;
    border-radius: 2px;
  }
  .legend span {
    color: var(--muted);
  }
  .compare {
    margin: 14px 0 0;
    font-size: 0.86rem;
    color: var(--muted);
  }
  .accounts {
    padding-bottom: 4px;
  }
  .acct {
    flex: none;
    min-width: 132px;
    padding: 12px 14px;
    border-radius: var(--r-md);
    background: var(--surface);
    border: 1px solid var(--line);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
    text-align: left;
  }
  .acct:active {
    background: var(--sunk);
  }
  .aname {
    font-size: 0.86rem;
    color: var(--muted);
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .aname i {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .abal {
    font-family: var(--display);
    font-weight: 600;
    font-size: 1.15rem;
  }
  .abal.neg {
    color: var(--rust);
  }
  .acct.add {
    min-width: 56px;
    align-items: center;
    justify-content: center;
    color: var(--muted);
    background: transparent;
    border-style: dashed;
  }
</style>

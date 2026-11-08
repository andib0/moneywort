<script lang="ts">
  import { api } from '$lib/api';
  import { app, category, currency } from '$lib/state.svelte';
  import { daysInMonth, money, monthName, pct, thisMonth, todayISO } from '$lib/fmt';
  import type { Summary } from '$lib/types';
  import MonthSwitch from '$lib/components/MonthSwitch.svelte';

  let s = $state<Summary | null>(null);

  $effect(() => {
    const m = app.month;
    void app.version;
    api.summary(m).then((r) => (s = r));
  });

  const saved = $derived((s?.income ?? 0) - (s?.expense ?? 0));
  const rate = $derived(s && s.income > 0 ? Math.round((saved / s.income) * 100) : null);

  const trendMax = $derived(Math.max(1, ...(s?.trend ?? []).flatMap((t) => [t.income, t.expense])));

  const days = $derived.by(() => {
    if (!s) return [];
    const n = daysInMonth(app.month);
    const map = new Map(s.daily.map((d) => [d.key, d.expense]));
    return Array.from({ length: n }, (_, i) => {
      const key = `${app.month}-${String(i + 1).padStart(2, '0')}`;
      return { key, day: i + 1, v: map.get(key) ?? 0 };
    });
  });
  const dayMax = $derived(Math.max(1, ...days.map((d) => d.v)));
  const elapsed = $derived(app.month === thisMonth() ? Number(todayISO().slice(8)) : days.length);
  const perDay = $derived(elapsed ? Math.round((s?.expense ?? 0) / elapsed) : 0);
  // Spent so far plus a typical (median) day for each remaining day, so one-off
  // big payments like rent don't get multiplied across the month.
  const projected = $derived.by(() => {
    if (app.month !== thisMonth() || elapsed < 5) return 0;
    const past = days.slice(0, elapsed).map((d) => d.v).sort((a, b) => a - b);
    const mid = Math.floor(past.length / 2);
    const median = past.length % 2 ? past[mid] : Math.round((past[mid - 1] + past[mid]) / 2);
    return (s?.expense ?? 0) + median * (days.length - elapsed);
  });
</script>

<header class="top">
  <h1>Insights</h1>
  <MonthSwitch />
</header>

{#if s}
  <section class="stats">
    <div>
      <span class="k">Came in</span>
      <span class="v num in">{money(s.income, currency())}</span>
    </div>
    <div>
      <span class="k">Went out</span>
      <span class="v num">{money(s.expense, currency())}</span>
    </div>
    <div>
      <span class="k">{saved >= 0 ? 'Kept' : 'Overspent'}</span>
      <span class="v num" class:over={saved < 0}>{money(Math.abs(saved), currency())}</span>
      {#if rate !== null && saved > 0}<span class="hint">{rate}% of income</span>{/if}
    </div>
  </section>

  <div class="section-head"><h2>Last six months</h2></div>
  <div class="trend" role="img" aria-label="Income and spending, last six months">
    {#each s.trend as t (t.key)}
      <button class="col" class:sel={t.key === app.month} onclick={() => (app.month = t.key)} aria-label={`${monthName(t.key)}: in ${money(t.income, currency())}, out ${money(t.expense, currency())}`}>
        <span class="bars">
          <i class="inc" style:height="{(t.income / trendMax) * 100}%"></i>
          <i class="exp" style:height="{(t.expense / trendMax) * 100}%"></i>
        </span>
        <span class="m">{monthName(t.key, 'short')}</span>
      </button>
    {/each}
  </div>
  <div class="key"><span><i class="inc"></i>Came in</span><span><i class="exp"></i>Went out</span></div>

  <div class="section-head">
    <h2>Day by day</h2>
    <span class="muted num">{money(perDay, currency())} a day</span>
  </div>
  <div class="daily" role="img" aria-label="Spending per day">
    {#each days as d (d.key)}
      <i style:height="{Math.max(d.v ? 4 : 2, (d.v / dayMax) * 100)}%" class:zero={!d.v}></i>
    {/each}
  </div>
  {#if projected}
    <p class="note">At this pace you'll spend about <b class="num">{money(projected, currency())}</b> by month end.</p>
  {/if}

  <div class="section-head"><h2>Where it went</h2></div>
  {#if s.by_category.length}
    <ul class="list">
      {#each s.by_category as c (c.category_id)}
        {@const cat = category(c.category_id)}
        <li class="row static">
          <span class="glyph" style:--c={cat?.color}>{cat?.icon ?? '•'}</span>
          <span class="grow">
            <span class="line">
              <span class="title">{cat?.name ?? 'Uncategorised'}</span>
              <span class="num amt">{money(c.total, currency())}</span>
            </span>
            <span class="bar" style:--c={cat?.color}><i style:width="{pct(c.total, s.by_category[0].total)}%"></i></span>
            <span class="sub">{pct(c.total, s.expense)}% of spending, {c.count} {c.count === 1 ? 'entry' : 'entries'}</span>
          </span>
        </li>
      {/each}
    </ul>
  {:else}
    <div class="empty"><strong>No spending in {monthName(app.month)}</strong>Your breakdown appears here once you add expenses.</div>
  {/if}
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
  .stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
    padding: 16px var(--pad) 0;
  }
  .stats div {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .k {
    font-size: 0.84rem;
    color: var(--muted);
  }
  .v {
    font-family: var(--display);
    font-weight: 600;
    font-size: 1.2rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .v.in {
    color: var(--mint);
  }
  .v.over {
    color: var(--rust);
  }
  .hint {
    font-size: 0.78rem;
    color: var(--muted);
  }
  .trend {
    display: flex;
    gap: 6px;
    height: 160px;
    padding: 0 var(--pad);
  }
  .col {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    border-radius: 12px;
    padding: 8px 0 6px;
  }
  .col.sel {
    background: var(--sunk);
  }
  .bars {
    flex: 1;
    display: flex;
    align-items: flex-end;
    gap: 3px;
    width: 100%;
    justify-content: center;
  }
  .bars i {
    width: 34%;
    max-width: 16px;
    border-radius: 4px 4px 2px 2px;
    min-height: 2px;
    transition: height 0.35s ease;
  }
  .inc {
    background: var(--mint);
  }
  .exp {
    background: var(--ink);
  }
  .col:not(.sel) .bars i {
    opacity: 0.45;
  }
  .m {
    font-size: 0.78rem;
    color: var(--muted);
  }
  .col.sel .m {
    color: var(--ink);
    font-weight: 600;
  }
  .key {
    display: flex;
    gap: 16px;
    padding: 10px var(--pad) 0;
    font-size: 0.82rem;
    color: var(--muted);
  }
  .key span {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .key i {
    width: 8px;
    height: 8px;
    border-radius: 2px;
  }
  .daily {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    height: 80px;
    padding: 0 var(--pad);
  }
  .daily i {
    flex: 1;
    background: var(--ink);
    border-radius: 2px;
    opacity: 0.8;
  }
  .daily i.zero {
    background: var(--line);
    opacity: 1;
  }
  .note {
    margin: 12px var(--pad) 0;
    font-size: 0.9rem;
    color: var(--muted);
  }
  .note b {
    color: var(--ink);
  }
  .row.static:active {
    background: none;
  }
  .grow {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .line {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }
  .amt {
    font-weight: 600;
  }
</style>

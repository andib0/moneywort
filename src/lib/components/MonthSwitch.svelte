<script lang="ts">
  import { app } from '$lib/state.svelte';
  import { addMonths, monthName, thisMonth } from '$lib/fmt';
  import Icon from './Icon.svelte';

  const atNow = $derived(app.month >= thisMonth());
</script>

<div class="ms">
  <button class="icon-btn" onclick={() => (app.month = addMonths(app.month, -1))} aria-label="Previous month">
    <Icon name="left" size={20} />
  </button>
  <button class="label" onclick={() => (app.month = thisMonth())} aria-label="Jump to this month">
    {monthName(app.month)}
  </button>
  <button
    class="icon-btn"
    onclick={() => (app.month = addMonths(app.month, 1))}
    disabled={atNow}
    aria-label="Next month"
  >
    <Icon name="right" size={20} />
  </button>
</div>

<style>
  .ms {
    display: inline-flex;
    align-items: center;
    margin-left: -12px;
  }
  .label {
    font-weight: 600;
    min-width: 92px;
    padding: 8px 2px;
  }
  .icon-btn:disabled {
    opacity: 0.25;
  }
</style>

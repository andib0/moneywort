<script lang="ts">
  import { closeSheet } from '$lib/state.svelte';
  import Sheet from './Sheet.svelte';

  let {
    title,
    body,
    action,
    danger = true,
    onyes,
    level,
  }: { title: string; body: string; action: string; danger?: boolean; onyes: () => void; level: number } = $props();

  let busy = $state(false);

  async function yes() {
    if (busy) return;
    busy = true;
    await closeSheet();
    onyes();
  }
</script>

<Sheet {title} {level}>
  <div class="c">
    <p>{body}</p>
    <button class="btn block" class:danger class:primary={!danger} onclick={yes} disabled={busy}>{action}</button>
    <button class="btn block" onclick={closeSheet}>Cancel</button>
  </div>
</Sheet>

<style>
  .c {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 0 var(--pad) 20px;
  }
  p {
    margin: 0 0 12px;
    color: var(--muted);
  }
</style>

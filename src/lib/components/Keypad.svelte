<script lang="ts">
  import Icon from './Icon.svelte';

  let { value = $bindable('') }: { value: string } = $props();

  const keys = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '.', '0', '⌫'];

  function press(k: string) {
    if (navigator.vibrate) navigator.vibrate(6);
    if (k === '⌫') {
      value = value.slice(0, -1);
      return;
    }
    if (k === '.') {
      if (value.includes('.')) return;
      value = (value || '0') + '.';
      return;
    }
    const [w, f] = value.split('.');
    if (f !== undefined && f.length >= 2) return;
    if (f === undefined && w.length >= 9) return;
    value = value === '0' ? k : value + k;
  }
</script>

<div class="pad" role="group" aria-label="Amount keypad">
  {#each keys as k (k)}
    <button
      class:fn={k === '⌫' || k === '.'}
      onclick={() => press(k)}
      oncontextmenu={(e) => {
        if (k === '⌫') {
          e.preventDefault();
          value = '';
        }
      }}
      aria-label={k === '⌫' ? 'Delete digit (hold to clear)' : k === '.' ? 'Decimal point' : k}
    >
      {#if k === '⌫'}<Icon name="erase" size={24} />{:else}{k}{/if}
    </button>
  {/each}
</div>

<style>
  .pad {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
    padding: 0 var(--pad);
  }
  button {
    height: 54px;
    border-radius: 14px;
    font-family: var(--display);
    font-size: 1.5rem;
    font-weight: 500;
    display: grid;
    place-items: center;
    transition: background 0.08s;
  }
  button:active {
    background: var(--sunk);
  }
  .fn {
    color: var(--muted);
  }
  @media (max-height: 700px) {
    button {
      height: 46px;
    }
  }
</style>

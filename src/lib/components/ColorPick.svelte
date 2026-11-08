<script lang="ts">
  let { value = $bindable() }: { value: string } = $props();
  const colors = ['#5b8c5a', '#2f7d5b', '#3a9a9a', '#4a7bb7', '#6b7a8f', '#8a6bbf', '#c2577a', '#e05d5d', '#d1763a', '#c9a227', '#8d8d8d', '#3d4a42'];
</script>

<div class="colors" role="radiogroup" aria-label="Colour">
  {#each colors as c (c)}
    <button type="button" role="radio" aria-checked={value === c} aria-label={c} style:background={c} onclick={() => (value = c)}></button>
  {/each}
</div>

<style>
  .colors {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 10px;
  }
  button {
    aspect-ratio: 1;
    border-radius: 50%;
    max-width: 44px;
    justify-self: center;
    width: 100%;
    outline-offset: 3px;
  }
  button[aria-checked='true'] {
    outline: 3px solid var(--ink);
  }
</style>

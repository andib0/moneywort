<script lang="ts">
  import type { Category, CatKind } from '$lib/types';
  import { api } from '$lib/api';
  import { closeSheet, confirmAction, mutate, toast } from '$lib/state.svelte';
  import Sheet from './Sheet.svelte';
  import ColorPick from './ColorPick.svelte';

  let { category, kind: k, level }: { category?: Category; kind?: CatKind; level: number } = $props();

  const icons = ['🛒', '🍜', '☕', '🍺', '🚌', '🚗', '⛽', '✈️', '🏠', '💡', '📱', '🌐', '🛍️', '👕', '💊', '🏋️', '🎬', '🎮', '🎵', '📚', '🎓', '👶', '🐾', '🎁', '💼', '🧑‍💻', '💰', '📈', '🔁', '🧾', '🔧', '💇', '❤️', '🌱', '➕', '•'];

  let name = $state(category?.name ?? '');
  let kind = $state<CatKind>(category?.kind ?? k ?? 'expense');
  let icon = $state(category?.icon ?? '🛒');
  let color = $state(category?.color ?? '#5b8c5a');
  let archived = $state(category?.archived ?? false);

  async function save() {
    if (!name.trim()) return;
    const ok = await mutate(() =>
      api.saveCategory({ id: category?.id ?? 0, name, kind, icon, color, archived, sort: category?.sort ?? 0 }),
    );
    if (ok) closeSheet();
  }

  function remove() {
    confirmAction(
      `Delete ${category!.name}?`,
      'If entries use this category it will be hidden instead, so past entries keep their label. Its budget is removed.',
      'Delete',
      async () => {
        const r = await mutate(() => api.deleteCategory(category!.id));
        if (r) {
          toast(r.value ? `${category!.name} hidden, entries still use it` : `${category!.name} deleted`);
          closeSheet();
        }
      },
    );
  }
</script>

<Sheet title={category ? 'Edit category' : 'New category'} {level}>
  <form class="form" onsubmit={(e) => { e.preventDefault(); save(); }}>
    <div class="preview">
      <span class="glyph big" style:--c={color}>{icon}</span>
      <input class="input" bind:value={name} placeholder="Category name" maxlength="30" aria-label="Name" />
    </div>
    {#if !category}
      <div class="seg" role="group" aria-label="Category type">
        <button type="button" aria-pressed={kind === 'expense'} onclick={() => (kind = 'expense')}>Expense</button>
        <button type="button" aria-pressed={kind === 'income'} onclick={() => (kind = 'income')}>Income</button>
      </div>
    {/if}
    <div class="field">
      <span class="label">Icon</span>
      <div class="icons" role="radiogroup" aria-label="Icon">
        {#each icons as i (i)}
          <button type="button" role="radio" aria-checked={icon === i} onclick={() => (icon = i)}>{i}</button>
        {/each}
      </div>
    </div>
    <div class="field">
      <span class="label">Colour</span>
      <ColorPick bind:value={color} />
    </div>
    {#if category}
      <label class="toggle">
        <input type="checkbox" bind:checked={archived} />
        <span>Hide when adding entries</span>
      </label>
    {/if}
    <button class="btn primary block" disabled={!name.trim()}>{category ? 'Save changes' : 'Add category'}</button>
    {#if category}
      <button type="button" class="btn danger block" onclick={remove}>Delete category</button>
    {/if}
  </form>
</Sheet>

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 4px var(--pad) 24px;
  }
  .preview {
    display: flex;
    gap: 12px;
    align-items: center;
  }
  .glyph.big {
    width: 50px;
    height: 50px;
    font-size: 1.4rem;
    border-radius: 16px;
  }
  .icons {
    display: grid;
    grid-template-columns: repeat(9, 1fr);
    gap: 4px;
  }
  .icons button {
    aspect-ratio: 1;
    border-radius: 10px;
    font-size: 1.2rem;
    display: grid;
    place-items: center;
  }
  .icons button[aria-checked='true'] {
    background: var(--sunk);
    outline: 2px solid var(--ink);
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

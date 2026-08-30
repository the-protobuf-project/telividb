<!--
  A search box for one list.

  Filters as you type rather than on submit: these lists are small and local, so
  there is nothing to wait for, and a search that needs Enter makes the reader
  guess whether it ran.

  It states how many rows matched. "No results" and "no results *for this
  query*" are the same words until the count is there.
-->
<script lang="ts">
  interface Props {
    /** What is typed. Bindable. */
    value?: string;
    /** What is being searched, for the placeholder and the label. */
    noun: string;
    /** How many rows match, and how many there are in total. */
    matched?: number;
    /** The total, when a query is narrowing it. */
    total?: number;
  }

  let { value = $bindable(""), noun, matched, total }: Props = $props();
</script>

<div style="display: flex; align-items: center; gap: calc(var(--u) * 3)">
  <input
    class="input"
    type="search"
    bind:value
    placeholder="Search {noun}…"
    aria-label="Search {noun}"
    style="width: 16rem"
  />
  {#if matched !== undefined && total !== undefined}
    <span class="hint">
      {#if value.trim()}
        {matched} of {total}
      {:else}
        {total}
        {total === 1 ? noun.replace(/s$/, "") : noun}
      {/if}
    </span>
  {/if}
</div>

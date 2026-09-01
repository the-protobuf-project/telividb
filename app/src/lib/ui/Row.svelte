<!--
  A selectable list row: a title with badges, a line or two of detail, and
  controls on the right.

  The list primitive of this design — models, people, collections are all this
  shape. Selection is `aria-pressed` rather than a class so the state is real
  rather than visual.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    /** The row's name. */
    name: string;
    /** Whether it is the chosen one. */
    selected?: boolean;
    /** Dimmed, for a soft-deleted or unavailable row. */
    muted?: boolean;
    /** Clicking selects. Omit for a row that is not selectable. */
    onclick?: () => void;
    /** Badges beside the name. */
    badges?: Snippet;
    /** Detail lines under it. */
    meta?: Snippet;
    /** Controls at the right-hand end. */
    action?: Snippet;
  }

  let { name, selected, muted, onclick, badges, meta, action }: Props = $props();
</script>

{#snippet body()}
  <div class="row-main">
    <div class="row-title">
      <span class="row-name">{name}</span>
      {#if badges}{@render badges()}{/if}
    </div>
    {#if meta}{@render meta()}{/if}
  </div>
  {#if action}
    <div style="display: flex; gap: 0.5rem; align-items: center; flex: none">
      {@render action()}
    </div>
  {/if}
{/snippet}

<!-- A row that does something is a button; one that does not is not pretending
     to be. Keyboard operation and the pressed state come free from the element
     rather than from a role and a handler that have to agree. -->
{#if onclick}
  <button class="row" type="button" aria-pressed={selected} style={muted ? "opacity:.5" : ""} {onclick}>
    {@render body()}
  </button>
{:else}
  <div class="row" style={muted ? "opacity:.5" : ""}>{@render body()}</div>
{/if}

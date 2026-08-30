<!--
  A row of tiles that fills the width.

  One pixel of gap over a ruled background, so the tiles read as one instrument
  panel rather than as separate cards.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    /** The tiles. */
    children: Snippet;
  }

  let { children }: Props = $props();
</script>

<div class="tiles">{@render children()}</div>

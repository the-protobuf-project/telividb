<!--
  What a panel shows when it has nothing.

  A sentence rather than a blank: "nothing has been imported" and "the request
  failed" look identical when both render as an empty box, and only one of them
  is the reader's to act on.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    /** What is absent, and what would fill it. */
    children: Snippet;
  }

  let { children }: Props = $props();
</script>

<div class="empty">{@render children()}</div>

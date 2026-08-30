<!--
  The full-window ground the first-run card sits on.

  The faint grid is not ornament: it is the only thing on screen before any data
  exists, and it makes the window read as a workspace rather than as a failed
  page load.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    /** What sits in the middle. */
    children: Snippet;
  }

  let { children }: Props = $props();
</script>

<div class="stage">{@render children()}</div>

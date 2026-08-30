<!-- A bordered stack of setting rows, sharing one outline. -->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    /** Dimmed, for a group whose controls are not wired yet. */
    muted?: boolean;
    /** The rows. */
    children: Snippet;
  }

  let { muted = false, children }: Props = $props();
</script>

<div class="set-group" style={muted ? "opacity: 0.6" : ""}>{@render children()}</div>

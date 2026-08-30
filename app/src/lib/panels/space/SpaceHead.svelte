<!--
  What space you are in, how it is protected, and which mode you are reading it
  in.

  The protection sits beside the name rather than in a detail panel, because it
  decides what may happen to everything below it — a remote model may not answer
  from a vault, and finding that out at the moment of asking is finding out too
  late.
-->
<script lang="ts">
  import { Seg } from "$lib/ui";
  import ProtectionBadge from "$lib/panels/workspace/ProtectionBadge.svelte";
  import type { Protection } from "$lib/ui";

  interface Props {
    /** The space's display name. */
    name: string;
    /** How it is protected. */
    protection: Protection;
    /** Whether its key is currently unavailable. */
    locked?: boolean;
    /** Which mode is showing. Bindable. */
    mode?: string;
    /** A short figure at the right — points, dimensions. */
    note?: string;
  }

  let {
    name,
    protection,
    locked = false,
    mode = $bindable("Conversation"),
    note,
  }: Props = $props();
</script>

<div class="space-head">
  <h2>{name}</h2>
  <ProtectionBadge {protection} {locked} />
  <span class="spacer" style="flex: 1"></span>
  <!-- A graph is a mode of a space rather than a place of its own: the same
       points, read two ways. A separate destination would imply separate data. -->
  <Seg options={["Conversation", "Graph"]} bind:value={mode} label="How to read this space" />
  {#if note}<span class="faint mono" style="font-size: 0.75rem">{note}</span>{/if}
</div>

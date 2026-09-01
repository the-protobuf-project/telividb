<!--
  What space you are in, how it is protected, and which mode you are reading it
  in.

  The protection sits beside the name rather than in a detail panel, because it
  decides what may happen to everything below it — a remote model may not answer
  from a vault, and finding that out at the moment of asking is finding out too
  late.
-->
<script lang="ts">
  import { IconButton, Seg } from "$lib/ui";
  import ProtectionBadge from "$lib/panels/workspace/ProtectionBadge.svelte";
  import type { Protection } from "$lib/ui";

  interface Props {
    /** The space's display name. */
    name: string;
    /** Leave the space and go back to the organization. */
    onclose?: () => void;
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
    onclose,
    protection,
    locked = false,
    mode = $bindable("Conversation"),
    note,
  }: Props = $props();
</script>

<div class="space-head">
  {#if onclose}
    <!-- The way back. Without it a space was a one-way door: the centre column
         became the conversation and the projects list, with its create form,
         was unreachable except by re-clicking an already-current rail row. -->
    <IconButton label="Back to the organization" onclick={onclose}>
      <svg width="15" height="15" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.3">
        <path d="M11 4l-5 5 5 5" />
      </svg>
    </IconButton>
  {/if}
  <h2>{name}</h2>
  <ProtectionBadge {protection} {locked} />
  <span class="spacer" style="flex: 1"></span>
  <!-- A graph is a mode of a space rather than a place of its own: the same
       points, read two ways. A separate destination would imply separate data. -->
  <Seg options={["Conversation", "Graph"]} bind:value={mode} label="How to read this space" />
  {#if note}<span class="faint mono" style="font-size: 0.75rem">{note}</span>{/if}
</div>

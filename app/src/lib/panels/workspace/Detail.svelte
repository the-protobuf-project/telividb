<!--
  The right-hand detail column: what the selected organization actually holds.

  Counts come from the engine rather than from the length of what this window
  happened to fetch — a project the caller cannot see still counts, so
  recomputing here would quietly disagree with the server.
-->
<script lang="ts">
  import { Kv, PanelLabel, Seg, SideTab } from "$lib/ui";
  import type { Protection } from "$lib/ui";
  import type { WorkspaceState } from "./state.svelte";

  interface Props {
    /** The tree this panel reads. */
    tree: WorkspaceState;
  }

  let { tree }: Props = $props();

  const choices: readonly Protection[] = ["none", "private", "vault", "sealed"];
</script>

<aside class="side">
  <div class="side-tabs">
    <SideTab label="Detail" current />
  </div>

  <div class="side-body">
    {#if tree.selected}
      <PanelLabel>Organization</PanelLabel>
      <Kv label="Name" value={tree.selected.name} />
      <Kv label="Projects" value={String(tree.selected.projectCount)} />
      <Kv label="Spaces" value={String(tree.selected.spaceCount)} />
      <Kv label="State" value={tree.selected.deleted ? "deleted" : "live"} />

      <PanelLabel>New space protection</PanelLabel>
      <!-- Chosen before the name because it cannot be changed afterwards: it
           decides which segments the contents are routed to, so altering it
           later is a rewrite rather than a field update. -->
      <Seg options={choices} bind:value={tree.protection} label="Protection for a new space" />
    {:else}
      <!-- This pane holds a third of the window at the size it opens at, so a
           two-word shrug is a third of the window saying nothing. The same
           phrase in Workspace already names the next move; this one now does
           too. -->
      <p class="hint">
        Nothing selected. Choose an organization, project or space in the rail
        and what it contains appears here.
      </p>
    {/if}
  </div>
</aside>

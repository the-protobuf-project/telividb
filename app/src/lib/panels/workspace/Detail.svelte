<!--
  The right-hand detail column: what the selected organization actually holds.

  Counts come from the engine rather than from the length of what this window
  happened to fetch — a project a caller cannot see still counts, so recomputing
  here would quietly disagree with the server.
-->
<script lang="ts">
  import type { Protection } from "@telividb/answer";
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
    <div class="side-tab" aria-current="true">Detail</div>
  </div>

  <div class="side-body">
    {#if tree.selected}
      <div>
        <div class="panel-label">Organization</div>
        <div class="kv"><span>Name</span><span class="mono">{tree.selected.name}</span></div>
        <div class="kv"><span>Projects</span><span class="mono">{tree.selected.projectCount}</span></div>
        <div class="kv"><span>Spaces</span><span class="mono">{tree.selected.spaceCount}</span></div>
        <div class="kv">
          <span>State</span>
          <span class="mono">{tree.selected.deleted ? "deleted" : "live"}</span>
        </div>
      </div>

      <div>
        <div class="panel-label">New space protection</div>
        <!-- Chosen before the name because it cannot be changed afterwards: it
             decides which segments the contents are routed to, so altering it
             later is a rewrite rather than a field update. -->
        <div class="seg" style="margin-top: 0.5rem">
          {#each choices as choice (choice)}
            <button
              type="button"
              aria-pressed={tree.protection === choice}
              onclick={() => (tree.protection = choice)}
            >
              {choice}
            </button>
          {/each}
        </div>
      </div>
    {:else}
      <p class="hint">Nothing selected.</p>
    {/if}
  </div>
</aside>

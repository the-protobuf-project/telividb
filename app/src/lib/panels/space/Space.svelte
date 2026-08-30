<!--
  A space, as the surface you actually work on.

  The centre column of the workspace: what space you are in, the conversation
  within it, and the box you add to it. Reading it as a graph is a mode of the
  same space rather than a separate destination — the points are identical, only
  the arrangement differs.
-->
<script lang="ts">
  import { Empty } from "$lib/ui";
  import type { Protection } from "$lib/ui";
  import GraphEditor from "$lib/graph/GraphEditor.svelte";
  import { edges as seedEdges, nodes as seedNodes } from "$lib/graph/fixture";
  import type { AskState } from "$lib/panels/ask/state.svelte";
  import SpaceHead from "./SpaceHead.svelte";
  import Turn from "./Turn.svelte";
  import Composer from "./Composer.svelte";
  import LockedVeil from "./LockedVeil.svelte";

  interface Props {
    /** The space's display name. */
    name: string;
    /** How it is protected. */
    protection: Protection;
    /** Whether its key is unavailable this session. */
    locked?: boolean;
    /** The conversation, and the calls behind it. */
    ask: AskState;
    /** Whether text can be turned into vectors yet. */
    canEmbed?: boolean;
    /** The resident model's width, stated on each stored point. */
    dimensions?: number;
  }

  let { name, protection, locked = false, ask, canEmbed = true, dimensions }: Props = $props();

  let mode = $state("Conversation");

  // The graph runs on fixtures until `Graph.ListEdges` is served. Held here
  // rather than inside the editor so a rearranged graph survives switching to
  // the conversation and back.
  let nodes = $state(structuredClone(seedNodes));
  let edges = $state(structuredClone(seedEdges));
</script>

<div class="main">
  <SpaceHead
    {name}
    {protection}
    {locked}
    bind:mode
    note={ask.history.length > 0 ? `${ask.history.length} turns` : undefined}
  />

  {#if locked}
    <LockedVeil {name} sealed={protection === "sealed"} />
  {:else if mode === "Graph"}
    <div style="position: relative; min-height: 0">
      <GraphEditor bind:nodes bind:edges />
    </div>
  {:else}
    <div class="thread">
      <div class="thread-inner">
        {#each ask.history as turn (turn.question + turn.hits.length)}
          <Turn {turn} {dimensions} />
        {:else}
          <Empty>
            Nothing asked yet. The first sentence has nothing to match against —
            the second is where it starts being interesting.
          </Empty>
        {/each}
      </div>
    </div>

    <Composer
      bind:draft={ask.draft}
      providers={ask.providers}
      bind:provider={ask.provider}
      bind:model={ask.model}
      busy={ask.asking}
      {canEmbed}
      onsend={() => ask.ask()}
    />
  {/if}
</div>

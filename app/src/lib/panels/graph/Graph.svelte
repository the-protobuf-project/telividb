<!--
  The graph, as a panel.

  On fixtures, and it says so. `Graph.ListEdges` is not served — the protos exist
  and nothing implements them — so the canvas is seeded from a module rather than
  from the engine. That is marked on the face of the panel rather than left to be
  discovered when an edge fails to save.

  It sits in the dock for now. In the design a graph is a *mode of a space*,
  reached by a toggle beside the thread; that arrangement waits on the workspace
  becoming the working surface it is meant to be.
-->
<script lang="ts">
  import { Kv, Notice, PanelLabel, Tag } from "$lib/ui";
  import GraphEditor from "$lib/graph/GraphEditor.svelte";
  import { edges as seedEdges, nodes as seedNodes } from "$lib/graph/fixture";

  let nodes = $state(structuredClone(seedNodes));
  let edges = $state(structuredClone(seedEdges));
  let threshold = $state(0);
  let picked = $state<string | null>(null);
  let took = $state<string | null>(null);

  let chosen = $derived(nodes.find((n) => n.id === picked));
  let neighbours = $derived(
    edges.filter((e) => e.source === picked || e.target === picked),
  );
  let hidden = $derived(
    edges.length -
      edges.filter((e) => ((e.data?.["weight"] as number) ?? 1) >= threshold).length,
  );

  /**
   * What an action means.
   *
   * Hide and inspect act for real, because they only rearrange what is already
   * on the canvas. Expand and open need the `Graph` service and the points
   * behind it, so they report rather than pretending.
   */
  function act(action: string, id: string) {
    took = `${action} · ${id}`;
    if (action === "hide") {
      nodes = nodes.filter((n) => n.id !== id);
      edges = edges.filter((e) => e.source !== id && e.target !== id);
      if (picked === id) picked = null;
    } else if (action === "inspect") {
      picked = id;
    } else if (action === "pin") {
      nodes = nodes.map((n) => (n.id === id ? { ...n, draggable: false } : n));
    }
  }
</script>

<div class="view workspace">
  <nav class="rail">
    <PanelLabel>Graph</PanelLabel>
    <div style="padding: 0 calc(var(--u) * 3.5); display: flex; flex-direction: column; gap: calc(var(--u) * 2)">
      <Kv label="Nodes" value={String(nodes.length)} />
      <Kv label="Edges" value={String(edges.length)} />
      {#if hidden > 0}<Kv label="Below threshold" value={String(hidden)} />{/if}
      <label class="hint" style="display: flex; align-items: center; gap: calc(var(--u) * 2)">
        Threshold
        <input type="range" min="0" max="1" step="0.05" bind:value={threshold} />
        <span class="mono">{Number(threshold).toFixed(2)}</span>
      </label>
    </div>

    <div style="padding: 0 calc(var(--u) * 3.5)">
      <Notice tone="warn">
        Running on sample data — Graph.ListEdges is not served yet.
      </Notice>
    </div>
  </nav>

  <div style="display: flex; flex-direction: column; min-height: 0">
    <div class="graph-bar">
      <span class="mono faint" style="font-size: 0.75rem">
        Click a node for its actions · drag between two to draw an edge
      </span>
      <div style="flex: 1"></div>
      {#if took}<span class="mono faint" style="font-size: 0.6875rem">{took}</span>{/if}
      <Tag tone="amber">sample data</Tag>
    </div>

    <!-- The canvas is absolute, so this is the box that gives it a size. -->
    <div style="flex: 1; min-height: 0; position: relative">
      <GraphEditor bind:nodes bind:edges {threshold} onselect={(id) => (picked = id)} onaction={act} />
    </div>
  </div>

  <aside class="side">
    <div class="side-tabs"><span class="side-tab" aria-current="true">Node</span></div>
    <div class="side-body">
      {#if chosen}
        <PanelLabel>Selected</PanelLabel>
        <p style="font-size: 0.8125rem">{chosen.data["label"]}</p>
        <Kv label="Id" value={chosen.id} />
        <Kv label="Degree" value={String(neighbours.length)} />
        <PanelLabel>Neighbours</PanelLabel>
        {#each neighbours as e (e.id)}
          <Kv label={String(e.label)} value={e.source === picked ? e.target : e.source} />
        {/each}
      {:else}
        <p class="hint">
          Nothing selected. Click a node to raise its actions, or drag from one
          node's right edge to another's left to draw an edge.
        </p>
      {/if}
    </div>
  </aside>
</div>

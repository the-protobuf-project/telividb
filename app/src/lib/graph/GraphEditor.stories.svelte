<!--
  The node editor, on sample data.

  Try it: drag a node, drag from a node's right edge to another's left to draw an
  edge, and move the threshold to hide weak ones. The `Graph` service is not
  served yet, so these points come from a fixture — which is why the panel says
  so rather than pretending.
-->
<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import GraphEditor from "./GraphEditor.svelte";
  import { nodes as seedNodes, edges as seedEdges } from "./fixture";
  import { Kv, PanelLabel, Tag } from "$lib/ui";

  const { Story } = defineMeta({
    title: "Graph/Node editor",
    parameters: { layout: "fullscreen" },
  });

  let nodes = $state(structuredClone(seedNodes));
  let edges = $state(structuredClone(seedEdges));
  let threshold = $state(0);
  let picked = $state<string | null>(null);
  /** The last action taken, so the wheel's effect is visible rather than claimed. */
  let took = $state<string | null>(null);

  /**
   * What an action means.
   *
   * Handled here rather than in the editor: the canvas raises the wheel and
   * reports the choice, but what "hide" does to a graph belongs to whoever owns
   * the data. Hide and inspect act for real; the rest report, because they need
   * the `Graph` service that is not served yet.
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

  let chosen = $derived(nodes.find((n) => n.id === picked));
  let degree = $derived(
    edges.filter((e) => e.source === picked || e.target === picked).length,
  );
  let hidden = $derived(
    edges.length -
      edges.filter((e) => ((e.data?.["weight"] as number) ?? 1) >= threshold).length,
  );
</script>

<Story name="Editable">
  <div style="display: grid; grid-template-columns: 1fr 20rem; height: 100vh">
    <div style="display: flex; flex-direction: column; min-height: 0">
      <div class="graph-bar">
        <span class="mono faint" style="font-size: 0.75rem">
          {nodes.length} nodes · {edges.length} edges
          {#if hidden > 0}· {hidden} below threshold{/if}
        </span>
        <div style="flex: 1"></div>
        {#if took}<span class="mono faint" style="font-size: 0.6875rem">{took}</span>{/if}
        <Tag tone="amber">sample data</Tag>
        <label class="hint" style="display: flex; align-items: center; gap: 0.5rem">
          Threshold
          <input type="range" min="0" max="1" step="0.05" bind:value={threshold} />
          <span class="mono">{Number(threshold).toFixed(2)}</span>
        </label>
      </div>

      <!-- The canvas is absolute, so this is the box that gives it a size. -->
      <div style="flex: 1; min-height: 0; position: relative">
        <GraphEditor
          bind:nodes
          bind:edges
          {threshold}
          onselect={(id) => (picked = id)}
          onaction={act}
        />
      </div>
    </div>

    <aside class="side">
      <div class="side-tabs"><span class="side-tab" aria-current="true">Node</span></div>
      <div class="side-body">
        {#if chosen}
          <PanelLabel>Selected</PanelLabel>
          <p style="font-size: 0.8125rem">{chosen.data["label"]}</p>
          <Kv label="Id" value={chosen.id} />
          <Kv label="Degree" value={String(degree)} />
          <PanelLabel>Neighbours</PanelLabel>
          {#each edges.filter((e) => e.source === picked || e.target === picked) as e (e.id)}
            <Kv
              label={String(e.label)}
              value={e.source === picked ? e.target : e.source}
            />
          {/each}
        {:else}
          <p class="hint">
            Click a node to raise its action wheel. Drag from one node's right
            edge to another's left to draw an edge. Escape or a click away
            dismisses the wheel.
          </p>
        {/if}
      </div>
    </aside>
  </div>
</Story>

<!--
  The node editor.

  A graph you can rearrange and draw in, not a rendered picture — dragging
  between two nodes creates an edge, which is the difference between reading a
  graph and building one.

  Deliberately thin: no shadows, no rounded nodes, no minimap, no gradient
  background. Position, line weight and one accent carry everything. Nodes snap
  to the same unit the rest of the window is built on, so a hand-arranged graph
  stays on the grid rather than drifting off it.
-->
<script lang="ts">
  import { SvelteFlow, Background, Controls, type Edge, type Node } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/base.css";
  import "./style.css";
  import PointNode from "./PointNode.svelte";
  import ActionWheel from "./ActionWheel.svelte";
  import { stagger } from "$lib/motion/motion";

  interface Props {
    /** The points, as nodes. Bindable — dragging moves them. */
    nodes: Node[];
    /** The edges between them. Bindable — dragging between handles adds one. */
    edges: Edge[];
    /** Only draw edges at or above this weight. */
    threshold?: number;
    /** Called when a node is chosen, so an inspector can follow. */
    onselect?: (id: string | null) => void;
    /**
     * Called with an action and the node it was chosen for.
     *
     * The editor raises the wheel and reports the choice; what "expand" or
     * "hide" *means* belongs to the panel that owns the data, not to the canvas
     * that drew it.
     */
    onaction?: (action: string, node: string) => void;
  }

  let {
    nodes = $bindable(),
    edges = $bindable(),
    threshold = 0,
    onselect,
    onaction,
  }: Props = $props();

  /** The canvas, so its nodes can be staggered in once they exist. */
  let canvas = $state<HTMLElement | null>(null);

  // Once, after the first layout. Re-running on every node change would restart
  // the entrance each time an edge was drawn, which reads as a flicker rather
  // than as an arrival.
  let arrived = false;
  $effect(() => {
    if (!canvas || arrived || nodes.length === 0) return;
    arrived = true;
    // A frame late on purpose: Svelte Flow positions its nodes after mount, and
    // animating before that staggers them at the origin.
    requestAnimationFrame(() => {
      const els = canvas?.querySelectorAll(".svelte-flow__node");
      if (els) stagger(els);
    });
  });

  /** The node the wheel is open for, and where to draw it. */
  let wheel = $state<{ id: string; label: string; x: number; y: number } | null>(null);

  const nodeTypes = { point: PointNode };

  // Filtered rather than removed: raising the threshold hides weak edges for
  // reading, it does not delete them. Dropping them from `edges` would make the
  // slider destructive, which is not what a viewing control should be.
  let shown = $derived(
    edges.filter((e) => ((e.data?.["weight"] as number) ?? 1) >= threshold),
  );

  /** Report the selection, or its absence, to whoever is inspecting. */
  function report(): void {
    onselect?.(nodes.find((n) => n.selected)?.id ?? null);
  }
</script>

<div class="graph-view" bind:this={canvas}>
  <SvelteFlow
    bind:nodes
    edges={shown}
    {nodeTypes}
    colorMode="dark"
    fitView
    minZoom={0.3}
    maxZoom={2}
    snapGrid={[8, 8]}
    proOptions={{ hideAttribution: true }}
    onnodeclick={({ event, node }) => {
      report();
      // Seated at the pointer rather than at the node's centre: a node is
      // 11rem wide, and a ring around its middle would open under the cursor
      // on one side and half a panel away on the other.
      const e = event as MouseEvent;
      wheel = {
        id: node.id,
        label: String((node.data as Record<string, unknown>)["label"] ?? node.id),
        x: e.clientX,
        y: e.clientY,
      };
    }}
    onpaneclick={() => {
      wheel = null;
      onselect?.(null);
    }}
    onconnect={({ source, target }) => {
      // Weight defaults to the middle of the range rather than to 1: an edge a
      // person drew by hand is an assertion, not a measurement, and drawing it
      // at full strength would outrank every computed edge on the canvas.
      edges = [
        ...edges,
        {
          id: `e-${source}-${target}-${edges.length}`,
          source,
          target,
          label: "follows",
          data: { weight: 0.5 },
          style: "stroke-width: 1.9px",
        },
      ];
    }}
  >
    <Background gap={24} size={1} bgColor="var(--ground)" patternColor="var(--grid-line)" />
    <Controls showLock={false} />
  </SvelteFlow>

  {#if wheel}
    <ActionWheel
      x={wheel.x}
      y={wheel.y}
      label={wheel.label}
      onpick={(action) => {
        onaction?.(action, wheel!.id);
        wheel = null;
      }}
      onclose={() => (wheel = null)}
    />
  {/if}
</div>

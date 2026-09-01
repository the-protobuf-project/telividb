<!--
  One point, as a node.

  A rectangle with a rule, not a rounded card with a shadow: the graph is drawn
  in the same language as the rest of the window, and a node that looked like a
  different product would be the only thing on screen that did.

  Text is clamped rather than wrapped. A node that grows to fit its content makes
  the layout depend on what happens to be stored, and the point of a fixed module
  is that it does not.
-->
<script module lang="ts">
/** What a point node carries. */
export interface PointNodeData {
/** The text the point holds, clamped for display. */
  label: string;
/** How many edges touch it — the reason a node is worth looking at. */
  degree: number;
/** Whether this node is the traversal seed. */
  seed?: boolean;
/** Whether it sits behind a protection the caller cannot see through. */
  suppressed?: boolean;
  [key: string]: unknown;
}
</script>

<script lang="ts">
  import { Handle, Position, type NodeProps } from "@xyflow/svelte";

  let { data, selected }: NodeProps = $props();
  let d = $derived(data as unknown as PointNodeData);
</script>

<div
  class="g-node"
  class:seed={d.seed}
  class:suppressed={d.suppressed}
  class:selected
>
  <Handle type="target" position={Position.Left} />
  {#if d.suppressed}
    <!-- Drawn as a boundary with nothing behind it, never as an absence: a
         reader must be able to tell "no neighbours" from "neighbours you cannot
         see" (rule 27). -->
    <span class="g-node-text faint">— withheld —</span>
  {:else}
    <span class="g-node-text">{d.label}</span>
  {/if}
  <span class="g-node-degree mono">{d.degree}</span>
  <Handle type="source" position={Position.Right} />
</div>

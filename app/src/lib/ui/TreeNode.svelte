<!--
  One line of the structure rail.

  Depth is the only thing separating the three kinds, and a space sits at the
  same depth as a project rather than beneath one — because that is what it is: a
  sibling under the organization that *references* projects. Nesting it would
  draw a parent that does not exist.
-->
<script lang="ts">
  import Lock from "./Lock.svelte";
  import type { Protection } from "./Lock.svelte";

  /** Which level of the tree this line sits at. */
  export type NodeKind = "org" | "project" | "space";

  interface Props {
    /** What it is called. */
    name: string;
    /** Which level it sits at. */
    kind: NodeKind;
    /** A short figure at the right — counts, or the id. */
    count?: string;
    /** Shown for a space, in place of the count. */
    protection?: Protection;
    /** Whether this is the open one. */
    current?: boolean;
    /** Dimmed, for a soft-deleted resource. */
    muted?: boolean;
    /** Clicking opens it. */
    onclick?: () => void;
  }

  let { name, kind, count, protection, current, muted, onclick }: Props = $props();
</script>

<button
  class="node {kind}"
  type="button"
  aria-current={current}
  style={muted ? "opacity:.5" : ""}
  {onclick}
>
  <span class="name">{name}</span>
  {#if protection}
    <Lock {protection} />
  {:else if count}
    <span class="count">{count}</span>
  {/if}
</button>

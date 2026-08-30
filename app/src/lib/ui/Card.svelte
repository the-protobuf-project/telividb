<!--
  The centred card the first-run flow lives in.

  Three regions rather than one box: the head carries the stepper and the
  heading, the body scrolls if it must, and the foot keeps its actions in the
  same place from step to step — which is what stops the primary button moving
  under the cursor between screens.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    /** Stepper or eyebrow, above the heading. */
    head?: Snippet;
    /** The card's content. */
    children: Snippet;
    /** Actions, pinned to the bottom. */
    foot?: Snippet;
  }

  let { head, children, foot }: Props = $props();
</script>

<div class="card">
  {#if head}<div class="card-head">{@render head()}</div>{/if}
  <div class="card-body">{@render children()}</div>
  {#if foot}<div class="card-foot">{@render foot()}</div>{/if}
</div>

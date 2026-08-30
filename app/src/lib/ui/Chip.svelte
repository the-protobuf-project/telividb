<!--
  A status chip for the top bar: a dot and a word.

  Reports the two facts nothing else on screen states — which model is resident,
  and which backend the engine actually selected. A build that quietly fell back
  to the host is indistinguishable from a healthy one everywhere else.
-->
<script lang="ts">
  import Dot from "./Dot.svelte";
  import type { DotState } from "./Dot.svelte";

  interface Props {
    /** What the chip reports. */
    label: string;
    /** The dot's state. */
    state?: DotState;
    /** Hover text, for the fact the label had no room to say. */
    title?: string;
  }

  let { label, state = "idle", title }: Props = $props();
</script>

<div class="chip" {title}>
  <!-- The label beside it already says this. -->
  <Dot {state} decorative />
  <span>{label}</span>
</div>

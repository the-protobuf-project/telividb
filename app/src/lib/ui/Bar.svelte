<!--
  A determinate progress bar.

  Determinate only: it takes a fraction, and a caller that does not yet know the
  total should not render one. A bar that fills from nothing to nothing reads as
  a stall rather than as a start.
-->
<script lang="ts">
  import { count } from "$lib/motion/motion";

  interface Props {
    /** How far along, from 0 to 1. Values outside are clamped. */
    value: number;
  }

  let { value }: Props = $props();

  let target = $derived(Math.max(0, Math.min(1, value)) * 100);

  /**
   * What is drawn, which follows the target rather than jumping to it.
   *
   * A download reports progress in chunks, so the raw value arrives in steps —
   * a width that snapped between poll results read as a stall and then a leap.
   * Tweening between them is the difference between "stuck, then jumped" and
   * "moving".
   */
  let shown = $state(0);

  $effect(() => {
    const to = target;
    count(shown, to, (v) => (shown = v));
  });
</script>

<div
  class="bar"
  role="progressbar"
  aria-valuenow={Math.round(target)}
  aria-valuemin={0}
  aria-valuemax={100}
>
  <!-- The reported value is the real one; only the drawn width lags. A screen
       reader announcing a tweened number would be reading an animation. -->
  <i style="width: {shown}%"></i>
</div>

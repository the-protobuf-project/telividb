<!--
  The onboarding progress header.

  Every step is shown, including the ones not reached, because a first-run flow
  that reveals its length one screen at a time gives no way to judge whether to
  start now or later.
-->
<script lang="ts">
  interface Props {
    /** The steps, in order. */
    labels: readonly string[];
    /** Which one is current, zero-based. Earlier ones are done. */
    current: number;
  }

  let { labels, current }: Props = $props();
</script>

<div class="steps">
  {#each labels as label, i (label)}
    <div class="step {i < current ? 'done' : i === current ? 'now' : ''}">
      <span class="n">{i < current ? "✓" : i + 1}</span>{label}
    </div>
    {#if i < labels.length - 1}<span class="step-gap"></span>{/if}
  {/each}
</div>

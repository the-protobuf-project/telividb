<!--
  A horizontal bar per row: what is resident, which operation is slowest.

  A bar chart rather than a line one because these are magnitudes at one moment,
  not a series over time — and a row of labelled bars is read without a legend,
  which a stacked column is not.
-->
<script lang="ts">
  /** One bar. */
  export interface Bar {
    /** What it measures. */
    readonly label: string;
    /** The magnitude. */
    readonly value: number;
    /** The figure as written, when it is not just the number. */
    readonly display?: string;
    /** The fill, when this bar means something different from its neighbours. */
    readonly color?: string;
  }

  interface Props {
    /** The bars, in the order they should be read. */
    bars: readonly Bar[];
    /** The top of the scale. Defaults to the largest bar. */
    max?: number;
  }

  let { bars, max }: Props = $props();

  // Shared across every bar, so their lengths are comparable to each other
  // rather than each being scaled to itself.
  let ceiling = $derived(max ?? Math.max(1, ...bars.map((b) => b.value)));
</script>

{#if bars.length === 0}
  <p class="hint" style="margin: 0">Nothing recorded yet.</p>
{:else}
  <div>
    {#each bars as bar (bar.label)}
      <div class="hbar">
        <span class="lbl" title={bar.label}>{bar.label}</span>
        <span class="track">
          <i
            style="width: {Math.min(100, (bar.value / ceiling) * 100)}%{bar.color
              ? `; background: ${bar.color}`
              : ''}"
          ></i>
        </span>
        <span class="val">{bar.display ?? bar.value}</span>
      </div>
    {/each}
  </div>
{/if}

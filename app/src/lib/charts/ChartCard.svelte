<!--
  A chart with its title, its subtitle and its legend.

  The legend sits in the header rather than under the plot because it is part of
  what the chart *is*, and a reader who has to look below the axis to find out
  what a line means has already misread it once.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Series } from "./geometry";

  interface Props {
    /** What the chart shows. */
    title: string;
    /** How to read it — the axis, the window, the unit. */
    subtitle?: string;
    /** Drawn as swatches in the header. Omit for a single-series chart. */
    series?: readonly Series[];
    /** The plot. */
    children: Snippet;
  }

  let { title, subtitle, series, children }: Props = $props();
</script>

<div class="chart-card">
  <div class="chart-head">
    <h3>{title}</h3>
    {#if subtitle}<span class="sub">{subtitle}</span>{/if}
    {#if series && series.length > 1}
      <div class="legend-row">
        {#each series as s (s.key)}
          <span><i class="swatch" style="background: {s.color}"></i>{s.label}</span>
        {/each}
      </div>
    {/if}
  </div>
  <div class="chart-body">{@render children()}</div>
</div>

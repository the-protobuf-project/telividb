<!--
  One operation's latency over its recent calls.

  The shadcn chart component rather than a hand-drawn SVG, for the same reason
  every other control here comes from the library: it reads `--chart-1` from the
  palette, carries its own tooltip and axes, and follows the theme without being
  told about it twice.
-->
<script lang="ts">
  import { AreaChart } from "layerchart";
  import * as Chart from "$lib/components/ui/chart";
  import type { Timing } from "$lib/api";

  interface Props {
    /** Samples for one operation, oldest first. */
    samples: readonly Timing[];
  }

  let { samples }: Props = $props();

  /**
   * Points indexed by position rather than by timestamp.
   *
   * The x-axis is "recent calls", not wall-clock time: these arrive in bursts
   * when a panel is used and not at all when it is idle, so a time axis would be
   * mostly empty space with the interesting part crushed at one end.
   */
  let data = $derived(samples.map((t, i) => ({ call: i + 1, ms: t.ms })));

  const chartConfig = {
    ms: { label: "milliseconds", color: "var(--chart-1)" },
  } satisfies Chart.ChartConfig;
</script>

{#if data.length === 0}
  <p class="text-muted-foreground py-6 text-xs">Nothing measured yet.</p>
{:else}
  <Chart.Container config={chartConfig} class="h-28 w-full">
    <AreaChart
      {data}
      x="call"
      y="ms"
      axis="y"
      series={[{ key: "ms", label: "ms", color: "var(--chart-1)" }]}
    >
      {#snippet tooltip()}
        <Chart.Tooltip />
      {/snippet}
    </AreaChart>
  </Chart.Container>
{/if}

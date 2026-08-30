<!--
  Performance: what the engine runs on, and how long this window's calls took.

  The two halves are separated deliberately. The device report is the engine's
  and is true of the machine; the latencies were measured here and are true only
  of the calls this window made. An empty chart means nothing has run yet — a
  fact about this session, not a gap in instrumentation.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import { MetricsState } from "./state.svelte";
  import Device from "./Device.svelte";
  import { ChartCard, LineChart, type Series } from "$lib/charts";
  import { Button, Empty, Notice, PanelLabel } from "$lib/ui";

  const metrics = new MetricsState(client);
  void metrics.load();

  /** One line per operation, coloured from the two validated series tokens. */
  function seriesFor(op: string, i: number): Series[] {
    return [
      {
        key: op,
        label: op,
        data: metrics.samples(op).map((t) => t.ms),
        color: i % 2 === 0 ? "var(--chart-1)" : "var(--chart-2)",
      },
    ];
  }
</script>

<div class="view one">
  <div class="page">
  <div class="page-top">
    <div class="page-inner">
      <div class="page-head">
        <h1>Performance</h1>
        <div class="spacer"></div>
        <Button variant="ghost" size="sm" onclick={() => metrics.clear()}>
          Reset measurements
        </Button>
      </div>
      <p class="lede">
        Measured in this process, not estimated. Everything below comes from work
        this window has actually done — so an empty chart means nothing has run
        yet.
      </p>
    </div>
  </div>

  <div class="page-scroll">
    <div class="page-inner">
      {#if metrics.error}
        <Notice tone="error">{metrics.error}</Notice>
      {/if}

      <Device state={metrics} />

      <PanelLabel>Latency</PanelLabel>

      {#each metrics.summaries as summary, i (summary.op)}
        <ChartCard
          title={summary.op}
          subtitle="{summary.count} calls · median {summary.median.toFixed(0)} ms · worst {summary.worst.toFixed(0)} ms{summary.failures >
          0
            ? ` · ${summary.failures} failed`
            : ''}"
        >
          <LineChart series={seriesFor(summary.op, i)} label="{summary.op} latency per call" />
        </ChartCard>
      {:else}
        <Empty>
          No calls measured yet. Open Workspace or Data, and the timings appear
          here.
        </Empty>
      {/each}
    </div>
  </div>
</div>
</div>

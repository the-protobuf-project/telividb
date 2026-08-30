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

  const metrics = new MetricsState(client);
  void metrics.load();

  // Loaded on demand: the chart brings layerchart and d3 with it — some 340 kB —
  // for a panel most sessions never open. Resolved once here, not per row.
  const chart = import("./LatencyChart.svelte").then((m) => m.default);
</script>

<div class="page">
  <div class="page-top">
    <div class="page-inner">
      <div class="page-head">
        <h1>Performance</h1>
        <div class="spacer"></div>
        <button class="btn ghost sm" type="button" onclick={() => metrics.clear()}>
          Reset measurements
        </button>
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
        <p class="selectable" style="color: var(--red-text)">{metrics.error}</p>
      {/if}

      <Device state={metrics} />

      <div>
        <div class="panel-label">Latency</div>
      </div>

      {#each metrics.summaries as summary (summary.op)}
        <div class="chart-card">
          <div class="chart-head">
            <h3 class="mono">{summary.op}</h3>
            <span class="sub">per call, most recent last</span>
            <div class="legend-row">
              <span class="mono">{summary.count} calls</span>
              <span class="mono">median {summary.median.toFixed(0)} ms</span>
              <span class="mono">worst {summary.worst.toFixed(0)} ms</span>
              {#if summary.failures > 0}
                <span class="mono" style="color: var(--red-text)">
                  {summary.failures} failed
                </span>
              {/if}
            </div>
          </div>
          <div class="chart-body">
            {#await chart then LatencyChart}
              <LatencyChart samples={metrics.samples(summary.op)} />
            {/await}
          </div>
        </div>
      {:else}
        <div class="empty">
          No calls measured yet. Open Workspace or Data, and the timings appear
          here.
        </div>
      {/each}
    </div>
  </div>
</div>

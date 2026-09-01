<!--
  Performance: what the engine runs on, and how long this window's calls took.

  The two halves are separated deliberately. The device report is the engine's
  and is true of the machine; the latencies were measured here and are true only
  of the calls this window made. An empty table means nothing has run yet — a
  fact about this session, not a gap in instrumentation.

  **Why this is a table and not a stack of charts.** It used to render a titled
  `ChartCard` with a full `LineChart` for every operation, one under the other.
  Ten operations meant ten charts and 1162px of content in a 594px pane, so the
  panel could only be read by scrolling — and each chart carried its own y-axis,
  which made the comparison a reader actually wants ("which call is slow?")
  impossible to make by eye. Rows put every operation on screen at once, the
  digits line up down their columns, and the shape survives as a sparkline. The
  table body is the one thing here allowed to scroll, because the number of
  operations is genuinely unbounded; the head and the tiles are not.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import { MetricsState } from "./state.svelte";
  import Device from "./Device.svelte";
  import { Sparkline } from "$lib/charts";
  import { Button, DataViewport, Empty, Notice, Page, PanelLabel } from "$lib/ui";

  const metrics = new MetricsState(client);
  void metrics.load();
</script>

<div class="view one">
  <Page
    title="Performance"
    lede="Measured in this process, not estimated. Everything below comes from work this window has actually done — so an empty table means nothing has run yet."
  >
    {#snippet actions()}
      <Button variant="ghost" size="sm" onclick={() => metrics.clear()}>
        Reset measurements
      </Button>
    {/snippet}

    <div class="metrics-stack">
      <div class="metrics-head">
        {#if metrics.error}
          <Notice tone="error">{metrics.error}</Notice>
        {/if}

        <Device state={metrics} />

        <PanelLabel>Latency by operation</PanelLabel>
      </div>

      {#if metrics.summaries.length === 0}
        <Empty>
          No calls measured yet. Open Workspace or Data, and the timings appear
          here.
        </Empty>
      {:else}
        <DataViewport label="Latency by operation">
          <table>
            <thead>
              <tr>
                <th scope="col">Operation</th>
                <th scope="col" class="num">Calls</th>
                <th scope="col" class="num">Median</th>
                <th scope="col" class="num">Worst</th>
                <th scope="col" class="num">Failed</th>
                <th scope="col">Shape</th>
              </tr>
            </thead>
            <tbody>
              {#each metrics.summaries as s, i (s.op)}
                <tr>
                  <td class="selectable">{s.op}</td>
                  <td class="num tnum">{s.count}</td>
                  <td class="num tnum">{s.median.toFixed(0)} ms</td>
                  <td class="num tnum">{s.worst.toFixed(0)} ms</td>
                  <!-- A count, and the word, so a failure is not carried by
                       colour alone. Zero stays a dash: a column of noughts
                       reads as data when it means "nothing to report". -->
                  <td class="num tnum" class:bad={s.failures > 0}>
                    {s.failures > 0 ? s.failures : "—"}
                  </td>
                  <td>
                    <Sparkline
                      data={metrics.samples(s.op).map((t) => t.ms)}
                      color={i % 2 === 0 ? "var(--chart-1)" : "var(--chart-2)"}
                      label="{s.op}: {s.count} calls, median {s.median.toFixed(0)} ms, worst {s.worst.toFixed(0)} ms"
                    />
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </DataViewport>
      {/if}
    </div>
  </Page>
</div>

<style>
  /* Tiles and label are fixed; the table takes what is left. This is what
     keeps the panel inside the pane at every window height rather than
     growing past it. */
  .metrics-stack {
    display: grid;
    grid-template-rows: auto 1fr;
    gap: calc(var(--u) * 4);
    min-height: 0;
    height: 100%;
  }

  .metrics-head {
    display: flex;
    flex-direction: column;
    gap: calc(var(--u) * 4);
  }

  /* The header row stays put while the body scrolls under it — otherwise the
     column names leave the screen exactly when a long list needs them. */
  thead th {
    position: sticky;
    top: 0;
    background: var(--surface);
    z-index: 1;
  }

  .num {
    text-align: right;
  }

  .bad {
    color: var(--red-text);
  }
</style>

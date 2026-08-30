<!--
  The charts, on sample series.

  Hover a plot: a vertical rule follows the pointer and the readout under the
  axis names the values at that point. The readout sits below rather than
  floating at the cursor, because a floating box covers the very points it
  describes exactly when the line rises.
-->
<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import { ChartCard, HBar, LineChart, type Series } from "./index";
  import { Empty, PanelLabel } from "$lib/ui";

  const { Story } = defineMeta({ title: "Charts" });

  /** A plausible latency walk: a slow first call, then a settled band. */
  function walk(seed: number, n: number, base: number, spread: number): number[] {
    let x = seed;
    return Array.from({ length: n }, (_, i) => {
      x = (x * 1103515245 + 12345) % 2147483648;
      const jitter = (x / 2147483648) * spread;
      // The first call pays for a cold path, which is exactly the shape a median
      // resists and a mean does not — worth showing rather than smoothing away.
      return i === 0 ? base * 4 + jitter : base + jitter;
    });
  }

  const latency: Series[] = [
    { key: "embed", label: "embed", data: walk(7, 24, 12, 6), color: "var(--chart-1)" },
    { key: "search", label: "search", data: walk(13, 24, 4, 3), color: "var(--chart-2)" },
  ];

  const points: Series[] = [
    {
      key: "points",
      label: "points",
      // Cumulative, so it only ever rises — a different shape from latency and
      // worth having in the set, since a chart that can only go up needs no
      // headroom above its peak to read correctly.
      data: walk(3, 24, 40, 30).map((_, i, a) => a.slice(0, i + 1).reduce((s, x) => s + x, 0)),
      color: "var(--chart-1)",
    },
  ];

  const resident = [
    { label: "qwen3-embedding-0.6b", value: 639, display: "639 MB" },
    { label: "hnsw · notes", value: 128, display: "128 MB" },
    { label: "flat · journal", value: 42, display: "42 MB" },
    { label: "ivfpq · archive", value: 18, display: "18 MB" },
  ];
</script>

<Story name="Every chart">
  <div style="display: flex; flex-direction: column; gap: 1rem; max-width: 46rem">
    <ChartCard title="Latency" subtitle="per call, most recent last" series={latency}>
      <LineChart series={latency} label="Latency per call, embed and search" />
    </ChartCard>

    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(19rem, 1fr)); gap: 1rem">
      <ChartCard title="Resident" subtitle="what is in memory now">
        <HBar bars={resident} />
      </ChartCard>

      <ChartCard title="Points written" subtitle="cumulative, this session">
        <LineChart series={points} label="Points written this session" format={(v) => v.toFixed(0)} unit="batch" />
      </ChartCard>
    </div>
  </div>
</Story>

<!-- What a chart shows before anything has run, which is most of a fresh
     session — and the state a happy-path drawing never includes. -->
<Story name="Nothing recorded">
  <div style="display: flex; flex-direction: column; gap: 1rem; max-width: 46rem">
    <ChartCard title="Latency" subtitle="per call, most recent last">
      <LineChart series={[{ key: "embed", label: "embed", data: [], color: "var(--chart-1)" }]} label="Latency, empty" />
    </ChartCard>
    <ChartCard title="Resident" subtitle="what is in memory now">
      <HBar bars={[]} />
    </ChartCard>
    <PanelLabel>And the panel around them</PanelLabel>
    <Empty>No calls measured yet. Open Workspace and the timings appear here.</Empty>
  </div>
</Story>

<!-- One point is a real state — the first call of a session — and the one most
     likely to be drawn wrong: it belongs in the middle, not against the axis. -->
<Story name="A single call">
  <div style="max-width: 32rem">
    <ChartCard title="Latency" subtitle="one call so far">
      <LineChart series={[{ key: "embed", label: "embed", data: [47.2], color: "var(--chart-1)" }]} label="Latency, one call" />
    </ChartCard>
  </div>
</Story>

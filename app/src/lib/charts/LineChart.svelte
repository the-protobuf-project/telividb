<!--
  A line chart, drawn in the design's own vocabulary.

  Hand-drawn SVG rather than a charting library. That is a deliberate trade: the
  design already specifies this chart down to the tick size and the grid colour,
  so a library meant fighting its defaults on every one of them — and layerchart
  brought some 340 kB with d3 to draw eleven elements.

  The final point carries its series name inline, so identity never rests on
  colour alone: a reader with a red deficiency, or one printing this in grey,
  still knows which line is which.
-->
<script lang="ts">
  import { ms, nearest, PAD, path, plot, type Series } from "./geometry";

  interface Props {
    /** The lines to draw. */
    series: readonly Series[];
    /** Height of the plot, in the SVG's coordinate space. */
    height?: number;
    /** How a value is written. Milliseconds by default. */
    format?: (v: number) => string;
    /** What the horizontal axis counts. */
    unit?: string;
    /** Described to a screen reader, which cannot see the plot at all. */
    label: string;
  }

  let {
    series,
    height = 150,
    format = ms,
    unit = "call",
    label,
  }: Props = $props();

  /**
   * The plot's width, measured rather than assumed.
   *
   * A fixed viewBox scaled to fit takes the type with it: the same chart in a
   * 19rem card rendered its 10px ticks at about 5px and became unreadable. One
   * SVG unit is one pixel, so the axis labels stay the size the design set them
   * at whatever width the card happens to be.
   */
  let measured = $state(560);
  let W = $derived(Math.max(240, measured));

  let p = $derived(plot(series, W, height));
  let hovered = $state<number | null>(null);

  /** Which point the pointer is over, from its position across the plot. */
  function track(event: MouseEvent) {
    const box = (event.currentTarget as SVGRectElement).ownerSVGElement?.getBoundingClientRect();
    if (!box) return;
    hovered = nearest((event.clientX - box.left) / box.width, p);
  }
</script>

<div bind:clientWidth={measured}>
{#if p.n === 0}
  <p class="hint" style="margin: 0">Nothing recorded yet.</p>
{:else}
  <svg viewBox="0 0 {W} {height}" role="img" aria-label={label} class="chart-svg">
    <!-- Rules first, so every line is drawn over them rather than under. -->
    {#each p.ticks as t (t)}
      <line class="grid-l" x1={PAD.l} y1={p.y(t)} x2={W - PAD.r} y2={p.y(t)} />
      <text class="tick" x={PAD.l - 6} y={p.y(t) + 3} text-anchor="end">{format(t)}</text>
    {/each}
    <line class="axis-l" x1={PAD.l} y1={height - PAD.b} x2={W - PAD.r} y2={height - PAD.b} />

    {#each series as s (s.key)}
      {#if s.data.length > 0}
        {@const last = s.data.length - 1}
        <path
          d={path(s, p)}
          fill="none"
          stroke={s.color}
          stroke-width="2"
          stroke-linejoin="round"
          stroke-linecap="round"
        />
        <circle
          cx={p.x(last)}
          cy={p.y(s.data[last] ?? 0)}
          r="3.5"
          fill={s.color}
          stroke="var(--surface)"
          stroke-width="2"
        />
        <text class="tick" x={p.x(last) + 8} y={p.y(s.data[last] ?? 0) + 3} fill={s.color}>
          {s.label}
        </text>
      {/if}
    {/each}

    {#if hovered !== null}
      <line
        class="grid-l"
        x1={p.x(hovered)}
        y1={PAD.t}
        x2={p.x(hovered)}
        y2={height - PAD.b}
        stroke="var(--rule-strong)"
      />
    {/if}

    <!-- One transparent rect takes every pointer event, so the readout works
         anywhere over the plot rather than only exactly on a 2px line. -->
    <rect
      x={PAD.l}
      y="0"
      width={W - PAD.l - PAD.r}
      height={height}
      fill="transparent"
      onmousemove={track}
      onmouseleave={() => (hovered = null)}
      role="presentation"
    />
  </svg>

  {#if hovered !== null}
    <div class="chart-readout">
      <span class="faint">{unit} {hovered + 1}</span>
      {#each series as s (s.key)}
        {#if s.data[hovered] !== undefined}
          <span class="r">
            <i class="swatch" style="background: {s.color}"></i>{s.label}
            <b>{format(s.data[hovered] ?? 0)}</b>
          </span>
        {/if}
      {/each}
    </div>
  {/if}
{/if}
</div>

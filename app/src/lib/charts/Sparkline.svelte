<!--
  One operation's shape, at the size of a table cell.

  This replaces a full `LineChart` per operation. That design put a titled card
  and a labelled axis around every RPC method and stacked them, so ten methods
  meant ten charts, 1162px of content in a 594px pane, and a panel that could
  only be read by scrolling. It also made the wrong comparison easy and the
  right one hard: each chart had its own y-axis, so two methods drawn the same
  height were not the same speed, and the number a reader actually wants —
  which call is slow — was never on screen at once.

  A sparkline gives up the axis on purpose. It answers "is this steady, spiky,
  or trending" and nothing else; the numbers that need to be exact are digits in
  the columns beside it, where they can be compared down the column. Small
  multiples in a table beat a stack of charts whenever the reader's question is
  "which of these is different".
-->
<script lang="ts">
  interface Props {
    /** Samples, oldest first. */
    data: readonly number[];
    /** Line colour. A chart token, never a status hue. */
    color?: string;
    /** What the shape is of, for anyone not looking at it. */
    label: string;
  }

  let { data, color = "var(--chart-1)", label }: Props = $props();

  const W = 88;
  const H = 20;

  /**
   * The polyline, normalized into the box.
   *
   * A flat series is drawn on the centre line rather than at the bottom: zero
   * range means every sample is equal, and pinning that to y=H would say
   * "always at the floor" when what happened is "never varied".
   */
  let points = $derived.by(() => {
    if (data.length === 0) return "";
    const lo = Math.min(...data);
    const hi = Math.max(...data);
    const span = hi - lo;
    const step = data.length > 1 ? W / (data.length - 1) : 0;
    return data
      .map((v, i) => {
        const y = span === 0 ? H / 2 : H - ((v - lo) / span) * H;
        // Inset by half the stroke so the extremes are not clipped by the box.
        return `${(i * step).toFixed(1)},${Math.min(H - 1, Math.max(1, y)).toFixed(1)}`;
      })
      .join(" ");
  });
</script>

{#if data.length > 1}
  <svg
    width={W}
    height={H}
    viewBox="0 0 {W} {H}"
    fill="none"
    role="img"
    aria-label={label}
    preserveAspectRatio="none"
  >
    <polyline
      {points}
      stroke={color}
      stroke-width="1.25"
      stroke-linejoin="round"
      vector-effect="non-scaling-stroke"
    />
  </svg>
{:else}
  <!-- One sample is not a shape. Saying so beats drawing a dot that reads as
       a trend. -->
  <span class="faint" style="font-size: 0.6875rem">one call</span>
{/if}

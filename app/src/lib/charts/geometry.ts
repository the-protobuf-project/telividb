/**
 * Turning numbers into coordinates.
 *
 * Kept apart from the components so the arithmetic can be reasoned about — and
 * tested — without a DOM. A chart that is wrong is usually wrong here, in the
 * scale or the padding, rather than in the markup.
 */

/** One line on a chart. */
export interface Series {
  /** Stable key, and what picks the colour. */
  readonly key: string;
  /** Shown in the legend and beside the final point. */
  readonly label: string;
  /** The values, oldest first. */
  readonly data: readonly number[];
  /** The stroke, as a CSS colour or variable. */
  readonly color: string;
}

/** Space reserved for the axis labels. */
export const PAD = { l: 34, r: 44, t: 8, b: 18 } as const;

/** The plot's own dimensions, in the SVG's coordinate space. */
export interface Plot {
  /** Viewbox width. */
  readonly w: number;
  /** Viewbox height. */
  readonly h: number;
  /** How many points the longest series holds. */
  readonly n: number;
  /** The top of the scale — the largest value with headroom above it. */
  readonly max: number;
  /** Horizontal position of point `i`. */
  x(i: number): number;
  /** Vertical position of value `v`. */
  y(v: number): number;
  /** Where the horizontal rules go. */
  readonly ticks: readonly number[];
}

/**
 * Measure a plot for one set of series.
 *
 * The scale always starts at zero. A chart auto-scaled to its own minimum makes
 * a flat, healthy series look like violent variation, which is the most common
 * way a latency chart lies — and 15% of headroom above the peak keeps the top
 * point off the frame, where it would otherwise read as clipped.
 */
export function plot(series: readonly Series[], w: number, h: number): Plot {
  const n = Math.max(0, ...series.map((s) => s.data.length));
  const peak = Math.max(0, ...series.flatMap((s) => [...s.data]));
  const max = peak * 1.15 || 1;
  const innerW = w - PAD.l - PAD.r;

  return {
    w,
    h,
    n,
    max,
    // A single point sits in the middle rather than hard against the axis,
    // where it would look like the start of a line that failed to draw.
    x: (i) => PAD.l + (n === 1 ? innerW / 2 : (i / (n - 1)) * innerW),
    y: (v) => h - PAD.b - (v / max) * (h - PAD.t - PAD.b),
    ticks: [0, max / 2, max],
  };
}

/** The `d` of a series' path. */
export function path(s: Series, p: Plot): string {
  return s.data
    .map((v, i) => `${i ? "L" : "M"}${p.x(i).toFixed(1)},${p.y(v).toFixed(1)}`)
    .join("");
}

/** Which point index a pointer at `fraction` across the plot is nearest. */
export function nearest(fraction: number, p: Plot): number {
  if (p.n <= 1) return 0;
  const rel = (fraction * p.w - PAD.l) / (p.w - PAD.l - PAD.r);
  return Math.max(0, Math.min(p.n - 1, Math.round(rel * (p.n - 1))));
}

/** Milliseconds, at a sensible precision for the magnitude. */
export function ms(v: number): string {
  return v >= 100 ? v.toFixed(0) : v >= 10 ? v.toFixed(1) : v.toFixed(2);
}

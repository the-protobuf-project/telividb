/**
 * Charts, drawn in the design's own vocabulary.
 *
 * No charting library: the design specifies these down to the tick size and the
 * grid colour, so a library meant overriding its defaults on every one of them
 * — and brought 340 kB of d3 to draw a polyline.
 */

export { default as ChartCard } from "./ChartCard.svelte";
export { default as HBar } from "./HBar.svelte";
export { default as LineChart } from "./LineChart.svelte";
export { ms, plot, type Series } from "./geometry";
export type { Bar } from "./HBar.svelte";

/**
 * A small graph to build against, until the `Graph` service is served.
 *
 * Real-shaped rather than lorem: resource names in the form the engine issues,
 * edge types from the schema, weights in the range a similarity edge actually
 * occupies. That matters because the panel's job is to make weight and degree
 * legible, and a fixture of uniform values would make a broken renderer look
 * fine.
 *
 * This module is the seam. When `Graph.ListEdges` lands, the panel imports the
 * client instead and the `sample data` tag comes off.
 */

import type { Edge, Node } from "@xyflow/svelte";

/** The kinds of edge the schema defines, and how each is drawn. */
export const EDGE_TYPES = {
  mentions: { label: "mentions", weight: 0.4 },
  follows: { label: "follows", weight: 0.7 },
  similar: { label: "similar to", weight: 0.9 },
} as const;

/** One text, its degree, and where it starts on the canvas. */
const POINTS: Array<[string, string, number, number, number]> = [
  ["p1", "Segments are sealed once written", 4, 0, 0],
  ["p2", "Compaction rewrites rather than edits", 3, 260, -80],
  ["p3", "Tombstones mark a delete", 3, 260, 80],
  ["p4", "mmap makes reads zero-copy", 2, 520, -140],
  ["p5", "Page faults must not block the executor", 2, 520, 0],
  ["p6", "HNSW degrades on delete", 2, 520, 140],
  ["p7", "Rebuild on compaction restores recall", 1, 780, 140],
];

/** Nodes as Svelte Flow wants them, seeded on the first point. */
export const nodes: Node[] = POINTS.map(([id, label, degree, x, y], i) => ({
  id,
  type: "point",
  position: { x, y },
  data: { label, degree, seed: i === 0 },
}));

/** Edges, typed and weighted; stroke width is drawn from the weight. */
export const edges: Edge[] = [
  { id: "e1", source: "p1", target: "p2", label: "follows", data: { weight: 0.7 } },
  { id: "e2", source: "p1", target: "p3", label: "follows", data: { weight: 0.6 } },
  { id: "e3", source: "p2", target: "p4", label: "similar to", data: { weight: 0.9 } },
  { id: "e4", source: "p2", target: "p5", label: "mentions", data: { weight: 0.35 } },
  { id: "e5", source: "p3", target: "p6", label: "similar to", data: { weight: 0.82 } },
  { id: "e6", source: "p6", target: "p7", label: "follows", data: { weight: 0.55 } },
  { id: "e7", source: "p1", target: "p5", label: "mentions", data: { weight: 0.3 } },
].map((e) => ({
  ...e,
  // Weight as stroke width, which is the whole reason to draw a graph rather
  // than list the neighbours: a strong edge should be visible without reading.
  style: `stroke-width: ${0.75 + (e.data.weight ?? 0.5) * 2.25}px`,
}));

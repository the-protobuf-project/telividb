/**
 * What a node can be asked to do, and where each sits on the wheel.
 *
 * A fixed set in a fixed order, because a radial menu is only faster than a list
 * if the position is learnable — an action that moves depending on context makes
 * every selection a fresh read, which is slower than the list it replaced.
 */

/** One action on the wheel. */
export interface NodeAction {
  /** Stable key, reported when it is chosen. */
  readonly id: string;
  /**
   * What it does, and what the button says.
   *
   * Words rather than glyphs: six icons around a node is six things to learn
   * before the menu is faster than a list, and several of these actions — pin,
   * expand, open — have no icon anyone would read the same way twice.
   */
  readonly label: string;
  /** The longer sentence, on hover. */
  readonly detail: string;
  /** Whether it is available for the current node. */
  readonly enabled?: boolean;
}

/** The six, clockwise from the top. */
export const NODE_ACTIONS: readonly NodeAction[] = [
  { id: "expand", label: "Expand", detail: "Pull in this point's neighbours" },
  { id: "connect", label: "Connect", detail: "Draw an edge from here" },
  { id: "inspect", label: "Inspect", detail: "Show it in the side panel" },
  { id: "pin", label: "Pin", detail: "Hold it in place while the graph moves" },
  { id: "hide", label: "Hide", detail: "Take it off the canvas" },
  { id: "open", label: "Open", detail: "Go to the point in Data" },
];

/**
 * Where an action sits, as an offset from the node's centre.
 *
 * An ellipse rather than a circle, and for a measured reason: a node is 176px
 * wide and 32px tall, so a circular ring wide enough to clear its sides would
 * have to be 88px in every direction — a ring that large stops reading as
 * belonging to the node. Matching the ring to the node's proportions clears the
 * text horizontally while keeping the whole thing compact vertically.
 */
export function seat(
  index: number,
  count: number,
  rx: number,
  ry: number = rx,
): { x: number; y: number } {
  // Clockwise from the top, so the first action is where the eye already is
  // after clicking — not at three o'clock, which is where a naive
  // `i / count * 2π` would put it.
  const angle = (index / count) * Math.PI * 2 - Math.PI / 2;
  return { x: Math.cos(angle) * rx, y: Math.sin(angle) * ry };
}

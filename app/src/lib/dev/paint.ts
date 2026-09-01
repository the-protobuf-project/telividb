/**
 * A layout inspector, in the spirit of Flutter's paint debugging.
 *
 * Hover any element and it draws the box model — margin, border, content — and
 * states the numbers: size, padding, margin, and whether the box sits on the
 * 4px module. Judging these by eye is what let a toggle ship with 1px of
 * clearance above its knob and 3px below, and what made "the alignment is off"
 * a matter of opinion rather than a measurement.
 *
 * Three modes, cycled from one control: off, outlines, outlines with the grid
 * overlaid so alignment can be read against the module directly.
 */

/** What the inspector is currently showing. */
export type PaintMode = "off" | "on" | "grid";

/** The overlay's own element, created once. */
let layer: HTMLElement | null = null;
/** Removes the listeners when the inspector is switched off. */
let detach: (() => void) | null = null;

/** Round to one decimal, so 31.999999 reads as 32. */
function px(n: number): string {
  return `${Math.round(n * 10) / 10}`;
}

/** Whether a measurement sits on the 4px module. */
function onGrid(...values: number[]): boolean {
  return values.every((v) => Math.abs(v % 4) < 0.51 || Math.abs((v % 4) - 4) < 0.51);
}

/** Draw one region of the box model. */
function region(cls: string, x: number, y: number, w: number, h: number): HTMLElement {
  const el = document.createElement("div");
  el.className = cls;
  el.style.cssText = `left:${x}px;top:${y}px;width:${Math.max(0, w)}px;height:${Math.max(0, h)}px`;
  return el;
}

/** Describe an element's box, as the readout shows it. */
function describe(el: Element): string {
  const r = el.getBoundingClientRect();
  const cs = getComputedStyle(el);
  const name =
    el.tagName.toLowerCase() +
    (el.className && typeof el.className === "string"
      ? "." + el.className.split(" ").filter(Boolean).slice(0, 2).join(".")
      : "");
  const pad = [cs.paddingTop, cs.paddingRight, cs.paddingBottom, cs.paddingLeft]
    .map((v) => parseFloat(v))
    .map(px)
    .join(" ");
  const fits = onGrid(r.width, r.height) ? "on grid" : "OFF GRID";
  return [
    `${name}`,
    `${px(r.width)} × ${px(r.height)}   ${fits}`,
    `padding  ${pad}`,
    `at       ${px(r.left)}, ${px(r.top)}`,
  ].join("\n");
}

/** Paint the box model for one element. */
function paint(el: Element): void {
  if (!layer) return;
  layer.replaceChildren();

  const r = el.getBoundingClientRect();
  const cs = getComputedStyle(el);
  const n = (v: string) => parseFloat(v) || 0;
  const m = [n(cs.marginTop), n(cs.marginRight), n(cs.marginBottom), n(cs.marginLeft)];
  const bw = [n(cs.borderTopWidth), n(cs.borderRightWidth), n(cs.borderBottomWidth), n(cs.borderLeftWidth)];
  const p = [n(cs.paddingTop), n(cs.paddingRight), n(cs.paddingBottom), n(cs.paddingLeft)];

  layer.append(
    region("paint-margin", r.left - m[3]!, r.top - m[0]!, r.width + m[1]! + m[3]!, r.height + m[0]! + m[2]!),
    region("paint-border", r.left, r.top, r.width, r.height),
    region(
      "paint-content",
      r.left + bw[3]! + p[3]!,
      r.top + bw[0]! + p[0]!,
      r.width - bw[1]! - bw[3]! - p[1]! - p[3]!,
      r.height - bw[0]! - bw[2]! - p[0]! - p[2]!,
    ),
  );

  const tip = document.createElement("div");
  tip.className = "paint-tip";
  tip.textContent = describe(el);
  // Below the box unless that would leave the viewport, in which case above —
  // a readout that falls off the screen tells you nothing.
  const below = r.bottom + 6;
  tip.style.left = `${Math.min(r.left, window.innerWidth - 260)}px`;
  tip.style.top = `${below + 80 > window.innerHeight ? Math.max(0, r.top - 78) : below}px`;
  layer.append(tip);
}

/**
 * Switch the inspector to a mode.
 *
 * Idempotent: calling it with the current mode does nothing, so a toolbar can
 * call it on every render without leaking listeners.
 */
export function setPaint(mode: PaintMode): void {
  const root = document.documentElement;
  detach?.();
  detach = null;
  layer?.remove();
  layer = null;

  if (mode === "off") {
    root.removeAttribute("data-paint");
    return;
  }

  root.setAttribute("data-paint", mode);
  layer = document.createElement("div");
  layer.id = "paint-layer";
  document.body.append(layer);

  const over = (e: MouseEvent) => {
    const el = e.target as Element | null;
    if (el && el.id !== "paint-layer") paint(el);
  };
  const out = () => layer?.replaceChildren();
  document.addEventListener("mouseover", over, true);
  document.addEventListener("mouseleave", out);
  detach = () => {
    document.removeEventListener("mouseover", over, true);
    document.removeEventListener("mouseleave", out);
  };
}

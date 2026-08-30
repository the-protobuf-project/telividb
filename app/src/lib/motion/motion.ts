/**
 * The window's motion, in one place.
 *
 * # Where the boundary is
 *
 * GSAP owns **sequenced motion**: entrances, step advances, anything where the
 * order or the timing carries meaning. CSS keeps **state transitions**: a hover
 * that changes a border, a focus ring, a dot that turns green. The rule is not
 * "no CSS animation" but *never both on the same property of the same element* —
 * that is how a hover ends up fighting its own entrance, with the browser
 * interpolating from wherever the tween happened to be.
 *
 * # Reduced motion is a refusal, not a discount
 *
 * Every helper here checks it and applies the *end state* directly. Someone who
 * asks for no motion means none, not a faster version of it.
 */

import { gsap } from "gsap";

/** Whether the reader has asked for less motion. */
export function reduced(): boolean {
  return (
    typeof window !== "undefined" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches
  );
}

/** The window's easing, matching the CSS `--ease` token. */
const EASE = "power2.out";

/**
 * Bring an element in.
 *
 * Used when a panel replaces another. The offset is small on purpose: a view
 * change is a change of subject, not a page navigation, and a large movement
 * would claim more than happened.
 */
export function enter(el: HTMLElement, from: "up" | "down" | "left" = "up"): void {
  if (reduced()) {
    gsap.set(el, { opacity: 1, x: 0, y: 0 });
    return;
  }
  const offset =
    from === "left" ? { x: -8, y: 0 } : { x: 0, y: from === "up" ? 6 : -6 };
  gsap.fromTo(
    el,
    { opacity: 0, ...offset },
    { opacity: 1, x: 0, y: 0, duration: 0.22, ease: EASE, clearProps: "transform" },
  );
}

/**
 * Bring a set of elements in one after another.
 *
 * The stagger is what makes a graph read as *arriving* rather than as having
 * been there all along — and it is capped, because sixty nodes at 30ms each is
 * two seconds of waiting for a canvas that could have been instant.
 */
export function stagger(els: HTMLElement[] | NodeListOf<Element>, step = 0.03): void {
  const list = Array.from(els) as HTMLElement[];
  if (list.length === 0) return;
  if (reduced()) {
    gsap.set(list, { opacity: 1, scale: 1 });
    return;
  }
  gsap.fromTo(
    list,
    { opacity: 0, scale: 0.96 },
    {
      opacity: 1,
      scale: 1,
      duration: 0.2,
      ease: EASE,
      // Total, not per-item: the whole entrance is bounded however many there
      // are, so a large graph does not take proportionally longer to appear.
      stagger: { each: step, amount: Math.min(list.length * step, 0.5) },
      clearProps: "transform",
    },
  );
}

/**
 * Tween a number, reporting each step.
 *
 * For a progress bar, where the value arrives in jumps as chunks complete: a
 * width that snapped between poll results looked like a stall and then a leap,
 * which reads as a stutter rather than as progress.
 */
export function count(
  from: number,
  to: number,
  onUpdate: (v: number) => void,
): void {
  if (reduced() || from === to) {
    onUpdate(to);
    return;
  }
  const box = { v: from };
  gsap.to(box, {
    v: to,
    duration: 0.3,
    ease: "none",
    onUpdate: () => onUpdate(box.v),
  });
}

/**
 * Slide one step of a sequence out and the next in.
 *
 * Direction carries the meaning: forward moves left, back moves right, so the
 * flow has a spatial sense rather than merely swapping contents.
 */
export function step(el: HTMLElement, direction: "forward" | "back"): void {
  if (reduced()) {
    gsap.set(el, { opacity: 1, x: 0 });
    return;
  }
  gsap.fromTo(
    el,
    { opacity: 0, x: direction === "forward" ? 14 : -14 },
    { opacity: 1, x: 0, duration: 0.24, ease: EASE, clearProps: "transform" },
  );
}

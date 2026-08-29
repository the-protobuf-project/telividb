/**
 * The launch sequence.
 *
 * One orchestrated moment rather than effects scattered through the UI. It runs
 * once, on the first paint, and covers the gap that already exists: the engine
 * is started before the window opens, so there is a real pause with nothing on
 * screen. Filling it with a considered movement is honest; adding motion
 * elsewhere to match would be decoration.
 */

import { gsap } from "gsap";

/** Elements the sequence moves, in the order it moves them. */
export interface IntroTargets {
  /** The wordmark. */
  mark: HTMLElement;
  /** The status bar. */
  bar: HTMLElement;
  /** The sidebar. */
  sidebar: HTMLElement;
  /** The panel area. */
  panel: HTMLElement;
}

/** Plays the launch sequence, once. */
export class Intro {
  /** Whether it has already run in this window. */
  private played = false;

  /**
   * Play, unless the reader asked for less motion.
   *
   * `prefers-reduced-motion` is honoured by setting the end state directly
   * rather than by shortening the animation — someone who asks for no motion
   * means none, not a fast one.
   */
  public play(targets: IntroTargets): void {
    if (this.played) return;
    this.played = true;

    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const parts = [targets.mark, targets.bar, targets.sidebar, targets.panel];

    if (reduced) {
      gsap.set(parts, { opacity: 1, y: 0, x: 0 });
      return;
    }

    gsap
      .timeline({ defaults: { ease: "power2.out" } })
      // The mark first and alone: everything else is chrome around it, and a
      // window whose parts all arrive together reads as a page load.
      .from(targets.mark, { opacity: 0, y: -6, duration: 0.4 })
      .from(targets.bar, { opacity: 0, duration: 0.3 }, "-=0.2")
      .from(targets.sidebar, { opacity: 0, x: -12, duration: 0.35 }, "-=0.15")
      .from(targets.panel, { opacity: 0, y: 8, duration: 0.35 }, "-=0.25");
  }
}

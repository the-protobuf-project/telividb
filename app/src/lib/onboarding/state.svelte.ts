/**
 * The first-run sequence.
 *
 * Three steps, and the count is a consequence rather than a design: a step is
 * here when the app can actually carry it out. Several things a setup wizard
 * would normally ask — an encryption choice, who the people are, which project
 * this belongs to — have no service behind them yet, and a form that collected
 * them would be theatre. They arrive with the services.
 */

import type { Capabilities, TelividbClient } from "$lib/api";

/** Where the sequence is. */
export type Step = "welcome" | "environment" | "collection" | "done";

/** The steps, in order, for a progress indicator. */
export const STEPS: readonly Step[] = ["welcome", "environment", "collection"];

/** Whether onboarding has been completed on this machine. */
const SEEN_KEY = "telividb.onboarded";

/** One run through onboarding. */
export class OnboardingState {
  /** The current step. */
  public step = $state<Step>("welcome");
  /** What the engine reported about itself. */
  public capabilities = $state<Capabilities | null>(null);
  /** What went wrong reaching the engine. */
  public error = $state<string | null>(null);

  /** How far along, for the indicator. */
  public readonly position = $derived(
    Math.max(0, STEPS.indexOf(this.step as Step)),
  );

  constructor(private readonly client: TelividbClient) {}

  /**
   * Whether this machine has been through onboarding.
   *
   * Kept in local storage rather than asked of the engine: a person who has
   * set the app up once should not meet the wizard again because they pointed
   * it at an empty data directory, and the engine has no notion of a person.
   */
  public static seen(): boolean {
    try {
      return localStorage.getItem(SEEN_KEY) === "1";
    } catch {
      // Private browsing, or a webview with storage disabled. Showing
      // onboarding again is the harmless failure; skipping it is not.
      return false;
    }
  }

  /** Read what the engine can do. */
  public async detect(): Promise<void> {
    try {
      this.capabilities = await this.client.capabilities();
    } catch (e) {
      this.error = String(e);
    }
  }

  /** Move to the next step. */
  public advance(): void {
    const next = STEPS[this.position + 1];
    this.step = next ?? "done";
    if (this.step === "done") {
      try {
        localStorage.setItem(SEEN_KEY, "1");
      } catch {
        // Not being able to remember is not a reason to refuse to finish.
      }
    }
  }

  /** Skip the rest and go straight in. */
  public finish(): void {
    this.step = "done";
    try {
      localStorage.setItem(SEEN_KEY, "1");
    } catch {
      // As above.
    }
  }
}

/** Bytes as a short human string — `16.0 GB`. */
export function bytes(value: number | null): string {
  if (value === null) return "—";
  const gb = value / 1024 ** 3;
  return gb >= 1 ? `${gb.toFixed(1)} GB` : `${(value / 1024 ** 2).toFixed(0)} MB`;
}

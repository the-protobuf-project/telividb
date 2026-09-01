/**
 * How long the calls this window made actually took.
 *
 * **Measured here, not reported by the engine.** The engine's telemetry goes to
 * a collector, and a window that claimed to show engine-wide numbers while
 * holding only its own would be describing a different system than the one it
 * names. Everything here is work this window did — so an empty chart means
 * nothing has run yet, which is a fact rather than a gap.
 */

/** One completed call. */
export interface Timing {
  /** The port method that ran, e.g. `search`. */
  readonly op: string;
  /** How long it took, in milliseconds. */
  readonly ms: number;
  /** When it finished, for ordering. */
  readonly at: number;
  /** Whether it returned rather than threw. */
  readonly ok: boolean;
}

/**
 * How many samples to keep per operation.
 *
 * A bounded ring rather than a growing list: this runs for every call for as
 * long as the window is open, and an unbounded array would be a slow leak that
 * only shows up in a long session.
 */
const KEEP = 60;

/** Every measurement this window has taken. */
export class Timings {
  /** Samples, oldest first, capped per operation. */
  public samples = $state<Timing[]>([]);

  /** Record one completed call. */
  public record(op: string, ms: number, ok: boolean): void {
    const next = [...this.samples, { op, ms, at: Date.now(), ok }];
    const perOp = new Map<string, number>();
    // Walk from the newest backwards so the ones dropped are the oldest of
    // whichever operation is over its allowance, not the oldest overall — a
    // chatty poll would otherwise evict a rare call nobody could then see.
    const kept: Timing[] = [];
    for (let i = next.length - 1; i >= 0; i--) {
      const t = next[i];
      if (!t) continue;
      const n = (perOp.get(t.op) ?? 0) + 1;
      if (n > KEEP) continue;
      perOp.set(t.op, n);
      kept.push(t);
    }
    this.samples = kept.reverse();
  }

  /** Samples for one operation, oldest first. */
  public forOp(op: string): Timing[] {
    return this.samples.filter((t) => t.op === op);
  }

  /** Operations seen so far, most recently used first. */
  public get operations(): string[] {
    const seen: string[] = [];
    for (let i = this.samples.length - 1; i >= 0; i--) {
      const op = this.samples[i]?.op;
      if (op && !seen.includes(op)) seen.push(op);
    }
    return seen;
  }

  /** Forget everything measured so far. */
  public clear(): void {
    this.samples = [];
  }
}

/** The window's measurements. One instance, shared by every panel. */
export const timings = new Timings();

/**
 * Wrap a client so every call it makes is timed.
 *
 * A proxy rather than a timed copy of each method: there are two dozen of them,
 * and a hand-written wrapper would be two dozen chances to forget one — which
 * would show up as an operation that silently never appears in the panel.
 * Measurement is not a decision, so this does not violate "the bridge forwards".
 */
export function instrument<T extends object>(client: T): T {
  return new Proxy(client, {
    get(target, key, receiver) {
      const value = Reflect.get(target, key, receiver);
      if (typeof value !== "function") return value;
      return async (...args: unknown[]) => {
        const started = performance.now();
        try {
          const result = await value.apply(target, args);
          timings.record(String(key), performance.now() - started, true);
          return result;
        } catch (cause) {
          // Failures are timed too. A call that takes four seconds and then
          // fails is the interesting one, and dropping it would make the
          // chart look healthier the worse things got.
          timings.record(String(key), performance.now() - started, false);
          throw cause;
        }
      };
    },
  });
}

/**
 * What the metrics panel shows.
 *
 * Two sources with different standing, kept apart on purpose. The engine's own
 * report — backend, device, memory budget — is asked of the server and is true
 * of the machine. The latencies are measured in this window and are true only of
 * the calls this window made. Presenting them as one number would be the lie
 * this panel exists to avoid.
 */

import type { Capabilities, TelividbClient, Timing } from "$lib/api";
import { timings } from "$lib/api";

/** A summary of one operation's samples. */
export interface OpSummary {
  /** The port method, e.g. `search`. */
  readonly op: string;
  /** How many calls were measured. */
  readonly count: number;
  /** The most recent duration, in milliseconds. */
  readonly last: number;
  /** The median, which resists the one slow first call better than a mean. */
  readonly median: number;
  /** The slowest measured call. */
  readonly worst: number;
  /** How many failed. */
  readonly failures: number;
}

/** The metrics panel's state. */
export class MetricsState {
  /** What the engine reports about itself. Null until asked. */
  public capabilities = $state<Capabilities | null>(null);
  /** What went wrong asking, as the engine phrased it. */
  public error = $state<string | null>(null);

  public constructor(private readonly client: TelividbClient) {}

  /** Every operation measured so far, busiest first. */
  public get summaries(): OpSummary[] {
    return timings.operations
      .map((op) => summarize(op, timings.forOp(op)))
      .sort((a, b) => b.count - a.count);
  }

  /** Samples for one operation, oldest first. */
  public samples(op: string): Timing[] {
    return timings.forOp(op);
  }

  /** Ask the engine what it is running on. */
  public async load(): Promise<void> {
    this.error = null;
    try {
      this.capabilities = await this.client.capabilities();
    } catch (cause) {
      this.error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  /** Forget every measurement, so a change can be judged from a clean start. */
  public clear(): void {
    timings.clear();
  }
}

/** Reduce one operation's samples to the numbers worth showing. */
function summarize(op: string, samples: readonly Timing[]): OpSummary {
  const durations = samples.map((t) => t.ms).sort((a, b) => a - b);
  const mid = Math.floor(durations.length / 2);
  return {
    op,
    count: samples.length,
    last: samples[samples.length - 1]?.ms ?? 0,
    // Median rather than mean: the first call of any operation pays for a
    // connection and a cold path, and a mean over few samples is mostly that.
    median: durations[mid] ?? 0,
    worst: durations[durations.length - 1] ?? 0,
    failures: samples.filter((t) => !t.ok).length,
  };
}

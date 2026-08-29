/**
 * The model catalog exchange.
 *
 * These mirror the Rust DTOs in `telividb-desktop-ipc` field for field, because
 * Tauri serializes those as JSON and this is the other end of it. A rename on
 * either side is a runtime failure rather than a type error, so the two files
 * are worth reading together.
 */

/** One model the engine offers to install. */
export interface CatalogModel {
  /** Catalog id, which is what an install call names. */
  id: string;
  /** Name shown in the list. */
  displayName: string;
  /** What the model is good for, in a sentence. */
  description: string;
  /**
   * Page for the weights.
   *
   * Offered beside every model so a choice can be checked — licence,
   * provenance, benchmarks — against the publisher rather than against this
   * catalog's one-line summary.
   */
  repositoryUri: string;
  /** Exact size, which is what a progress bar divides by. */
  sizeBytes: number;
  /**
   * Components per vector.
   *
   * A collection's vector field must declare the same width, so this is what
   * the field offers once the model is installed.
   */
  dimensions: number;
  /** Longest input in tokens; anything longer is truncated. */
  contextLength: number;
  /** SPDX identifier for the weights' licence. */
  license: string;
  /** Whether this is the default offer for someone with no preference. */
  recommended: boolean;
  /** Whether the file is present locally and matches its digest. */
  installed: boolean;
}

/**
 * How far an installation has got.
 *
 * `state` is a string rather than the wire's integer so a template can compare
 * against something readable. `unknown` means the server reported a state this
 * build does not know — visible rather than silently treated as "not finished".
 */
export type InstallationState =
  | "pending"
  | "downloading"
  | "verifying"
  | "succeeded"
  | "failed"
  | "cancelled"
  | "unknown";

/** A running or finished installation. */
export interface Installation {
  /** Resource name, which is what a poll names. */
  name: string;
  /** How far it has got. */
  state: InstallationState;
  /** Bytes written so far, including any resumed from an earlier attempt. */
  progressBytes: number;
  /** Total expected. */
  totalBytes: number;
  /** Why it stopped, when it failed. Empty otherwise. */
  error: string;
}

/** Whether an installation has stopped, for any reason. */
export function isFinished(state: InstallationState): boolean {
  return state === "succeeded" || state === "failed" || state === "cancelled";
}

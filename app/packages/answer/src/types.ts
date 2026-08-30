/**
 * What answering needs, and what it produces.
 *
 * Answering runs here rather than in the engine: the SDKs worth using are the
 * ones the model vendors publish, they are TypeScript, and this window is
 * already a TypeScript runtime. The engine's part is the provider table and the
 * keychain — see `telividb-providers`.
 */

import type { Protection } from "./guard";

/** Where a provider runs, which decides what may be sent to it. */
export type Locality = "local" | "remote";

/**
 * A model host this build can talk to.
 *
 * Mirrors `telividb_providers::Provider` and arrives over IPC rather than being
 * declared again here. Two copies of this table would be two things that must
 * agree, and the engine's copy is the one the keychain is keyed by.
 */
export interface Provider {
  /** Stable id, used in configuration and to pick a client. */
  readonly id: string;
  /** Name shown to a person. */
  readonly displayName: string;
  /** Whether the prompt leaves the machine. */
  readonly locality: Locality;
  /** What it is, in the terms that decide whether to use it. */
  readonly note: string;
  /** Models it offers, most useful first. */
  readonly models: readonly string[];
  /** What its credential looks like, shown as placeholder text. */
  readonly credentialHint: string;
  /** Whether a credential is already stored for it. Never the value. */
  readonly configured: boolean;
}

/** One retrieved passage, as it will be shown and as it will be cited. */
export interface Passage {
  /** The point this came from, so the citation can be followed. */
  readonly id: string;
  /** The text itself. */
  readonly text: string;
  /** Similarity, for the reader to judge the retrieval by. */
  readonly score: number;
}

/** A question, the passages found for it, and who should answer. */
export interface Ask {
  /** What was typed. */
  readonly question: string;
  /** The space the passages came from, named for the refusal message. */
  readonly space: string;
  /**
   * How that space is protected.
   *
   * Carried on the request rather than passed beside it so the guard cannot be
   * skipped by a caller who did not know to call it — building an `Ask` at all
   * means stating this.
   */
  readonly protection: Protection;
  /** What retrieval returned, best first. */
  readonly passages: readonly Passage[];
  /** Which provider answers. */
  readonly provider: Provider;
  /** Which of its models. */
  readonly model: string;
  /** The key, held only for the duration of the call. */
  readonly credential: string;
}

/**
 * One piece of a streaming answer.
 *
 * Streamed rather than awaited because a grounded answer over five passages
 * takes seconds, and a window that shows nothing for seconds reads as broken.
 */
export interface AnswerChunk {
  /** Text to append. */
  readonly text: string;
}

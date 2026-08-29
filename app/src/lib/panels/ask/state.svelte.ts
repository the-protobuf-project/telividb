/**
 * What the ask panel is doing right now.
 *
 * The orchestration lives here rather than in a Tauri command, because it is
 * sequencing rather than a decision: make sure the collection exists, search it,
 * then remember what was asked. Each step is a call the client already offers,
 * and keeping them here leaves the bridge forwarding.
 */

import type { SearchHit, TelividbClient } from "$lib/api";

/** The collection this panel reads and writes. Created on first use. */
const COLLECTION = "notes";

/** The preset supplying its schema, and the field its text lands in. */
const PRESET = "notes";
const FIELD = "text";

/** How many neighbours to show. Enough to judge, few enough to read. */
const TOP_K = 5;

/** One exchange: what was asked, and what came back. */
export interface Exchange {
  /** What was typed. */
  readonly question: string;
  /** What the collection returned, best first. */
  readonly hits: readonly SearchHit[];
  /** Whether this text was new to the collection. */
  readonly stored: boolean;
}

/** The ask panel's state. */
export class AskState {
  /** What is typed but not yet sent. */
  public draft = $state("");
  /** Exchanges so far, newest first. */
  public history = $state<Exchange[]>([]);
  /** What went wrong, as the engine phrased it. */
  public error = $state<string | null>(null);
  /** Whether a question is in flight. */
  public asking = $state(false);

  /** Whether the collection has been ensured this session. */
  private ready = false;

  public constructor(private readonly client: TelividbClient) {}

  /** Whether the draft is worth sending. */
  public get canAsk(): boolean {
    return this.draft.trim().length > 0 && !this.asking;
  }

  /**
   * Search what has been said so far, then add this to it.
   *
   * Searching *before* storing is deliberate: otherwise every question returns
   * itself as the best match, which is true and useless.
   */
  public async ask(): Promise<void> {
    const question = this.draft.trim();
    if (!question || this.asking) return;

    this.asking = true;
    this.error = null;
    try {
      await this.ensureCollection();

      const found = await this.client.search({
        collection: COLLECTION,
        field: FIELD,
        text: question,
        k: TOP_K,
      });

      await this.client.importPoints({
        collection: COLLECTION,
        field: FIELD,
        rows: [{ id: crypto.randomUUID(), text: question }],
      });

      this.history = [
        { question, hits: found.hits, stored: true },
        ...this.history,
      ];
      this.draft = "";
    } catch (cause) {
      this.error = String(cause);
    } finally {
      this.asking = false;
    }
  }

  /**
   * Create the collection if it is not there, at the resident model's width.
   *
   * The width is asked of the engine rather than assumed. A vector field is
   * bound to one model, so a collection declared at the wrong width is one
   * nothing can write to — and the widths in use differ: the BERT-family models
   * are 768, Qwen3-Embedding is 1024.
   */
  private async ensureCollection(): Promise<void> {
    if (this.ready) return;

    const existing = await this.client.listCollections();
    if (existing.some((c) => c.id === COLLECTION)) {
      this.ready = true;
      return;
    }

    const models = await this.client.listModels();
    const resident = models.find((m) => m.resident);
    if (!resident) {
      throw new Error(
        "No embedding model is loaded yet, so there is nothing to turn text " +
          "into vectors. Install one from Models, or wait for it to finish " +
          "loading if you just started the app.",
      );
    }

    await this.client.createCollection({
      preset: PRESET,
      collection: COLLECTION,
      dimensions: resident.dimensions,
    });
    this.ready = true;
  }
}

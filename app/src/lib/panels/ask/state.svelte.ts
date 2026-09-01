/**
 * What the ask panel is doing right now.
 *
 * The orchestration lives here rather than in a Tauri command, because it is
 * sequencing rather than a decision: make sure the collection exists, search it,
 * then remember what was asked. Each step is a call the client already offers,
 * and keeping them here leaves the bridge forwarding.
 */

import type { TelividbClient } from "$lib/api";
import type { Protection, Provider } from "@telividb/answer";
import { Exchange } from "./exchange.svelte";

export { Exchange };

/** The collection this panel reads and writes. Created on first use. */
const COLLECTION = "notes";

/** The preset supplying its schema, and the field its text lands in. */
const PRESET = "notes";
const FIELD = "text";

/** How many neighbours to show. Enough to judge, few enough to read. */
const TOP_K = 5;

/**
 * How the notes collection is protected.
 *
 * Unprotected until spaces are wired through, and named here rather than assumed inside
 * the guard so the day it becomes a real value there is one line to change. A
 * literal `"open"` buried in a call is the version that gets missed.
 */
const PROTECTION: Protection = "none";

/** The ask panel's state. */
export class AskState {
  /** What is typed but not yet sent. */
  public draft = $state("");
  /** Exchanges so far, newest first. */
  public history = $state<Exchange[]>([]);
  /** Model hosts this build knows, as the engine reports them. */
  public providers = $state<Provider[]>([]);
  /** Which one answers. Null until the list has loaded. */
  public provider = $state<Provider | null>(null);
  /** Which of its models. */
  public model = $state("");
  /** What went wrong, as the engine phrased it. */
  public error = $state<string | null>(null);
  /** Whether a question is in flight. */
  public asking = $state(false);

  /** Whether the collection has been ensured this session. */
  private ready = false;

  public constructor(private readonly client: TelividbClient) {}

  /**
   * Load the provider list and pick a default.
   *
   * Prefers a configured one, then a local one, because Ollama needs no key and
   * so is the only provider that can work on a machine nobody has configured.
   * Failing quietly is right here: retrieval works without a provider, and the
   * panel says what is missing when a question is asked.
   */
  public async loadProviders(): Promise<void> {
    try {
      this.providers = await this.client.listProviders();
      const preferred =
        this.providers.find((p) => p.configured && p.locality === "local") ??
        this.providers.find((p) => p.configured) ??
        this.providers[0];
      if (preferred) this.select(preferred);
    } catch {
      this.providers = [];
    }
  }

  /** Choose a provider, resetting the model to its first. */
  public select(provider: Provider): void {
    this.provider = provider;
    this.model = provider.models[0] ?? "";
  }

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

      const exchange = new Exchange(question, found.hits);
      this.history = [exchange, ...this.history];
      this.draft = "";

      // Not awaited: retrieval is done and the hits are on screen, so the
      // composer is usable again while the answer streams in beneath them.
      // Showing what was retrieved before the prose arrives is the point of this
      // panel, not a loading state to get past.
      if (this.provider && this.model) {
        void exchange.write(
          this.client,
          this.provider,
          this.model,
          COLLECTION,
          PROTECTION,
        );
      }
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

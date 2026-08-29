/**
 * Creating a collection from a shipped preset.
 *
 * A class so the rules are testable without a DOM: which ids are acceptable is
 * the engine's constraint, not a component's, and asserting it directly is
 * cheaper than driving a form.
 */

import type { CreateCollectionRequest, Preset, TelividbClient } from "$lib/api";

/**
 * Ids the engine will accept as a final path segment.
 *
 * A resource name is the only portable identity here, so a segment that needs
 * escaping to appear in one is refused at the edge rather than after it has
 * been written into an archive.
 */
const VALID_ID = /^[a-z][a-z0-9-]{0,62}$/;

/** One collection-creation form. */
export class CollectionState {
  /** Schemas this build can create from. */
  public presets = $state<Preset[]>([]);
  /** Which preset the form is on. */
  public preset = $state("");
  /** The collection id being typed. */
  public collection = $state("");
  /** What went wrong, as the engine phrased it. */
  public error = $state<string | null>(null);
  /** The resource name of the last collection created. */
  public created = $state<string | null>(null);
  /** Whether a create is in flight. */
  public running = $state(false);

  /** Whether the form can be submitted. */
  public readonly ready = $derived(
    this.preset !== "" && VALID_ID.test(this.collection),
  );

  /**
   * Why the id is unacceptable, or null while it is fine.
   *
   * Empty reads as "not started" rather than "wrong", so it carries no
   * complaint — a form that scolds before anything is typed is noise.
   */
  public readonly idProblem = $derived(
    this.collection === "" || VALID_ID.test(this.collection)
      ? null
      : "Lowercase letters, digits and hyphens; starting with a letter.",
  );

  constructor(private readonly client: TelividbClient) {}

  /** Load the preset list. */
  public async loadPresets(): Promise<void> {
    try {
      this.presets = await this.client.listPresets();
      const first = this.presets.at(0);
      if (!this.preset && first) this.preset = first.id;
    } catch (e) {
      this.error = String(e);
    }
  }

  /** Create the collection, if the form is ready. */
  public async create(): Promise<string | null> {
    if (!this.ready || this.running) return null;
    this.running = true;
    this.error = null;
    try {
      const request: CreateCollectionRequest = {
        preset: this.preset,
        collection: this.collection,
      };
      this.created = await this.client.createCollection(request);
      return this.created;
    } catch (e) {
      this.error = String(e);
      return null;
    } finally {
      this.running = false;
    }
  }
}

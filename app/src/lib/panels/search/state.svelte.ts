/**
 * What a search panel is doing right now.
 *
 * A class rather than component state, for two reasons. It is testable without
 * a DOM — the rules below about what makes a query runnable are worth asserting
 * directly. And it takes the client as a constructor argument, so a test drives
 * it with a stub and the panel is left with nothing to do but render.
 */

import type { CollectionSummary, SearchResponse, TelividbClient } from "$lib/api";

/** The default `k`: enough to judge a result set, cheap enough to be instant. */
const DEFAULT_K = 10;

/** One search panel's state. */
export class SearchState {
  /** Collections offered in the picker. */
  public collections = $state<CollectionSummary[]>([]);
  /** Collection to search. */
  public collection = $state("");
  /** Named vector field to search. */
  public field = $state("");
  /** Query text. */
  public text = $state("");
  /** How many neighbours to ask for. */
  public k = $state(DEFAULT_K);

  /** The last answer, or null before the first query. */
  public results = $state<SearchResponse | null>(null);
  /** What went wrong, as the server phrased it. */
  public error = $state<string | null>(null);
  /** Whether a query is in flight. */
  public running = $state(false);

  /**
   * Whether the query is runnable.
   *
   * All three are required by the server rather than by taste: a query without
   * a field has no model to encode against, and one without a collection has
   * nothing to search.
   */
  public readonly ready = $derived(
    this.collection !== "" && this.field !== "" && this.text !== "",
  );

  constructor(private readonly client: TelividbClient) {}

  /**
   * Load the collection picker.
   *
   * A failure is reported rather than swallowed: an empty picker and an
   * unreachable engine look identical, and only one of them is the user's
   * problem to solve.
   */
  async loadCollections(): Promise<void> {
    try {
      this.collections = await this.client.listCollections();
      // Index access yields `T | undefined` under noUncheckedIndexedAccess,
      // and the compiler is right to insist: an engine with no collections is
      // the ordinary state of a fresh install, not an edge case.
      const first = this.collections.at(0);
      if (!this.collection && first) {
        this.collection = first.id;
      }
    } catch (e) {
      this.error = String(e);
    }
  }

  /** Run the query, if it is runnable and not already running. */
  async run(): Promise<void> {
    if (!this.ready || this.running) return;
    this.running = true;
    this.error = null;
    try {
      this.results = await this.client.search({
        collection: this.collection,
        field: this.field,
        text: this.text,
        k: this.k,
      });
    } catch (e) {
      // The server refuses a text query when no model is resident. That is a
      // real limit with a real remedy, so it is shown as the server phrased it
      // rather than flattened into "search failed".
      this.error = String(e);
      this.results = null;
    } finally {
      this.running = false;
    }
  }
}

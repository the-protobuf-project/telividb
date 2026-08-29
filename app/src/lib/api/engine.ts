/**
 * What the engine this window supervises can currently do.
 */

/** Engine capabilities, as the app knows them at startup. */
export interface Capabilities {
  /**
   * Whether an embedding model is loaded.
   *
   * False means text is refused for storage as well as for search, and only
   * precomputed vectors work. Asked before offering an import rather than
   * discovered after one fails.
   */
  has_model: boolean;
  /** Where the engine is listening. */
  address: string;
}

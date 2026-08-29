/**
 * The search exchange.
 *
 * These mirror the Rust DTOs in `telividb-desktop-ipc` field for field, because
 * Tauri serializes those as JSON and this is the other end of it. A rename on
 * either side is a runtime failure rather than a type error, so the two files
 * are worth reading together.
 */

/** A query for one collection. */
export interface SearchRequest {
  /** Collection to search. */
  collection: string;
  /**
   * Named vector field to search.
   *
   * Required, and rightly: each field carries its own model and metric, so a
   * query only means something against the field it was encoded for.
   */
  field: string;
  /**
   * Text for the server to encode.
   *
   * Exactly one of this and `vector` carries the query. Text needs a model
   * resident on the server; without one the query is refused, and the window
   * says so rather than showing an empty result.
   */
  text?: string;
  /** A query vector the caller already has. */
  vector?: number[];
  /**
   * How many neighbours to return.
   *
   * The `k` of a nearest-neighbour search, not a display limit: it decides how
   * much work the index does.
   */
  k: number;
}

/** One matching point. */
export interface SearchHit {
  /** The point's id within its collection. */
  id: string;
  /** Similarity, on the scale of the field's own metric. */
  score: number;
  /** Text the point carries inline, when it carries any. */
  text: string | null;
}

/** What a search answered. */
export interface SearchResponse {
  /** Matching points, nearest first. */
  hits: SearchHit[];
  /**
   * Whether every source answered.
   *
   * Shown rather than assumed. A reader handed only hits cannot tell "nothing
   * matched" from "nothing you can currently see matched", and those are
   * different answers.
   */
  complete: boolean;
  /** Vaults that were locked, by name. Names only, never contents. */
  locked_vaults: string[];
}

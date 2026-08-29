/**
 * Bringing rows in from a file.
 *
 * Text rather than vectors: a CSV has none, and the server encodes through the
 * field's own model — which is also the only path that gives the stored vectors
 * provenance the engine can check.
 */

/** One row on its way in. */
export interface ImportRow {
  /** The point's id within the collection. */
  id: string;
  /** The text to embed and store. */
  text: string;
}

/** A batch of rows for one collection. */
export interface ImportRequest {
  /** Collection to write into. */
  collection: string;
  /** Named vector field the text is encoded for. */
  field: string;
  /** The rows. */
  rows: ImportRow[];
}

/** What an import wrote. */
export interface ImportResponse {
  /** How many points were created. */
  written: number;
}

/** One stored point, as a table row. */
export interface PointRow {
  /** The point's id within its collection. */
  id: string;
  /** Text the point carries inline, when it carries any. */
  text: string | null;
}

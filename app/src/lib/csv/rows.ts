/**
 * Turning parsed columns into points.
 *
 * Kept apart from the parser because it is a different decision: which column
 * is the id and which is the text is the user's to make, and the parser should
 * not have an opinion about it.
 */

import type { ImportRow } from "$lib/api";
import type { ParsedCsv } from "./parse";

/** Which columns carry the id and the text. */
export interface ColumnMapping {
  /** Column supplying each point's id. */
  id: string;
  /** Column supplying the text to embed. */
  text: string;
}

/** Maps parsed rows onto the shape the engine takes. */
export class RowMapper {
  /** The parsed file. */
  private readonly parsed: ParsedCsv;

  constructor(parsed: ParsedCsv) {
    this.parsed = parsed;
  }

  /** Column names, for a picker. */
  public get columns(): readonly string[] {
    return this.parsed.columns;
  }

  /**
   * Map rows through `mapping`.
   *
   * Rows whose text is empty are dropped rather than sent: a point with nothing
   * to embed produces no vector, so it would be written and never found —
   * present in a listing and absent from every search, which reads as a bug in
   * the engine rather than an empty cell in a file.
   */
  public rows(mapping: ColumnMapping): ImportRow[] {
    const idAt = this.parsed.columns.indexOf(mapping.id);
    const textAt = this.parsed.columns.indexOf(mapping.text);
    if (idAt < 0 || textAt < 0) return [];

    return this.parsed.rows
      .map((row, index) => ({
        // A file without stable ids still imports: the row number is stable
        // within one file, which is enough to write and to look back at.
        id: (row[idAt] ?? "").trim() || `row-${index + 1}`,
        text: (row[textAt] ?? "").trim(),
      }))
      .filter((row) => row.text !== "");
  }
}

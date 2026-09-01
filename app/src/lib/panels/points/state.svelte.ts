/**
 * Importing a file, and listing what a collection holds.
 *
 * Import and listing share a class because they share a collection and a
 * refresh: an import that did not update the table beside it would leave a
 * reader unsure whether it worked.
 */

import type { PointRow, TelividbClient } from "$lib/api";
import { CsvError, CsvParser, type ParsedCsv } from "$lib/csv/parse";
import { RowMapper, type ColumnMapping } from "$lib/csv/rows";

/** One points panel. */
export class PointsState {
  /** Collection being read and written. */
  public collection = $state("");
  /** Named vector field the imported text is encoded for. */
  public field = $state("text");
  /** Rows currently in the collection. */
  public rows = $state<PointRow[]>([]);
  /** The parsed file, once one is chosen. */
  public parsed = $state<ParsedCsv | null>(null);
  /** Which columns carry the id and the text. */
  public mapping = $state<ColumnMapping>({ id: "", text: "" });
  /** What went wrong, as the parser or the engine phrased it. */
  public error = $state<string | null>(null);
  /**
   * Whether a fetch is in flight.
   *
   * Models has had this and Points has not, so switching collections showed the
   * previous collection's rows — or "No points yet" — while the new ones were
   * still loading. Two near-identical list panels behaving differently is worse
   * than either behaviour on its own.
   */
  public loading = $state(false);
  /** What the last import wrote. */
  public imported = $state<number | null>(null);
  /** Whether an import or a refresh is in flight. */
  public running = $state(false);

  /** Column names from the parsed file, for the pickers. */
  public readonly columns = $derived(this.parsed?.columns ?? []);

  /** How many rows would be written, after empty text is dropped. */
  public readonly importable = $derived(
    this.parsed === null
      ? 0
      : new RowMapper(this.parsed).rows(this.mapping).length,
  );

  /** Whether the import can run. */
  public readonly ready = $derived(
    this.collection !== "" &&
      this.field !== "" &&
      this.mapping.id !== "" &&
      this.mapping.text !== "" &&
      this.importable > 0,
  );

  constructor(private readonly client: TelividbClient) {}

  /**
   * Parse `text` as CSV and guess the column mapping.
   *
   * The guess is a starting point, not a decision: a wrong column would embed
   * the wrong thing, so both pickers stay visible and changeable.
   */
  public read(text: string): void {
    this.error = null;
    this.imported = null;
    try {
      const parsed = new CsvParser().parse(text);
      this.parsed = parsed;
      this.mapping = {
        id: guess(parsed.columns, ["id", "key", "name"]) ?? parsed.columns[0] ?? "",
        text:
          guess(parsed.columns, ["text", "body", "content", "message"]) ??
          parsed.columns[1] ??
          parsed.columns[0] ??
          "",
      };
    } catch (e) {
      // A parse failure names the line, which is the only thing that makes a
      // malformed file fixable.
      this.parsed = null;
      this.error =
        e instanceof CsvError ? `Line ${e.line}: ${e.message}` : String(e);
    }
  }

  /** Write the mapped rows, then refresh the table. */
  public async importRows(): Promise<void> {
    if (!this.ready || this.parsed === null || this.running) return;
    this.running = true;
    this.error = null;
    try {
      const rows = new RowMapper(this.parsed).rows(this.mapping);
      const result = await this.client.importPoints({
        collection: this.collection,
        field: this.field,
        rows,
      });
      this.imported = result.written;
      this.parsed = null;
      await this.refresh();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.running = false;
    }
  }

  /** Reload the table. */
  public async refresh(): Promise<void> {
    // Before the guard, not after: an early return with `loading` already set
    // leaves the panel showing skeletons for a collection it never fetched.
    if (this.collection === "") {
      this.rows = [];
      return;
    }

    this.loading = true;
    // Cleared before the attempt rather than only on failure. A stale error
    // outlived every later success, so one bad request left the panel looking
    // broken for the rest of the session.
    this.error = null;
    try {
      this.rows = await this.client.listPoints(this.collection);
    } catch (cause) {
      this.error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      this.loading = false;
    }
  }
}

/** The first column whose name matches one of `wanted`, case-insensitively. */
function guess(
  columns: readonly string[],
  wanted: readonly string[],
): string | undefined {
  return columns.find((c) => wanted.includes(c.trim().toLowerCase()));
}

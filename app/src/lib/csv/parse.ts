/**
 * Reading a CSV into rows the engine can take.
 *
 * A small parser rather than a dependency, and worth being honest about which
 * part of the format it covers: quoted fields, embedded commas, escaped quotes
 * (`""`), and CRLF. What it does not do is anything a spreadsheet would —
 * type inference, locale-aware numbers, multi-character delimiters. A file that
 * needs those needs a real CSV library, and this refuses rather than
 * mis-parsing.
 */

/** A parsed file: a header row and the rows beneath it. */
export interface ParsedCsv {
  /** Column names, in file order. */
  readonly columns: readonly string[];
  /** Rows, each the same length as `columns`. */
  readonly rows: readonly (readonly string[])[];
}

/** What went wrong reading a file. */
export class CsvError extends Error {
  /** The one-based line the failure was found on. */
  public readonly line: number;

  constructor(message: string, line: number) {
    super(message);
    this.name = "CsvError";
    this.line = line;
  }
}

/** Reads delimited text into rows. */
export class CsvParser {
  /** The field separator. */
  private readonly delimiter: string;

  constructor(delimiter = ",") {
    this.delimiter = delimiter;
  }

  /**
   * Parse `text`, treating the first row as the header.
   *
   * @throws {CsvError} when a row has a different width from the header.
   * A short row is almost always an unquoted comma rather than a missing
   * value, and guessing which column to leave empty would put text under the
   * wrong heading — silently, and permanently once it is embedded.
   */
  public parse(text: string): ParsedCsv {
    const records = this.records(text);
    const [header, ...rest] = records;
    if (!header || header.length === 0) {
      throw new CsvError("the file has no header row.", 1);
    }

    rest.forEach((row, index) => {
      if (row.length !== header.length) {
        throw new CsvError(
          `row has ${row.length} field(s) but the header has ${header.length}. ` +
            `An unquoted comma inside a value is the usual cause.`,
          index + 2,
        );
      }
    });

    return { columns: header, rows: rest };
  }

  /** Split into records, honouring quotes and embedded newlines. */
  private records(text: string): string[][] {
    const out: string[][] = [];
    let row: string[] = [];
    let field = "";
    let quoted = false;

    for (let i = 0; i < text.length; i += 1) {
      const ch = text[i];

      if (quoted) {
        if (ch !== '"') {
          field += ch;
        } else if (text[i + 1] === '"') {
          // A doubled quote is one literal quote, which is how the format
          // escapes them.
          field += '"';
          i += 1;
        } else {
          quoted = false;
        }
        continue;
      }

      if (ch === '"' && field === "") {
        quoted = true;
      } else if (ch === this.delimiter) {
        row.push(field);
        field = "";
      } else if (ch === "\n" || ch === "\r") {
        // Consume CRLF as one break rather than two.
        if (ch === "\r" && text[i + 1] === "\n") i += 1;
        row.push(field);
        field = "";
        out.push(row);
        row = [];
      } else {
        field += ch;
      }
    }

    // A file that does not end in a newline still has a last row.
    if (field !== "" || row.length > 0) {
      row.push(field);
      out.push(row);
    }

    // Trailing blank lines are not rows.
    return out.filter((r) => r.length > 1 || r[0] !== "");
  }
}

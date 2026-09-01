/**
 * Filtering and paging one list.
 *
 * A class rather than a pair of helpers because the two interact: typing must
 * return to the first page, or the reader searches, sees "no results", and is
 * actually looking at page 4 of a two-page result. That coupling is the whole
 * reason this exists as one object.
 *
 * The match is a case-insensitive substring across whichever fields the caller
 * names — deliberately not fuzzy. A catalogue of sixteen models and a group of
 * four people are small enough that a substring finds what someone meant, and
 * fuzzy matching returns confident nonsense on short queries.
 */

/** How a row is turned into the text a search runs against. */
export type Searchable<T> = (
  item: T,
) => readonly (string | number | null | undefined)[];

/** One list's query, page and the slice they produce. */
export class Paged<T> {
  /** What is typed. Private, so the setter below cannot be bypassed. */
  private q = $state("");
  /** Which page is showing, zero-based. */
  public page = $state(0);

  /** What is typed in the search box. */
  public get query(): string {
    return this.q;
  }

  /**
   * Setting it returns to the first page.
   *
   * A getter/setter pair rather than a plain field so `bind:value` cannot skip
   * it. Without the reset, someone on page 4 who types a query sees "no
   * results" while looking at page 4 of a two-page result — the list is right
   * and the reader concludes their search failed.
   */
  public set query(value: string) {
    this.q = value;
    this.page = 0;
  }

  public constructor(
    private readonly source: () => readonly T[],
    private readonly fields: Searchable<T>,
    /** How many rows fit before paging is worth the reader's attention. */
    public readonly size = 10,
  ) {}

  /** Everything matching the query, unpaged. */
  public get matches(): T[] {
    const q = this.q.trim().toLowerCase();
    const all = [...this.source()];
    if (!q) return all;
    return all.filter((item) =>
      this.fields(item).some((f) => String(f ?? "").toLowerCase().includes(q)),
    );
  }

  /** How many pages the matches fill. At least one, so "1 of 1" is never "1 of 0". */
  public get pages(): number {
    return Math.max(1, Math.ceil(this.matches.length / this.size));
  }

  /** The rows to render. */
  public get rows(): T[] {
    // Clamped rather than trusted: deleting the last row of the last page would
    // otherwise leave the reader on a page that no longer exists.
    const at = Math.min(this.page, this.pages - 1);
    return this.matches.slice(at * this.size, at * this.size + this.size);
  }

  /** Whether paging is worth showing at all. */
  public get paged(): boolean {
    return this.matches.length > this.size;
  }

  /** Step a page, within bounds. */
  public go(delta: number): void {
    this.page = Math.max(0, Math.min(this.pages - 1, this.page + delta));
  }
}

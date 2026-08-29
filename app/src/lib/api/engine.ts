/**
 * What the engine this window supervises can currently do.
 */

/** The compute environment, as the process found it. */
export interface Environment {
  /** The selected backend: `metal`, `cuda`, `cpu`, and so on. */
  backend: string;
  /**
   * Device memory in bytes, when the backend reports it.
   *
   * Null rather than zero when unknown — a host backend has no separate device
   * memory, which is different from having none.
   */
  total_bytes: number | null;
  /** Free device memory in bytes, when the backend reports it. */
  free_bytes: number | null;
  /** Whether the selection came from `TELIVIDB_DEVICE` rather than detection. */
  overridden: boolean;
}

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
  /** The compute environment, detected rather than configured. */
  environment: Environment;
  /** The directory holding segments, the write-ahead log and metadata. */
  data_dir: string;
}

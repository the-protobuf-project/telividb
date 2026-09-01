/**
 * What the engine this window talks to can currently do.
 *
 * Mirrors the Rust DTOs in `telividb-desktop-ipc` field for field.
 */

/**
 * The compute environment, as the engine selected it.
 *
 * **Reported by the engine, not detected here.** The desktop build could detect
 * it locally only because it links the engine in-process; a browser talking to a
 * Linux daemon cannot, and both must reach the same answer. A client that
 * detected its own hardware would be describing the wrong machine.
 */
export interface System {
  /**
   * The selected backend: `metal`, `cuda`, `cpu`, and so on.
   *
   * The one fact no orchestrator sees from outside the process. A build that
   * fell back to the host looks healthy, allocated and idle from every angle
   * except this one, so it is stated rather than inferred from felt slowness.
   */
  backend: string;
  /** Human-readable device description. */
  device: string;
  /** Device memory ceiling this process will use. Zero when none is reported. */
  budgetLimitBytes: number;
  /**
   * Device memory held by resident models and indexes.
   *
   * Zero until the engine tracks it. Reporting the device's own used figure
   * would credit this process with every other process's allocations, so zero
   * is the honest answer to a question nothing yet answers.
   */
  budgetUsedBytes: number;
  /**
   * Whether the ceiling was `measured`, `estimated` or `configured`.
   *
   * An estimate on a discrete card overshoots, so anyone sizing a deployment
   * has to be able to tell which kind of number they are reading.
   */
  budgetSource: "measured" | "estimated" | "configured";
  /** Version of the engine build that answered. */
  version: string;
}

/** Engine capabilities, as the app knows them. */
export interface Capabilities {
  /**
   * Whether an embedding model is loaded.
   *
   * False means text is refused for storage as well as for search, and only
   * precomputed vectors work.
   */
  has_model: boolean;
  /** Where the engine is listening. */
  address: string;
  /** The compute environment the engine reported. */
  environment: System;
  /** The directory holding segments, the write-ahead log and metadata. */
  data_dir: string;
}

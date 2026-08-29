/**
 * Collection schemas the app ships with.
 *
 * The engine takes a compiled descriptor set, never a `.proto`, and a window
 * cannot compile one. These are what let a fresh install create a collection at
 * all rather than only search collections made elsewhere.
 */

/** A schema a collection can be created from without a toolchain. */
export interface Preset {
  /** Stable key, passed back to create from it. */
  id: string;
  /** What the reader sees in the picker. */
  display_name: string;
  /** What this schema is for, in one sentence. */
  description: string;
  /** The named vector field a collection of this shape carries. */
  field: string;
}

/** A request to create a collection from a preset. */
export interface CreateCollectionRequest {
  /** Which preset supplies the schema. */
  preset: string;
  /** The collection's id, forming the final segment of its name. */
  collection: string;
  /**
   * Width to declare for the text field.
   *
   * Must match the model that will embed into it — a field is bound to one
   * model identity, and a mismatch is refused on the first write. Omitted falls
   * back to the BERT-family width, which is right only if that is what is
   * loaded: Qwen3-Embedding is 1024.
   */
  dimensions?: number;
}

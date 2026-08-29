/**
 * What the window can ask of an engine.
 *
 * A port, not a class. The desktop app satisfies it over Tauri IPC; a browser
 * would satisfy it over gRPC-web against a remote daemon. Panels are written
 * against this and mention neither, which is what lets one codebase reach the
 * same behaviour both ways.
 */

import type { CollectionSummary } from "./collection";
import type { SearchRequest, SearchResponse } from "./search";
import type { CreateCollectionRequest, Preset } from "./preset";
import type { ImportRequest, ImportResponse, PointRow } from "./import";
import type { Capabilities } from "./engine";

/** An engine this window can reach. */
export interface TelividbClient {
  /** Collections this engine holds. */
  listCollections(): Promise<CollectionSummary[]>;
  /** Search one collection. */
  search(request: SearchRequest): Promise<SearchResponse>;
  /** Where the engine is listening, for an external tool. */
  engineAddress(): Promise<string>;
  /** What the engine can currently do. */
  capabilities(): Promise<Capabilities>;
  /** The schemas this build can create a collection from. */
  listPresets(): Promise<Preset[]>;
  /** Create a collection from a preset. Resolves to its resource name. */
  createCollection(request: CreateCollectionRequest): Promise<string>;
  /** Write a batch of text rows into one collection. */
  importPoints(request: ImportRequest): Promise<ImportResponse>;
  /** The points of one collection. */
  listPoints(collection: string): Promise<PointRow[]>;
}

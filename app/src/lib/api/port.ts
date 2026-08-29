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

/** An engine this window can reach. */
export interface TelividbClient {
  /** Collections this engine holds. */
  listCollections(): Promise<CollectionSummary[]>;
  /** Search one collection. */
  search(request: SearchRequest): Promise<SearchResponse>;
  /** Where the engine is listening, for an external tool. */
  engineAddress(): Promise<string>;
}

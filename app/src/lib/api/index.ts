/**
 * The client this build talks to, and the shapes it exchanges.
 *
 * One import for every panel, so selecting a transport is a change on the last
 * line of this file and nowhere else. Today there is one adapter; a browser
 * build would export the gRPC-web one instead.
 */

export type { CollectionSummary } from "./collection";
export type { SearchHit, SearchRequest, SearchResponse } from "./search";
export type { Capabilities, Environment } from "./engine";
export type { CreateCollectionRequest, Preset } from "./preset";
export type { ImportRequest, ImportResponse, ImportRow, PointRow } from "./import";
export type { TelividbClient } from "./port";

import { TauriClient } from "./tauri";
import type { TelividbClient } from "./port";

export { TauriClient } from "./tauri";

/**
 * The engine this build talks to.
 *
 * One instance, constructed here, so selecting a transport is a change on this
 * line and nowhere else. A browser build would construct the gRPC-web adapter
 * instead and every panel would be unchanged.
 */
export const client: TelividbClient = new TauriClient();

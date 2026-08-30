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
export type { Organization, Project, Space } from "./tenancy";
export { resourceId, suggestId } from "./tenancy";

import { TauriClient } from "./tauri";
import { GrpcWebClient } from "./grpc";
import type { TelividbClient } from "./port";

export { TauriClient } from "./tauri";
export { GrpcWebClient } from "./grpc";

/**
 * Whether this page is running inside the desktop app.
 *
 * Tauri injects `__TAURI_INTERNALS__` before any application code runs, so its
 * presence is a fact about the host rather than a guess. Checked on `window`
 * because this module is imported during server-side rendering too, where there
 * is no window at all.
 */
export function inDesktopApp(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Where a browser build reaches the daemon.
 *
 * Same origin by default: served by the daemon itself, the page and the API
 * share a host, and assuming anything else would need configuration nobody has
 * supplied yet.
 */
function daemonOrigin(): string {
  return typeof window === "undefined" ? "" : window.location.origin;
}

/**
 * The engine this build talks to.
 *
 * **Inside the desktop app this is always IPC; on the web it is always gRPC.**
 * The desktop build links the engine into its own process, so IPC is a call
 * across a thread boundary with no socket to secure and no origin to configure.
 * A browser has no engine and must reach a daemon over the network.
 *
 * Selected here and nowhere else, so every panel is written against the port
 * and none of them knows which transport carried the call.
 */
export function resolveClient(): TelividbClient {
  return inDesktopApp() ? new TauriClient() : new GrpcWebClient(daemonOrigin());
}

/** The engine this build talks to. One instance, selected at load. */
export const client: TelividbClient = resolveClient();
export * from "./model";

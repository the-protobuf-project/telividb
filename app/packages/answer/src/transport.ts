/**
 * Which `fetch` the provider SDKs use.
 *
 * A webview enforces CORS, and `tauri://localhost` is a foreign origin to every
 * provider — so a plain `fetch` to `api.anthropic.com` is refused before it is
 * sent. Tauri's HTTP plugin performs the request in Rust, where CORS does not
 * apply, and the SDKs all accept a `fetch` to use instead of the global one.
 *
 * In a browser (the Linux daemon's UI) there is no plugin and the global `fetch`
 * is used, which means that deployment depends on each provider's own CORS
 * support. That is a real difference between the two targets, and it is here
 * rather than hidden inside an adapter.
 */

/** The subset of `fetch` the SDKs require. */
export type FetchLike = typeof globalThis.fetch;

/** Whether this window is the desktop app rather than a browser tab. */
function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * The `fetch` to hand an SDK.
 *
 * Imported lazily so a browser build never pulls the plugin in, and so a missing
 * plugin degrades to the global rather than throwing at module load.
 */
export async function resolveFetch(): Promise<FetchLike> {
  if (!inTauri()) return globalThis.fetch.bind(globalThis);
  try {
    const http = await import("@tauri-apps/plugin-http");
    return http.fetch as unknown as FetchLike;
  } catch {
    return globalThis.fetch.bind(globalThis);
  }
}

/** Temperature for every provider: low, not zero. */
export const TEMPERATURE = 0.2;

/**
 * Ceiling on an answer, required by Anthropic and applied everywhere for parity.
 *
 * Generous for a grounded answer over five passages, small enough that a runaway
 * one stops rather than billing until it is noticed.
 */
export const MAX_TOKENS = 1024;

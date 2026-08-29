/**
 * The port, satisfied over Tauri IPC.
 *
 * A class rather than an object of functions. The behaviour belongs to a type,
 * so a reader finds it by looking at the thing it operates on — the same rule
 * the Rust side follows — and a second adapter is a second class implementing
 * the same interface rather than a second bag of functions.
 */

import { invoke } from "@tauri-apps/api/core";
import type { CollectionSummary } from "./collection";
import type { SearchRequest, SearchResponse } from "./search";
import type { CreateCollectionRequest, Preset } from "./preset";
import type { ImportRequest, ImportResponse, PointRow } from "./import";
import type { Capabilities } from "./engine";
import type { TelividbClient } from "./port";

/** Names the Rust side registers. A typo here is a runtime failure, so they
 *  are named once and referenced rather than spelled at each call site. */
const enum Command {
  ListCollections = "list_collections",
  Search = "search",
  EngineAddress = "engine_address",
  Capabilities = "capabilities",
  ListPresets = "list_presets",
  CreateCollection = "create_collection",
  ImportPoints = "import_points",
  ListPoints = "list_points",
}

/** A client reaching the engine this window supervises. */
export class TauriClient implements TelividbClient {
  /**
   * Collections this engine holds.
   *
   * @throws the engine's own message when the call is refused. Tauri carries a
   * command failure as a string, and the caller shows it rather than
   * flattening it into "something went wrong".
   */
  public async listCollections(): Promise<CollectionSummary[]> {
    return await invoke<CollectionSummary[]>(Command.ListCollections);
  }

  /** Search one collection. */
  public async search(request: SearchRequest): Promise<SearchResponse> {
    // The argument name must match the Rust parameter name — that is Tauri's
    // contract, and a mismatch surfaces at runtime rather than here.
    return await invoke<SearchResponse>(Command.Search, { request });
  }

  /** Where the engine is listening, for an external tool. */
  public async engineAddress(): Promise<string> {
    return await invoke<string>(Command.EngineAddress);
  }

  /** What the engine can currently do. */
  public async capabilities(): Promise<Capabilities> {
    return await invoke<Capabilities>(Command.Capabilities);
  }

  /** The schemas this build can create a collection from. */
  public async listPresets(): Promise<Preset[]> {
    return await invoke<Preset[]>(Command.ListPresets);
  }

  /** Create a collection from a preset. */
  public async createCollection(
    request: CreateCollectionRequest,
  ): Promise<string> {
    return await invoke<string>(Command.CreateCollection, { request });
  }

  /** Write a batch of text rows into one collection. */
  public async importPoints(request: ImportRequest): Promise<ImportResponse> {
    return await invoke<ImportResponse>(Command.ImportPoints, { request });
  }

  /** The points of one collection. */
  public async listPoints(collection: string): Promise<PointRow[]> {
    return await invoke<PointRow[]>(Command.ListPoints, { collection });
  }
}

/**
 * The port, satisfied over Tauri IPC.
 *
 * A class rather than an object of functions. The behaviour belongs to a type,
 * so a reader finds it by looking at the thing it operates on — the same rule
 * the Rust side follows — and a second adapter is a second class implementing
 * the same interface rather than a second bag of functions.
 */

import { invoke } from "@tauri-apps/api/core";
import { Command } from "./tauri-commands";
import { TauriSettings } from "./tauri-settings";
import type { CollectionSummary } from "./collection";
import type { SearchRequest, SearchResponse } from "./search";
import type { CreateCollectionRequest, Preset } from "./preset";
import type { ImportRequest, ImportResponse, PointRow } from "./import";
import type { Capabilities } from "./engine";
import type { CatalogModel, Installation } from "./model";
import type { TelividbClient } from "./port";

/** Names the Rust side registers. A typo here is a runtime failure, so they
 *  are named once and referenced rather than spelled at each call site. */

/** A client reaching the engine this window supervises. */
export class TauriClient extends TauriSettings implements TelividbClient {
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

  /** Models the engine offers to install. */
  public async listModels(): Promise<CatalogModel[]> {
    return await invoke<CatalogModel[]>(Command.ListModels);
  }

  /**
   * Begin installing a model.
   *
   * Returns as soon as the work is accepted, not when it finishes — a model
   * file is hundreds of megabytes. Poll {@link installation} to follow it.
   *
   * Safe to call twice: the engine returns the running installation rather than
   * starting a second transfer, so the button needs no guard of its own.
   */
  public async installModel(id: string): Promise<Installation> {
    return await invoke<Installation>(Command.InstallModel, { id });
  }

  /** How far an installation has got. */
  public async installation(name: string): Promise<Installation> {
    return await invoke<Installation>(Command.Installation, { name });
  }

  /**
   * Stop an installation.
   *
   * The partial file is kept, so installing again resumes rather than starting
   * over.
   */
  public async cancelInstallation(name: string): Promise<Installation> {
    return await invoke<Installation>(Command.CancelInstallation, { name });
  }
}
